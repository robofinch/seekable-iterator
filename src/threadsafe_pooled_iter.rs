#![expect(unsafe_code, reason = "Return buffer to pool when ThreadsafePoolItem is dropped")]

use core::iter;
use core::mem::ManuallyDrop;
use core::{
    borrow::{Borrow, BorrowMut},
    ops::{Deref, DerefMut},
};
use alloc::{borrow::ToOwned, sync::Arc, vec::Vec};
use std::sync::{Condvar, Mutex, MutexGuard, PoisonError};

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
#[derive(Debug)]
pub struct ThreadsafePooledIter<I, OwnedItem> {
    iter: I,
    pool: ThreadsafePool<OwnedItem>,
}

impl<I, OwnedItem: Default> ThreadsafePooledIter<I, OwnedItem> {
    /// Create a `ThreadsafePooledIter` that can lend out up to `num_buffers` items at a time.
    #[must_use]
    pub fn new(iter: I, num_buffers: usize) -> Self {
        let mut pool = Vec::new();
        pool.reserve_exact(num_buffers);
        pool.extend(iter::repeat_with(|| Some(OwnedItem::default())));
        let pool = ThreadsafePool(Arc::new((
            Mutex::new(pool),
            Condvar::new(),
        )));

        Self { iter, pool }
    }
}

impl<I, OwnedItem> PooledIterator for ThreadsafePooledIter<I, OwnedItem>
where
    I: CursorLendingIterator,
    for<'lend> LentItem<'lend, I>: ToOwned<Owned = OwnedItem>,
{
    type Item = ThreadsafePoolItem<OwnedItem>;

    /// Move the iterator one position forwards, and return the entry at that position.
    /// Returns `None` if the iterator was at the last entry.
    ///
    /// May need to wait for a buffer to become available.
    ///
    /// # Potential Panics or Deadlocks
    /// If `self.buffer_pool_size() == 0`, then this method panics.
    /// This method may also cause a deadlock if no buffers are currently available, and the
    /// current thread needs to make progress in order to release a buffer.
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|item| self.pool.fill_buffer::<I>(&item))
    }

    fn try_next(&mut self) -> Result<Option<Self::Item>, OutOfBuffers> {
         if let Some(item) = self.iter.next() {
            if let Ok(pool_item) = self.pool.try_fill_buffer::<I>(&item) {
                Ok(Some(pool_item))
            } else {
                Err(OutOfBuffers)
            }
        } else {
            Ok(None)
        }
    }

    fn buffer_pool_size(&self) -> usize {
        self.pool.pool_contents().len()
    }

    fn available_buffers(&self) -> usize {
        #[expect(clippy::bool_to_int_with_if, reason = "clarity")]
        self.pool.pool_contents()
            .iter()
            .map(|slot| if slot.is_none() { 1 } else { 0 })
            .sum()
    }
}

impl<I, OwnedItem> CursorPooledIterator for ThreadsafePooledIter<I, OwnedItem>
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
    /// May need to wait for a buffer to become available.
    ///
    /// # Potential Panics or Deadlocks
    /// If `self.buffer_pool_size() == 0`, then this method panics.
    /// This method may also cause a deadlock if no buffers are currently available, and the
    /// current thread needs to make progress in order to release a buffer.
    ///
    /// [valid]: CursorPooledIterator::valid
    #[inline]
    fn current(&self) -> Option<Self::Item> {
        self.iter.current().map(|item| self.pool.fill_buffer::<I>(&item))
    }

    fn try_current(&self) -> Result<Option<Self::Item>, OutOfBuffers> {
        if let Some(item) = self.iter.current() {
            if let Ok(pool_item) = self.pool.try_fill_buffer::<I>(&item) {
                Ok(Some(pool_item))
            } else {
                Err(OutOfBuffers)
            }
        } else {
            Ok(None)
        }
    }

    /// Move the iterator one position back, and return the entry at that position.
    /// Returns `None` if the iterator was at the last entry.
    ///
    /// May need to wait for a buffer to become available.
    ///
    /// # Potential Panics or Deadlocks
    /// If `self.buffer_pool_size() == 0`, then this method panics.
    /// This method may also cause a deadlock if no buffers are currently available, and the
    /// current thread needs to make progress in order to release a buffer.
    fn prev(&mut self) -> Option<Self::Item> {
        self.iter.prev().map(|item| self.pool.fill_buffer::<I>(&item))
    }

    fn try_prev(&mut self) -> Result<Option<Self::Item>, OutOfBuffers> {
         if let Some(item) = self.iter.prev() {
            if let Ok(pool_item) = self.pool.try_fill_buffer::<I>(&item) {
                Ok(Some(pool_item))
            } else {
                Err(OutOfBuffers)
            }
        } else {
            Ok(None)
        }
    }
}

impl<I, OwnedItem, Key, Cmp> Seekable<Key, Cmp> for ThreadsafePooledIter<I, OwnedItem>
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

#[derive(Debug)]
struct ThreadsafePool<OwnedItem>(Arc<(
    Mutex<Vec<Option<OwnedItem>>>,
    Condvar,
)>);

impl<OwnedItem> Clone for ThreadsafePool<OwnedItem> {
    #[inline]
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }

    #[inline]
    fn clone_from(&mut self, source: &Self) {
        self.0.clone_from(&source.0);
    }
}
impl<OwnedItem> ThreadsafePool<OwnedItem> {
    #[inline]
    fn pool_contents(&self) -> MutexGuard<'_, Vec<Option<OwnedItem>>> {
        #[expect(clippy::unwrap_used, reason = "This only ignores Mutex poison")]
        self.0.0.lock().unwrap()
    }

    fn try_fill_buffer<I>(
        &self,
        item: &LentItem<'_, I>,
    ) -> Result<ThreadsafePoolItem<OwnedItem>, MutexGuard<'_, Vec<Option<OwnedItem>>>>
    where
        I: CursorLendingIterator,
        for<'lend> LentItem<'lend, I>: ToOwned<Owned = OwnedItem>,
    {
        let mut pool_contents = self.pool_contents();

        let pool_item = pool_contents.iter_mut()
            .enumerate()
            .find_map(|(idx, slot)| {
                slot.take().map(|mut owned_item| {
                    item.clone_into(&mut owned_item);
                    ThreadsafePoolItem {
                        item:      ManuallyDrop::new(owned_item),
                        pool_slot: idx,
                        pool:      self.clone(),
                    }
                })
            });

        pool_item.ok_or(pool_contents)
    }

    /// # Panics
    /// Panics if `self` was created with `num_buffers` set to `0`.
    #[must_use]
    fn fill_buffer<I>(&self, item: &LentItem<'_, I>) -> ThreadsafePoolItem<OwnedItem>
    where
        I: CursorLendingIterator,
        for<'lend> LentItem<'lend, I>: ToOwned<Owned = OwnedItem>,
    {
        let pool_contents = match self.try_fill_buffer::<I>(item) {
            Ok(pool_item)      => return pool_item,
            Err(pool_contents) => pool_contents,
        };

        assert_ne!(
            pool_contents.len(), 0,
            "A ThreadsafePooledIter with zero buffers had `next`, `current`, or `prev` \
             called on it, which can never succeed",
        );

        self.fill_buffer_fallback::<I>(pool_contents, item)
    }

    #[inline(never)]
    #[must_use]
    fn fill_buffer_fallback<I>(
        &self,
        mut pool_contents: MutexGuard<'_, Vec<Option<OwnedItem>>>,
        item: &LentItem<'_, I>,
    ) -> ThreadsafePoolItem<OwnedItem>
    where
        I: CursorLendingIterator,
        for<'lend> LentItem<'lend, I>: ToOwned<Owned = OwnedItem>,
    {
        loop {
            let poison_result: Result<_, PoisonError<_>> = self.0.1.wait(pool_contents);

            #[expect(clippy::unwrap_used, reason = "only unwrapping Mutex poison")]
            {
                pool_contents = poison_result.unwrap();
            };

            let pool_item = pool_contents.iter_mut()
                .enumerate()
                .find_map(|(idx, slot)| {
                    slot.take().map(|mut owned_item| {
                        item.clone_into(&mut owned_item);
                        ThreadsafePoolItem {
                            item:      ManuallyDrop::new(owned_item),
                            pool_slot: idx,
                            pool:      self.clone(),
                        }
                    })
                });

            if let Some(pool_item) = pool_item {
                return pool_item;
            }
        }
    }
}

/// The type of an item returned by [`ThreadsafePooledIter`].
///
/// The owned item buffer is returned to the [`ThreadsafePooledIter`] when the
/// `ThreadsafePoolItem` is dropped.
#[derive(Debug)]
pub struct ThreadsafePoolItem<OwnedItem> {
    item:      ManuallyDrop<OwnedItem>,
    pool_slot: usize,
    pool:      ThreadsafePool<OwnedItem>,
}

impl<OwnedItem> Drop for ThreadsafePoolItem<OwnedItem> {
    fn drop(&mut self) {
        let mut pool_contents = self.pool.pool_contents();

        #[expect(
            clippy::indexing_slicing,
            reason = "the pool Vec's length is never changed after pool construction, and \
                      `pool_slot` was a valid index into the Vec when the index was made",
        )]
        let pool_slot: &mut Option<OwnedItem> = &mut pool_contents[self.pool_slot];

        // SAFETY:
        // We must never again use the `self.item` `ManuallyDrop` value.
        // This is last line of the destructor, so that is trivially true.
        *pool_slot = Some(unsafe { ManuallyDrop::take(&mut self.item) });
    }
}

impl<OwnedItem> Deref for ThreadsafePoolItem<OwnedItem> {
    type Target = OwnedItem;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.item
    }
}

impl<OwnedItem> DerefMut for ThreadsafePoolItem<OwnedItem> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.item
    }
}

impl<OwnedItem> Borrow<OwnedItem> for ThreadsafePoolItem<OwnedItem> {
    #[inline]
    fn borrow(&self) -> &OwnedItem {
        self
    }
}

impl<OwnedItem> BorrowMut<OwnedItem> for ThreadsafePoolItem<OwnedItem> {
    #[inline]
    fn borrow_mut(&mut self) -> &mut OwnedItem {
        self
    }
}

impl<OwnedItem> AsRef<OwnedItem> for ThreadsafePoolItem<OwnedItem> {
    #[inline]
    fn as_ref(&self) -> &OwnedItem {
        self
    }
}

impl<OwnedItem> AsMut<OwnedItem> for ThreadsafePoolItem<OwnedItem> {
    #[inline]
    fn as_mut(&mut self) -> &mut OwnedItem {
        self
    }
}
