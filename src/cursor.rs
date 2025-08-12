use crate::{
    lending_iterator_support::{LendItem, LentItem},
    pooled::{OutOfBuffers, PooledIterator},
};
#[cfg(feature = "lender")]
use crate::lender_adapter::LenderAdapter;
#[cfg(feature = "lending-iterator")]
use crate::lending_iterator_adapter::LendingIteratorAdapter;


/// A `CursorIterator` provides access to the entries of some sorted collection, and can move its
/// current position in either direction.
///
/// Conceptually, it is circular, and its initial position is before the first entry and after the
/// last entry. As such, it is not a [`FusedIterator`], as continuing to call `next()` at the
/// end of iteration wraps around to the start. (Note that if the collection is empty, then the
/// iterator will remain at that phantom position.)
///
/// Implementations may or may not be threadsafe. Even if an implementation is threadsafe,
/// newly-added entries may or may not be seen immediately by other threads.
///
/// Forwards iteration should be preferred over backwards iteration.
///
/// [`FusedIterator`]: core::iter::FusedIterator
pub trait CursorIterator: Iterator {
    /// Determine whether the iterator is currently at any value in the collection.
    /// If the iterator is invalid, then it is conceptually one position before the first entry
    /// and one position after the last entry. (Or, there may be no entries.)
    ///
    /// [`current()`] will be `Some` if and only if the iterator is valid.
    ///
    /// [`current()`]: CursorIterator::current
    #[must_use]
    fn valid(&self) -> bool;

    /// Get the current value the iterator is at, if the iterator is [valid].
    ///
    /// [valid]: CursorIterator::valid
    #[must_use]
    fn current(&self) -> Option<Self::Item>;

    /// Move the iterator one position back, and return the entry at that position.
    /// Returns `None` if the iterator was at the first entry.
    fn prev(&mut self) -> Option<Self::Item>;
}

/// A `CursorLendingIterator` provides access to the entries of some sorted collection, and can
/// move its current position in either direction.
///
/// As a lending iterator, only one entry can be accessed at a time.
///
/// Conceptually, it is circular, and its initial position is before the first entry and after the
/// last entry. As such, it is not a [`FusedIterator`], as continuing to call `next()` at the
/// end of iteration wraps around to the start. (Note that if the collection is empty, then the
/// iterator will remain at that phantom position.)
///
/// Implementations may or may not be threadsafe. Even if an implementation is threadsafe,
/// newly-added entries may or may not be seen immediately by other threads.
///
/// Forwards iteration should be preferred over backwards iteration.
///
/// [`FusedIterator`]: core::iter::FusedIterator
pub trait CursorLendingIterator: for<'a> LendItem<'a> {
    /// Determine whether the iterator is currently at any value in the collection.
    /// If the iterator is invalid, then it is conceptually one position before the first entry
    /// and one position after the last entry. (Or, there may be no entries.)
    ///
    /// [`current()`] will be `Some` if and only if the iterator is valid.
    ///
    /// [`current()`]: CursorLendingIterator::current
    #[must_use]
    fn valid(&self) -> bool;

    /// Move the iterator one position forwards, and return the entry at that position.
    /// Returns `None` if the iterator was at the last entry.
    fn next(&mut self) -> Option<LentItem<'_, Self>>;

    /// Get the current value the iterator is at, if the iterator is [valid].
    ///
    /// [valid]: CursorLendingIterator::valid
    #[must_use]
    fn current(&self) -> Option<LentItem<'_, Self>>;

    /// Move the iterator one position back, and return the entry at that position.
    /// Returns `None` if the iterator was at the first entry.
    fn prev(&mut self) -> Option<LentItem<'_, Self>>;

    /// Convert the `CursorLendingIterator` into a [`lender::Lender`] lending iterator.
    ///
    /// The seekability and access to cursor methods are preserved, though none of the
    /// `Cursor*Iterator` or `Seekable*Iterator` traits are implemented for the adaptor, in order
    /// to avoid conflicts with the `next()` method name.
    #[cfg(feature = "lender")]
    #[inline]
    #[must_use]
    fn into_lender(self) -> LenderAdapter<Self> where Self: Sized {
        LenderAdapter::new(self)
    }

    /// Convert the `CursorLendingIterator` into a [`lending_iterator::LendingIterator`].
    ///
    /// The seekability and access to cursor methods are preserved, though none of the
    /// `Cursor*Iterator` or `Seekable*Iterator` traits are implemented for the adaptor, in order
    /// to avoid conflicts with the `next()` method name.
    #[cfg(feature = "lending-iterator")]
    #[inline]
    #[must_use]
    fn into_lending_iterator(self) -> LendingIteratorAdapter<Self> where Self: Sized {
        LendingIteratorAdapter::new(self)
    }
}

/// A `CursorPooledIterator` provides access to the entries of some sorted collection, and can
/// move its current position in either direction.
///
/// The iterator is similar to a lending iterator (which can lend one item at a time), but can
/// make use of a buffer pool to lend out multiple items at a time.
///
/// Conceptually, it is circular, and its initial position is before the first entry and after the
/// last entry. As such, it is not a [`FusedIterator`], as continuing to call `next()` at the
/// end of iteration wraps around to the start. (Note that if the collection is empty, then the
/// iterator will remain at that phantom position.)
///
/// Implementations may or may not be threadsafe. Even if an implementation is threadsafe,
/// newly-added entries may or may not be seen immediately by other threads.
///
/// Forwards iteration should be preferred over backwards iteration.
///
/// [`FusedIterator`]: core::iter::FusedIterator
pub trait CursorPooledIterator: PooledIterator {
    /// Determine whether the iterator is currently at any value in the collection.
    /// If the iterator is invalid, then it is conceptually one position before the first entry
    /// and one position after the last entry. (Or, there may be no entries.)
    ///
    /// [`current()`] will be `Some` if and only if the iterator is valid.
    ///
    /// [`current()`]: CursorPooledIterator::current
    #[must_use]
    fn valid(&self) -> bool;

    /// Get the current value the iterator is at, if the iterator is [valid].
    ///
    /// May need to wait for a buffer to become available.
    ///
    /// # Potential Panics or Deadlocks
    /// If `self.buffer_pool_size() == 0`, then this method may panic or deadlock.
    ///
    /// [valid]: CursorPooledIterator::valid
    #[must_use]
    fn current(&self) -> Option<Self::Item>;

    /// Move the iterator one position back, and return the entry at that position.
    /// Returns `None` if the iterator was at the first entry.
    ///
    /// May need to wait for a buffer to become available.
    ///
    /// # Potential Panics or Deadlocks
    /// If `self.buffer_pool_size() == 0`, then this method may panic or deadlock.
    fn prev(&mut self) -> Option<Self::Item>;

    /// If a buffer is available, get the current value the iterator is at, if the iterator is
    /// [valid].
    ///
    /// # Errors
    /// Returns an error if no buffers were available.
    ///
    /// [valid]: CursorPooledIterator::valid
    fn try_current(&self) -> Result<Option<Self::Item>, OutOfBuffers>;

    /// If a buffer is available, move the iterator one position back, and return the entry at
    /// that position. Returns `Ok(None)` if the iterator was at the first entry.
    ///
    /// # Errors
    /// Returns an error if no buffers were available.
    fn try_prev(&mut self) -> Result<Option<Self::Item>, OutOfBuffers>;
}
