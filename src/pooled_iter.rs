#![expect(
    unsafe_code,
    reason = "RefCell can easily be determined to be unnecessary; also, return buffer to pool when \
              PoolItem is dropped",
)]

use core::iter;
use core::{cell::UnsafeCell, mem::ManuallyDrop};
use core::{
    borrow::{Borrow, BorrowMut},
    ops::{Deref, DerefMut},
};
use alloc::{borrow::ToOwned, rc::Rc, vec::Vec};

use crate::{comparator::Comparator, lending_iterator_support::LentItem, seekable::Seekable};
use crate::{
    pooled::{OutOfBuffers, PooledIterator},
    cursor::{CursorLendingIterator, CursorPooledIterator},
};


/// Convert a [`CursorLendingIterator`] into a [`CursorPooledIterator`] by storing recently
/// accessed items in reusable buffers.
///
/// This effectively allows the iterator to lend out multiple items at once, unlike a lending
/// iterator which can only lend out one. This comes primarily at the cost of extra copying
/// into buffers, and in memory usage. The costs of allocating buffers is likely amortized by
/// their reuse.
///
/// The user of a `PooledIter` is required not to attempt to get more [`PoolItem`]s than there
/// are buffers in the `PooledIter`; because `PooledIter` can only be used from a single thread,
/// it is impossible for a buffer to be returned to the iterator while [`PooledIter::next`]
/// is running, for example, unlike with the `ThreadsafePooledIter` type. Therefore, `PooledIter`
/// panics in such a scenario.
#[derive(Debug)]
pub struct PooledIter<I, OwnedItem> {
    iter: I,
    pool: Pool<OwnedItem>,
}

impl<I, OwnedItem: Default> PooledIter<I, OwnedItem> {
    /// Create a `PooledIter` that can lend out up to `num_buffers` items at a time.
    ///
    /// The user of a `PooledIter` is required not to attempt to get more than `num_buffers`
    /// [`PoolItem`]s from this `PooledIter` at a time; because `PooledIter` can only be used from
    /// a single thread, it is impossible for a buffer to be returned to the iterator while
    /// [`PooledIter::next`] is running, for example, unlike with the `ThreadsafePooledIter` type.
    /// Therefore, `PooledIter` panics in such a scenario.
    #[must_use]
    pub fn new(iter: I, num_buffers: usize) -> Self {
        let mut pool = Vec::new();
        pool.reserve_exact(num_buffers);
        pool.extend(iter::repeat_with(|| Some(OwnedItem::default())));
        let pool = Pool(Rc::new(UnsafeCell::new(pool)));

        Self { iter, pool }
    }
}

impl<I, OwnedItem> PooledIterator for PooledIter<I, OwnedItem>
where
    I: CursorLendingIterator,
    for<'lend> LentItem<'lend, I>: ToOwned<Owned = OwnedItem>,
{
    type Item = PoolItem<OwnedItem>;

    /// Move the iterator one position forwards, and return the entry at that position.
    /// Returns `None` if the iterator was at the last entry.
    ///
    /// # Panics
    /// Panics if there are no buffers available.
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|item| self.pool.fill_buffer::<I>(item))
    }

    fn try_next(&mut self) -> Result<Option<Self::Item>, OutOfBuffers> {
         if let Some(item) = self.iter.next() {
            if let Some(pool_item) = self.pool.try_fill_buffer::<I>(item) {
                Ok(Some(pool_item))
            } else {
                Err(OutOfBuffers)
            }
        } else {
            Ok(None)
        }
    }

    fn buffer_pool_size(&self) -> usize {
        let pool_contents: *mut Vec<Option<OwnedItem>> = self.pool.0.get();
        // SAFETY:
        // We only need to ensure that this access is unique in order for this to be sound.
        // See the note on `Pool`. This is one of only four functions that access the
        // `UnsafeCell` contents, and none allow a reference to escape, or call each other.
        let pool_contents: &mut Vec<Option<OwnedItem>> = unsafe { &mut *pool_contents };

        pool_contents.len()
    }

    fn available_buffers(&self) -> usize {
        let pool_contents: *mut Vec<Option<OwnedItem>> = self.pool.0.get();
        // SAFETY:
        // We only need to ensure that this access is unique in order for this to be sound.
        // See the note on `Pool`. This is one of only four functions that access the
        // `UnsafeCell` contents, and none allow a reference to escape, or call each other.
        let pool_contents: &mut Vec<Option<OwnedItem>> = unsafe { &mut *pool_contents };

        #[expect(clippy::bool_to_int_with_if, reason = "clarity")]
        pool_contents.iter()
            .map(|slot| if slot.is_none() { 1 } else { 0 })
            .sum()
    }
}

impl<I, OwnedItem> CursorPooledIterator for PooledIter<I, OwnedItem>
where
    I: CursorLendingIterator,
    for<'lend> LentItem<'lend, I>: ToOwned<Owned = OwnedItem>,
{
    #[inline]
    fn valid(&self) -> bool {
        self.iter.valid()
    }

    /// Get the current value the iterator is at, if the iterator is [valid].
    ///
    /// # Panics
    /// Panics if there are no buffers available.
    ///
    /// [valid]: CursorPooledIterator::valid
    #[inline]
    fn current(&self) -> Option<Self::Item> {
        self.iter.current().map(|item| self.pool.fill_buffer::<I>(item))
    }

    fn try_current(&self) -> Result<Option<Self::Item>, OutOfBuffers> {
        if let Some(item) = self.iter.current() {
            if let Some(pool_item) = self.pool.try_fill_buffer::<I>(item) {
                Ok(Some(pool_item))
            } else {
                Err(OutOfBuffers)
            }
        } else {
            Ok(None)
        }
    }

    /// Move the iterator one position back, and return the entry at that position.
    /// Returns `None` if the iterator was at the first entry.
    ///
    /// Some iterator implementations used as `I` may have worse performance for backwards
    /// iteration than forwards iteration, so prefer to not use `prev`.
    ///
    /// # Panics
    /// Panics if there are no buffers available.
    fn prev(&mut self) -> Option<Self::Item> {
        self.iter.prev().map(|item| self.pool.fill_buffer::<I>(item))
    }

    fn try_prev(&mut self) -> Result<Option<Self::Item>, OutOfBuffers> {
         if let Some(item) = self.iter.prev() {
            if let Some(pool_item) = self.pool.try_fill_buffer::<I>(item) {
                Ok(Some(pool_item))
            } else {
                Err(OutOfBuffers)
            }
        } else {
            Ok(None)
        }
    }
}

impl<I, OwnedItem, Key, Cmp> Seekable<Key, Cmp> for PooledIter<I, OwnedItem>
where
    I:   CursorLendingIterator + Seekable<Key, Cmp>,
    Key: ?Sized,
    Cmp: Comparator<Key>,
    for<'lend> LentItem<'lend, I>: ToOwned<Owned = OwnedItem>,
{
    #[inline]
    fn reset(&mut self) {
        self.iter.reset();
    }

    fn seek(&mut self, min_bound: &Key) {
        self.iter.seek(min_bound);
    }

    fn seek_before(&mut self, strict_upper_bound: &Key) {
        self.iter.seek_before(strict_upper_bound);
    }

    #[inline]
    fn seek_to_first(&mut self) {
        self.iter.seek_to_first();
    }

    fn seek_to_last(&mut self) {
        self.iter.seek_to_last();
    }
}

/// # Safety
/// `PooledIter::buffer_pool_size`, `PooledIter::available_buffers`,
/// `Pool::try_fill_buffer`, and `PoolItem::drop` access the `UnsafeCell`. Those functions
/// do not call each other, and do not leak references to the `UnsafeCell`'s contents.
///
/// No other functions should directly access the `UnsafeCell` contents.
#[derive(Debug)]
struct Pool<OwnedItem>(Rc<UnsafeCell<Vec<Option<OwnedItem>>>>);

impl<OwnedItem> Clone for Pool<OwnedItem> {
    #[inline]
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }

    #[inline]
    fn clone_from(&mut self, source: &Self) {
        self.0.clone_from(&source.0);
    }
}
impl<OwnedItem> Pool<OwnedItem> {
    #[expect(
        clippy::needless_pass_by_value,
        reason = "lent item is likely already a reference/pointer of some sort",
    )]
    #[must_use]
    fn try_fill_buffer<I>(
        &self,
        item: LentItem<'_, I>,
    ) -> Option<PoolItem<OwnedItem>>
    where
        I: CursorLendingIterator,
        for<'lend> LentItem<'lend, I>: ToOwned<Owned = OwnedItem>,
    {
        let pool_contents: *mut Vec<Option<OwnedItem>> = self.0.get();
        // SAFETY:
        // We only need to ensure that this access is unique in order for this to be sound.
        // See the note on `Pool`. This is one of only four functions that access the
        // `UnsafeCell` contents, and none allow a reference to escape, or call each other.
        let pool_contents: &mut Vec<Option<OwnedItem>> = unsafe { &mut *pool_contents };

        pool_contents.iter_mut()
            .enumerate()
            .find_map(|(idx, slot)| {
                slot.take().map(|mut owned_item| {
                    item.clone_into(&mut owned_item);
                    PoolItem {
                        item:      ManuallyDrop::new(owned_item),
                        pool_slot: idx,
                        pool:      self.clone(),
                    }
                })
            })
    }

    /// # Panics
    /// Panics if `self` is out of buffers. Because the pool and its buffers are not `Sync` or
    /// `Send`, no buffer could ever become available while this function runs.
    #[must_use]
    fn fill_buffer<I>(&self, item: LentItem<'_, I>) -> PoolItem<OwnedItem>
    where
        I: CursorLendingIterator,
        for<'lend> LentItem<'lend, I>: ToOwned<Owned = OwnedItem>,
    {
        #[expect(
            clippy::expect_used,
            reason = "this call will never succeed. Also, this is documented.",
        )]
        self.try_fill_buffer::<I>(item)
            .expect(
                "A single-threaded PooledIter ran out of buffers, \
                 and had `next`, `current`, or `prev` called on it, which can never succeed",
            )
    }
}

/// The type of an item returned by [`PooledIter`].
///
/// The owned item buffer is returned to [`PooledIter`] when the `PoolItem` is dropped.
#[derive(Debug)]
pub struct PoolItem<OwnedItem> {
    item:      ManuallyDrop<OwnedItem>,
    pool_slot: usize,
    pool:      Pool<OwnedItem>,
}

impl<OwnedItem> Drop for PoolItem<OwnedItem> {
    fn drop(&mut self) {
        let pool_contents: *mut Vec<Option<OwnedItem>> = self.pool.0.get();
        // SAFETY:
        // We only need to ensure that this access is unique in order for this to be sound.
        // See the note on `Pool`. This is one of only four functions that access the
        // `UnsafeCell` contents, and none allow a reference to escape, or call each other.
        let pool_contents: &mut Vec<Option<OwnedItem>> = unsafe { &mut *pool_contents };

        #[expect(
            clippy::indexing_slicing,
            reason = "the pool Vec's length is never changed after pool construction, \
                      and `pool_slot` was a valid index into the Vec when the PoolItem was made",
        )]
        let pool_slot: &mut Option<OwnedItem> = &mut pool_contents[self.pool_slot];

        // SAFETY:
        // We must never again use the `self.item` `ManuallyDrop` value.
        // This is last line of the destructor, so that is trivially true.
        *pool_slot = Some(unsafe { ManuallyDrop::take(&mut self.item) });
    }
}

impl<OwnedItem> Deref for PoolItem<OwnedItem> {
    type Target = OwnedItem;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.item
    }
}

impl<OwnedItem> DerefMut for PoolItem<OwnedItem> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.item
    }
}

impl<OwnedItem> Borrow<OwnedItem> for PoolItem<OwnedItem> {
    #[inline]
    fn borrow(&self) -> &OwnedItem {
        self
    }
}

impl<OwnedItem> BorrowMut<OwnedItem> for PoolItem<OwnedItem> {
    #[inline]
    fn borrow_mut(&mut self) -> &mut OwnedItem {
        self
    }
}

impl<OwnedItem> AsRef<OwnedItem> for PoolItem<OwnedItem> {
    #[inline]
    fn as_ref(&self) -> &OwnedItem {
        self
    }
}

impl<OwnedItem> AsMut<OwnedItem> for PoolItem<OwnedItem> {
    #[inline]
    fn as_mut(&mut self) -> &mut OwnedItem {
        self
    }
}
