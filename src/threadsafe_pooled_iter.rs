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
pub struct ThreadsafePooledIter<I, BorrowedItem: ToOwned> {
    iter: I,
    pool: SharedBoundedPool<BorrowedItem::Owned, ResetNothing>,
}

impl<I, BorrowedItem> ThreadsafePooledIter<I, BorrowedItem>
where
    BorrowedItem:        ToOwned,
    BorrowedItem::Owned: Default,
{
    /// Create a `ThreadsafePooledIter` that can lend out up to `num_buffers` items at a time.
    #[must_use]
    pub fn new(iter: I, num_buffers: usize) -> Self {
        let pool = SharedBoundedPool::new_default_without_reset(num_buffers);

        Self { iter, pool }
    }
}

impl<I, BorrowedItem> ThreadsafePooledIter<I, BorrowedItem>
where
    I:                             CursorLendingIterator,
    BorrowedItem:                  ToOwned,
    for<'lend> LentItem<'lend, I>: Borrow<BorrowedItem>,
{
    /// # Potential Panics or Deadlocks
    /// If `self.buffer_pool_size() == 0`, then this method panics.
    /// This method may also cause a deadlock if no buffers are currently available, and the
    /// current thread needs to make progress in order to release a buffer.
    #[expect(clippy::needless_pass_by_value, reason = "lent item usually consists of references")]
    #[inline]
    fn fill_buffer(
        pool: &SharedBoundedPool<BorrowedItem::Owned, ResetNothing>,
        item: LentItem<'_, I>,
    ) -> ThreadsafePoolItem<BorrowedItem::Owned> {
        let mut pool_item = pool.get();
        item.borrow().clone_into(&mut pool_item);
        ThreadsafePoolItem(pool_item)
    }
}

impl<I, BorrowedItem> PooledIterator for ThreadsafePooledIter<I, BorrowedItem>
where
    I:                             CursorLendingIterator,
    BorrowedItem:                  ToOwned,
    for<'lend> LentItem<'lend, I>: Borrow<BorrowedItem>,
{
    type Item = ThreadsafePoolItem<BorrowedItem::Owned>;

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
        let mut buffer = self.pool.try_get()
            .map_err(|ResourcePoolEmpty| OutOfBuffers)?;

        if let Some(item) = self.iter.next() {
            item.borrow().clone_into(&mut buffer);
            Ok(Some(ThreadsafePoolItem(buffer)))
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

impl<I, BorrowedItem> CursorPooledIterator for ThreadsafePooledIter<I, BorrowedItem>
where
    I:                             CursorLendingIterator,
    BorrowedItem:                  ToOwned,
    for<'lend> LentItem<'lend, I>: Borrow<BorrowedItem>,
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
        let mut buffer = self.pool.try_get()
            .map_err(|ResourcePoolEmpty| OutOfBuffers)?;

        if let Some(item) = self.iter.current() {
            item.borrow().clone_into(&mut buffer);
            Ok(Some(ThreadsafePoolItem(buffer)))
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
        let mut buffer = self.pool.try_get()
            .map_err(|ResourcePoolEmpty| OutOfBuffers)?;

        if let Some(item) = self.iter.prev() {
            item.borrow().clone_into(&mut buffer);
            Ok(Some(ThreadsafePoolItem(buffer)))
        } else {
            Ok(None)
        }
    }
}

impl<I, BorrowedItem, Key, Cmp> Seekable<Key, Cmp> for ThreadsafePooledIter<I, BorrowedItem>
where
    I:                             CursorLendingIterator + Seekable<Key, Cmp>,
    BorrowedItem:                  ToOwned,
    Key:                           ?Sized,
    Cmp:                           Comparator<Key>,
    for<'lend> LentItem<'lend, I>: Borrow<BorrowedItem>,
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


#[cfg(test)]
mod tests {
    use crate::test_iter::TestIter;
    use super::*;


    #[test]
    fn threadsafe_pooled_test_iter() {
        let data: &[u8] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9].as_slice();
        let mut iter = ThreadsafePooledIter::<_, u8>::new(TestIter::new(data).unwrap(), 2);

        // Hold one buffer the entire time
        let first = iter.next().unwrap();
        assert_eq!(*first, 0);

        for i in 1..=9 {
            assert!(iter.valid());
            let next = iter.next().unwrap();
            // Both of the two buffers are in use
            assert!(iter.try_next().is_err());
            assert_eq!(*next, i);
        }
        drop(first);

        assert!(iter.next().is_none());
        iter.prev();

        for i in (0..9).rev() {
            let current = iter.current();
            let prev = iter.prev().unwrap();

            // Both of the two buffers are in use
            assert!(iter.try_next().is_err());
            assert!(iter.valid());

            // This drops `current`
            assert!(!current.is_some_and(|curr| *curr == *prev));

            let new_current = iter.current().unwrap();

            assert_eq!(*prev, i);
            assert_eq!(*new_current, i);
        }
    }

    #[test]
    fn seek_test() {
        let data: &[u8] = [0, 1, 2, 3, 4, 4, 4, 4, 4, 4, 4, 4, 5, 6, 7, 8, 9, 99].as_slice();
        let mut iter = ThreadsafePooledIter::<_, u8>::new(TestIter::new(data).unwrap(), 1);

        iter.seek_to_first();
        assert_eq!(*iter.current().unwrap(), 0);

        iter.seek(&0);
        assert_eq!(*iter.current().unwrap(), 0);

        iter.seek(&1);
        assert_eq!(*iter.current().unwrap(), 1);

        iter.seek(&9);
        assert_eq!(*iter.current().unwrap(), 9);

        iter.seek(&8);
        assert_eq!(*iter.current().unwrap(), 8);

        iter.seek(&10);
        assert_eq!(*iter.current().unwrap(), 99);

        iter.seek_before(&92);
        assert_eq!(*iter.current().unwrap(), 9);

        iter.seek_before(&99);
        assert_eq!(*iter.current().unwrap(), 9);

        iter.seek_before(&100);
        assert_eq!(*iter.current().unwrap(), 99);

        iter.seek_before(&1);
        assert_eq!(*iter.current().unwrap(), 0);

        iter.seek_before(&0);
        assert!(!iter.valid());

        iter.seek(&100);
        assert!(!iter.valid());

        iter.seek(&99);
        assert_eq!(*iter.current().unwrap(), 99);

        iter.seek_to_last();
        assert_eq!(*iter.current().unwrap(), 99);

        iter.seek_before(&4);
        assert_eq!(*iter.current().unwrap(), 3);
    }
}
