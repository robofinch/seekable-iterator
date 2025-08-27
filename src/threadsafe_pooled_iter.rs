use core::{
    borrow::{Borrow, BorrowMut},
    ops::{Deref, DerefMut},
};
use alloc::borrow::ToOwned;

use anchored_pool::{PooledResource, ResetNothing, ResourcePoolEmpty, SharedBoundedPool};

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
    pool: SharedBoundedPool<OwnedItem, ResetNothing>,
}

impl<I, OwnedItem: Default> ThreadsafePooledIter<I, OwnedItem> {
    /// Create a `ThreadsafePooledIter` that can lend out up to `num_buffers` items at a time.
    #[must_use]
    pub fn new(iter: I, num_buffers: usize) -> Self {
        let pool = SharedBoundedPool::new_default_without_reset(num_buffers);

        Self { iter, pool }
    }
}

impl<I, OwnedItem> ThreadsafePooledIter<I, OwnedItem>
where
    I: CursorLendingIterator,
    for<'lend> LentItem<'lend, I>: ToOwned<Owned = OwnedItem>,
{
    /// # Potential Panics or Deadlocks
    /// If `self.buffer_pool_size() == 0`, then this method panics.
    /// This method may also cause a deadlock if no buffers are currently available, and the
    /// current thread needs to make progress in order to release a buffer.
    #[expect(clippy::needless_pass_by_value, reason = "lent item usually consists of references")]
    #[inline]
    fn fill_buffer(
        pool: &SharedBoundedPool<OwnedItem, ResetNothing>,
        item: LentItem<'_, I>,
    ) -> ThreadsafePoolItem<OwnedItem> {
        let mut pool_item = pool.get();
        item.clone_into(&mut pool_item);
        ThreadsafePoolItem(pool_item)
    }

    #[expect(clippy::needless_pass_by_value, reason = "lent item usually consists of references")]
    #[inline]
    fn try_fill_buffer(
        pool: &SharedBoundedPool<OwnedItem, ResetNothing>,
        item: LentItem<'_, I>,
    ) -> Result<ThreadsafePoolItem<OwnedItem>, OutOfBuffers> {
        pool.try_get()
            .map(|mut pool_item| {
                item.clone_into(&mut pool_item);
                ThreadsafePoolItem(pool_item)
            })
            .map_err(|ResourcePoolEmpty| OutOfBuffers)
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
    /// Returns `None` if the iterator was at the last entry.
    ///
    /// May need to wait for a buffer to become available.
    ///
    /// # Potential Panics or Deadlocks
    /// If `self.buffer_pool_size() == 0`, then this method panics.
    /// This method may also cause a deadlock if no buffers are currently available, and the
    /// current thread needs to make progress in order to release a buffer.
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

/// The type of an item returned by [`ThreadsafePooledIter`].
///
/// The owned item buffer is returned to the [`ThreadsafePooledIter`] when the
/// `ThreadsafePoolItem` is dropped.
#[derive(Debug)]
pub struct ThreadsafePoolItem<OwnedItem>(
    PooledResource<SharedBoundedPool<OwnedItem, ResetNothing>, OwnedItem>,
);

impl<OwnedItem> Deref for ThreadsafePoolItem<OwnedItem> {
    type Target = OwnedItem;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<OwnedItem> DerefMut for ThreadsafePoolItem<OwnedItem> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
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

// TODO: a bunch of tests
