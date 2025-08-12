use lender::{Lend, Lender, Lending};

use crate::seekable::delegate_seekable;
use crate::{
    comparator::Comparator, lending_iterator_support::LentItem,
    pooled::PooledIterator, seekable::Seekable,
};
use crate::cursor::{CursorLendingIterator, CursorPooledIterator};


/// An adapter for [`CursorLendingIterator`] which implements [`lender::Lender`].
///
/// To avoid conflicts between `Lender::next` and `CursorLendingIterator::next`,
/// the `CursorLendingIterator` is not implemented for the adapter; however,
/// the other cursor methods (`valid`, `current`, `prev`) are implemented, and [`Seekable`]
/// is implemented if `I: Seekable`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LenderAdapter<I>(I);

impl<I> LenderAdapter<I> {
    #[inline]
    #[must_use]
    pub(crate) const fn new(iter: I) -> Self {
        Self(iter)
    }

    /// Convert the adapter back into the inner iterator.
    #[inline]
    #[must_use]
    pub fn into_inner(self) -> I {
        self.0
    }
}

impl<'lend, I: CursorLendingIterator> Lending<'lend> for LenderAdapter<I> {
    type Lend = LentItem<'lend, I>;
}

impl<I: CursorLendingIterator> Lender for LenderAdapter<I> {
    #[inline]
    fn next(&mut self) -> Option<Lend<'_, Self>> {
        self.0.next()
    }
}

impl<I: CursorLendingIterator> LenderAdapter<I> {
    /// Determine whether the iterator is currently at any value in the collection.
    ///
    /// See [`CursorLendingIterator::valid()`].
    #[inline]
    #[must_use]
    pub fn valid(&self) -> bool {
        self.0.valid()
    }

    /// Get the current value the iterator is at.
    ///
    /// See [`CursorLendingIterator::current()`].
    #[inline]
    #[must_use]
    pub fn current(&self) -> Option<Lend<'_, Self>> {
        self.0.current()
    }

    /// Move the iterator one position back, and return the entry at that position.
    /// Returns `None` if the iterator was at the first entry.
    ///
    /// See [`CursorLendingIterator::prev()`].
    #[inline]
    #[must_use]
    pub fn prev(&mut self) -> Option<Lend<'_, Self>> {
        self.0.prev()
    }
}

delegate_seekable!(LenderAdapter.0);

/// An adapter for [`PooledIterator`] which implements [`lender::Lender`].
///
/// To avoid conflicts between `Lender::next` and `PooledIterator::next`,
/// the `PooledIterator` is not implemented for the adapter; however, the other cursor methods
/// (`valid`, `current`, `prev`) are implemented if `I: CursorPooledIterator`, and [`Seekable`]
/// is implemented if `I: Seekable`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PooledLenderAdapter<I: PooledIterator> {
    iter: I,
    item: Option<I::Item>,
}

impl<I: PooledIterator> PooledLenderAdapter<I> {
    #[inline]
    #[must_use]
    pub(crate) const fn new(iter: I) -> Self {
        Self {
            iter,
            item: None,
        }
    }

    /// Convert the adapter back into the inner iterator.
    #[inline]
    #[must_use]
    pub fn into_inner(self) -> I {
        self.iter
    }
}

impl<'lend, I: PooledIterator> Lending<'lend> for PooledLenderAdapter<I> {
    type Lend = &'lend I::Item;
}

impl<I: PooledIterator> Lender for PooledLenderAdapter<I> {
    #[inline]
    fn next(&mut self) -> Option<Lend<'_, Self>> {
        // Make sure any previous item is dropped
        self.item = None;
        self.item = self.iter.next();
        self.item.as_ref()
    }
}

impl<I: CursorPooledIterator> PooledLenderAdapter<I> {
    /// Determine whether the iterator is currently at any value in the collection.
    ///
    /// See [`CursorPooledIterator::valid()`].
    #[inline]
    #[must_use]
    pub fn valid(&self) -> bool {
        self.iter.valid()
    }

    /// Get the current value the iterator is at.
    ///
    /// This is cheap, and does not require getting a new buffer.
    ///
    /// See [`CursorPooledIterator::current()`].
    #[inline]
    #[must_use]
    pub const fn current(&self) -> Option<Lend<'_, Self>> {
        self.item.as_ref()
    }

    /// Move the iterator one position back, and return the entry at that position.
    /// Returns `None` if the iterator was at the first entry.
    ///
    /// See [`CursorPooledIterator::prev()`].
    #[inline]
    #[must_use]
    pub fn prev(&mut self) -> Option<Lend<'_, Self>> {
        // Make sure any previous item is dropped
        self.item = None;
        self.item = self.iter.prev();
        self.item.as_ref()
    }
}

delegate_seekable!(PooledLenderAdapter.iter PooledIterator);
