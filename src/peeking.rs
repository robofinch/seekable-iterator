use crate::lending_iterator_support::LentItem;
use crate::cursor::{CursorIterator, CursorLendingIterator, CursorPooledIterator};


/// Extend a `CursorIterator` with the ability to peek at the next element.
pub trait PeekNext: CursorIterator {
    /// Peek at the next element of the collection (or `None` if the iterator is at the last
    /// entry).
    ///
    /// After any number of additional calls to `self.peek_next()` and non-mutating methods
    /// like `self.current()` and `self.valid()`, `self.next()` will return this peeked element.
    ///
    /// Calling any other methods, including `self.peek_prev()`, may break that guarantee;
    /// this is relevant in particular to iterators over concurrent collections.
    #[must_use]
    fn peek_next(&mut self) -> Option<Self::Item>;
}

/// Extend a `CursorIterator` with the ability to peek at the previous element.
pub trait PeekPrev: CursorIterator {
    /// Peek at the previous element of the collection (or `None` if the iterator is at the first
    /// entry).
    ///
    /// After any number of additional calls to `self.peek_prev()` and non-mutating methods
    /// like `self.current()` and `self.valid()`, `self.prev()` will return this peeked element.
    ///
    /// Calling any other methods, including `self.peek_next()`, may break that guarantee;
    /// this is relevant in particular to iterators over concurrent collections.
    ///
    /// Some implementations may have worse performance for backwards iteration than forwards
    /// iteration, so prefer to not use `peek_prev`.
    #[must_use]
    fn peek_prev(&mut self) -> Option<Self::Item>;
}

/// Extend a `CursorLendingIterator` with the ability to peek at the next element.
pub trait PeekNextLend: CursorLendingIterator {
    /// Peek at the next element of the collection (or `None` if the iterator is at the last
    /// entry).
    ///
    /// After any number of additional calls to `self.peek_next()` and non-mutating methods
    /// like `self.current()` and `self.valid()`, `self.next()` will return this peeked element.
    ///
    /// Calling any other methods, including `self.peek_prev()`, may break that guarantee;
    /// this is relevant in particular to iterators over concurrent collections.
    #[must_use]
    fn peek_next(&mut self) -> Option<LentItem<'_, Self>>;
}

/// Extend a `CursorLendingIterator` with the ability to peek at the previous element.
pub trait PeekPrevLend: CursorLendingIterator {
    /// Peek at the previous element of the collection (or `None` if the iterator is at the first
    /// entry).
    ///
    /// After any number of additional calls to `self.peek_prev()` and non-mutating methods
    /// like `self.current()` and `self.valid()`, `self.prev()` will return this peeked element.
    ///
    /// Calling any other methods, including `self.peek_next()`, may break that guarantee;
    /// this is relevant in particular to iterators over concurrent collections.
    ///
    /// Some implementations may have worse performance for backwards iteration than forwards
    /// iteration, so prefer to not use `peek_prev`.
    #[must_use]
    fn peek_prev(&mut self) -> Option<LentItem<'_, Self>>;
}

/// Extend a `CursorPooledIterator` with the ability to peek at the next element.
pub trait PeekNextPooled: CursorPooledIterator {
    /// Peek at the next element of the collection (or `None` if the iterator is at the last
    /// entry).
    ///
    /// After any number of additional calls to `self.peek_next()` and non-mutating methods
    /// like `self.current()` and `self.valid()`, `self.next()` will return this peeked element.
    ///
    /// Calling any other methods, including `self.peek_prev()`, may break that guarantee;
    /// this is relevant in particular to iterators over concurrent collections.
    #[must_use]
    fn peek_next(&mut self) -> Option<Self::Item>;
}

/// Extend a `CursorPooledIterator` with the ability to peek at the previous element.
pub trait PeekPrevPooled: CursorPooledIterator {
    /// Peek at the previous element of the collection (or `None` if the iterator is at the first
    /// entry).
    ///
    /// After any number of additional calls to `self.peek_prev()` and non-mutating methods
    /// like `self.current()` and `self.valid()`, `self.prev()` will return this peeked element.
    ///
    /// Calling any other methods, including `self.peek_next()`, may break that guarantee;
    /// this is relevant in particular to iterators over concurrent collections.
    ///
    /// Some implementations may have worse performance for backwards iteration than forwards
    /// iteration, so prefer to not use `peek_prev`.
    #[must_use]
    fn peek_prev(&mut self) -> Option<Self::Item>;
}
