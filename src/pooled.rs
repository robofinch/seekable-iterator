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
    /// The item of this iterator, which likely has a nontrivial drop implementation that returns
    /// a buffer to the iterator's buffer pool.
    type Item;

    /// Move the iterator one position forwards, and return the entry at that position.
    /// Returns `None` if the iterator was at the last entry.
    ///
    /// May need to wait for a buffer to become available.
    ///
    /// # Potential Panics or Deadlocks
    /// If `self.buffer_pool_size() == 0`, then this method is permitted to panic or deadlock.
    /// This method may also panic or cause a deadlock if no buffers are currently available, and
    /// the current thread needs to make progress in order to release a buffer.
    ///
    /// If it is possible for a different thread to make progress and make a buffer available,
    /// this method should not panic or deadlock.
    fn next(&mut self) -> Option<Self::Item>;

    /// If a buffer is available, move the iterator one position forwards, and return the entry at
    /// that position. Returns `Ok(None)` if the iterator was at the last entry.
    ///
    /// # Errors
    /// Returns an error if no buffers were available.
    fn try_next(&mut self) -> Result<Option<Self::Item>, OutOfBuffers>;

    /// Get the total number of buffers in the buffer pool, including buffers that are
    /// currently in use.
    #[must_use]
    fn buffer_pool_size(&self) -> usize;

    /// Get the number of buffers that are currently available.
    ///
    /// In multithreaded scenarios, the returned value could change at any time.
    #[must_use]
    fn available_buffers(&self) -> usize;

    /// Convert the `PooledIterator` into a [`lender::Lender`] lending iterator which only uses
    /// one buffer at a time.
    ///
    /// The seekability and access to cursor methods are preserved, though none of the
    /// `Cursor*Iterator` or `Seekable*Iterator` traits are implemented for the adaptor, in order
    /// to avoid conflicts with the `next()` method name.
    ///
    /// # Potential Panics or Deadlocks
    /// If `self.buffer_pool_size() == 0`, then methods of the returned iterator might panic or
    /// deadlock.
    #[cfg(feature = "lender")]
    #[inline]
    #[must_use]
    fn into_lender(self) -> PooledLenderAdapter<Self> where Self: Sized {
        PooledLenderAdapter::new(self)
    }

    /// Convert the `PooledIterator` into a [`lending_iterator::LendingIterator`] which only uses
    /// one buffer at a time.
    ///
    /// The seekability and access to cursor methods are preserved, though none of the
    /// `Cursor*Iterator` or `Seekable*Iterator` traits are implemented for the adaptor, in order
    /// to avoid conflicts with the `next()` method name.
    ///
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
