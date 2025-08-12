use crate::pooled::{OutOfBuffers, PooledIterator};
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
    #[must_use]
    fn valid(&self) -> bool;

    #[must_use]
    fn current(&self) -> Option<Self::Item>;

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
pub trait CursorLendingIterator {
    type Item<'a> where Self: 'a;

    #[must_use]
    fn valid(&self) -> bool;

    fn next(&mut self) -> Option<Self::Item<'_>>;

    #[must_use]
    fn current(&self) -> Option<Self::Item<'_>>;

    fn prev(&mut self) -> Option<Self::Item<'_>>;

    #[cfg(feature = "lender")]
    #[inline]
    #[must_use]
    fn into_lender(self) -> LenderAdapter<Self> where Self: Sized {
        LenderAdapter::new(self)
    }

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
    #[must_use]
    fn valid(&self) -> bool;

    #[must_use]
    fn current(&self) -> Option<Self::Item>;

    fn prev(&mut self) -> Option<Self::Item>;

    /// # Errors
    /// Returns an error if no buffers were available.
    fn try_current(&self) -> Result<Option<Self::Item>, OutOfBuffers>;

    /// # Errors
    /// Returns an error if no buffers were available.
    fn try_prev(&mut self) -> Result<Option<Self::Item>, OutOfBuffers>;
}
