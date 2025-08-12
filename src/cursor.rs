use crate::pooled::{PooledIterator, WouldBlock};
#[cfg(feature = "lender")]
use crate::lender_adapter::{LenderAdapter, PooledLenderAdapter};
#[cfg(feature = "lending-iterator")]
use crate::lending_iterator_adapter::{LendingIteratorAdapter, PooledLendingIteratorAdapter};


pub trait CursorIterator: Iterator {
    #[must_use]
    fn valid(&self) -> bool;

    #[must_use]
    fn current(&self) -> Option<Self::Item>;

    fn prev(&mut self) -> Option<Self::Item>;
}

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

pub trait CursorPooledIterator: PooledIterator {
    #[must_use]
    fn valid(&self) -> bool;

    #[must_use]
    fn current(&self) -> Option<Self::Item>;

    fn prev(&mut self) -> Option<Self::Item>;

    /// # Errors
    /// Errors if the operation would have blocked, due to no buffers being available.
    fn try_current(&self) -> Result<Option<Self::Item>, WouldBlock>;

    /// # Errors
    /// Errors if the operation would have blocked, due to no buffers being available.
    fn try_prev(&mut self) -> Result<Option<Self::Item>, WouldBlock>;

    #[cfg(feature = "lender")]
    #[inline]
    #[must_use]
    fn into_lender(self) -> PooledLenderAdapter<Self> where Self: Sized {
        PooledLenderAdapter::new(self)
    }

    #[cfg(feature = "lending-iterator")]
    #[inline]
    #[must_use]
    fn into_lending_iterator(self) -> PooledLendingIteratorAdapter<Self> where Self: Sized {
        PooledLendingIteratorAdapter::new(self)
    }
}
