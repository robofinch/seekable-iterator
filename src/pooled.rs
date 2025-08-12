use core::error::Error;
use core::fmt::{Display, Formatter, Result as FmtResult};

#[cfg(feature = "lender")]
use crate::lender_adapter::PooledLenderAdapter;
#[cfg(feature = "lending-iterator")]
use crate::lending_iterator_adapter::PooledLendingIteratorAdapter;


/// A `PooledIterator` is similar to a lending iterator (which can lend one item at a time), but
/// can make use of a buffer pool to lend out multiple items at a time.
///
/// Implementations may or may not be threadsafe. Even if an implementation is threadsafe,
/// items newly-added to the backing collection of the iterator may or may not be seen
/// immediately by other threads.
///
/// [`FusedIterator`]: core::iter::FusedIterator
pub trait PooledIterator {
    type Item;

    /// # Potential Panics or Deadlocks
    /// If `self.buffer_pool_size() == 0`, then this method may panic or deadlock.
    fn next(&mut self) -> Option<Self::Item>;

    #[must_use]
    fn buffers_available(&self) -> bool;

    /// # Errors
    /// Returns an error if no buffers were available.
    fn try_next(&mut self) -> Result<Option<Self::Item>, OutOfBuffers>;

    #[must_use]
    fn buffer_pool_size(&self) -> usize;

    #[must_use]
    fn available_buffers(&self) -> usize;

    /// # Potential Panics or Deadlocks
    /// If `self.buffer_pool_size() == 0`, then methods of the returned iterator might panic or
    /// deadlock.
    #[cfg(feature = "lender")]
    #[inline]
    #[must_use]
    fn into_lender(self) -> PooledLenderAdapter<Self> where Self: Sized {
        PooledLenderAdapter::new(self)
    }

    /// # Potential Panics or Deadlocks
    /// If `self.buffer_pool_size() == 0`, then methods of the returned iterator might panic or
    /// deadlock.
    #[cfg(feature = "lending-iterator")]
    #[inline]
    #[must_use]
    fn into_lending_iterator(self) -> PooledLendingIteratorAdapter<Self> where Self: Sized {
        PooledLendingIteratorAdapter::new(self)
    }
}

/// An error that may be returned if no buffer pools were available in a [`PooledIterator`],
/// instead of waiting for a buffer to become available.
#[derive(Debug, Clone, Copy)]
pub struct OutOfBuffers;

impl Display for OutOfBuffers {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "a pooled iterator operation was not performed because there were no buffers available",
        )
    }
}

impl Error for OutOfBuffers {}
