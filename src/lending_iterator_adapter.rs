use lending_iterator::lending_iterator::Item;

use crate::seekable::delegate_seekable;
use crate::{comparator::Comparator, pooled::PooledIterator, seekable::Seekable};
use crate::cursor::{CursorLendingIterator, CursorPooledIterator};


/// An adapter for [`CursorLendingIterator`] which implements [`lending_iterator::LendingIterator`].
///
/// To avoid conflicts between `LendingIterator::next` and `CursorLendingIterator::next`,
/// the `CursorLendingIterator` is not implemented for the adapter; however,
/// the other cursor methods (`valid`, `current`, `prev`) are implemented, and [`Seekable`]
/// is implemented if `I: Seekable`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LendingIteratorAdapter<I>(I);

impl<I> LendingIteratorAdapter<I> {
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

impl<I: CursorLendingIterator> LendingIteratorAdapter<I> {
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
    pub fn current(&self) -> Option<Item<'_, Self>> {
        self.0.current()
    }

    /// Move the iterator one position back, and return the entry at that position.
    /// Returns `None` if the iterator was at the first entry.
    ///
    /// See [`CursorLendingIterator::prev()`].
    #[inline]
    #[must_use]
    pub fn prev(&mut self) -> Option<Item<'_, Self>> {
        self.0.prev()
    }
}

delegate_seekable!(LendingIteratorAdapter.0);

/// An adapter for [`PooledIterator`] which implements [`lending_iterator::LendingIterator`].
///
/// To avoid conflicts between `LendingIterator::next` and `PooledIterator::next`,
/// the `PooledIterator` is not implemented for the adapter; however, the other cursor methods
/// (`valid`, `current`, `prev`) are implemented if `I: CursorPooledIterator`, and [`Seekable`]
/// is implemented if `I: Seekable`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PooledLendingIteratorAdapter<I: PooledIterator> {
    iter: I,
    item: Option<I::Item>,
}

impl<I: PooledIterator> PooledLendingIteratorAdapter<I> {
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

impl<I: CursorPooledIterator> PooledLendingIteratorAdapter<I> {
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
    pub const fn current(&self) -> Option<Item<'_, Self>> {
        self.item.as_ref()
    }

    /// Move the iterator one position back, and return the entry at that position.
    /// Returns `None` if the iterator was at the first entry.
    ///
    /// See [`CursorPooledIterator::prev()`].
    #[inline]
    #[must_use]
    pub fn prev(&mut self) -> Option<Item<'_, Self>> {
        // Make sure any previous item is dropped
        self.item = None;
        self.item = self.iter.prev();
        self.item.as_ref()
    }
}

delegate_seekable!(PooledLendingIteratorAdapter.iter PooledIterator);

mod lint_and_glob_scope {
    use lending_iterator::prelude::*;

    use crate::{cursor::CursorLendingIterator, pooled::PooledIterator};
    use super::{LendingIteratorAdapter, PooledLendingIteratorAdapter};


    #[gat]
    impl<I: CursorLendingIterator> LendingIterator for LendingIteratorAdapter<I> {
        type Item<'next> = I::Item<'next>;

        #[inline]
        fn next(&mut self) -> Option<Item<'_, Self>> {
            self.0.next()
        }
    }

    #[gat]
    impl<I: PooledIterator> LendingIterator for PooledLendingIteratorAdapter<I> {
        type Item<'next> = &'next I::Item;

        #[inline]
        fn next(&mut self) -> Option<Item<'_, Self>> {
            // Make sure any previous item is dropped
            self.item = None;
            self.item = self.iter.next();
            self.item.as_ref()
        }
    }
}
