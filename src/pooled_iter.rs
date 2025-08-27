use core::{
    borrow::{Borrow, BorrowMut},
    ops::{Deref, DerefMut},
};
use alloc::borrow::ToOwned;

use anchored_pool::{PooledResource, ResetNothing, ResourcePoolEmpty, BoundedPool};

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
    pool: BoundedPool<OwnedItem, ResetNothing>,
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
        let pool = BoundedPool::new_default_without_reset(num_buffers);

        Self { iter, pool }
    }
}

impl<I, OwnedItem> PooledIter<I, OwnedItem>
where
    I: CursorLendingIterator,
    for<'lend> LentItem<'lend, I>: ToOwned<Owned = OwnedItem>,
{
    /// # Panics
    /// Panics if there are no buffers available.
    #[expect(clippy::needless_pass_by_value, reason = "lent item usually consists of references")]
    #[inline]
    fn fill_buffer(
        pool: &BoundedPool<OwnedItem, ResetNothing>,
        item: LentItem<'_, I>,
    ) -> PoolItem<OwnedItem> {
        let mut pool_item = pool.get();
        item.clone_into(&mut pool_item);
        PoolItem(pool_item)
    }

    #[expect(clippy::needless_pass_by_value, reason = "lent item usually consists of references")]
    #[inline]
    fn try_fill_buffer(
        pool: &BoundedPool<OwnedItem, ResetNothing>,
        item: LentItem<'_, I>,
    ) -> Result<PoolItem<OwnedItem>, OutOfBuffers> {
        pool.try_get()
            .map(|mut pool_item| {
                item.clone_into(&mut pool_item);
                PoolItem(pool_item)
            })
            .map_err(|ResourcePoolEmpty| OutOfBuffers)
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
        self.iter.next().map(|item| Self::fill_buffer(&self.pool, item))
    }

    fn try_next(&mut self) -> Result<Option<Self::Item>, OutOfBuffers> {
        if let Some(item) = self.iter.next() {
            Self::try_fill_buffer(&self.pool, item).map(Some)
        } else {
            Ok(None)
        }
    }

    #[inline]
    fn buffer_pool_size(&self) -> usize {
        self.pool.pool_size()
    }

    fn available_buffers(&self) -> usize {
        self.pool.available_resources()
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
        self.iter.current().map(|item| Self::fill_buffer(&self.pool, item))
    }

    fn try_current(&self) -> Result<Option<Self::Item>, OutOfBuffers> {
        if let Some(item) = self.iter.current() {
            Self::try_fill_buffer(&self.pool, item).map(Some)
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
        self.iter.prev().map(|item| Self::fill_buffer(&self.pool, item))
    }

    fn try_prev(&mut self) -> Result<Option<Self::Item>, OutOfBuffers> {
        if let Some(item) = self.iter.prev() {
            Self::try_fill_buffer(&self.pool, item).map(Some)
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

/// The type of an item returned by [`PooledIter`].
///
/// The owned item buffer is returned to [`PooledIter`] when the `PoolItem` is dropped.
#[derive(Debug)]
pub struct PoolItem<OwnedItem>(
    PooledResource<BoundedPool<OwnedItem, ResetNothing>, OwnedItem>,
);

impl<OwnedItem> Deref for PoolItem<OwnedItem> {
    type Target = OwnedItem;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<OwnedItem> DerefMut for PoolItem<OwnedItem> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
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

// TODO: a bunch of tests
