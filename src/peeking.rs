use crate::{CursorIterator, CursorLendingIterator, CursorPooledIterator, LentItem};


/// Extend a `CursorIterator` with the ability to peek at the next element and choose based
/// on that element whether to move the iterator's position.
pub trait PeekNext: CursorIterator {
    /// Peek at the next element of the collection, and decide based on that element whether
    /// to move the iterator's position forward one element.
    ///
    /// The iterator's position is changed if the callback returns `true`, and remains unchanged
    /// if the callback returns `false`.
    ///
    /// The callback is provided with `None` if the iterator is at the last entry.
    fn peek_next_and_commit_if<F>(&mut self, f: F) where F: Fn(Option<&Self::Item>) -> bool;
}

/// Extend a `CursorIterator` with the ability to peek at the previous element and choose based
/// on that element whether to move the iterator's position.
pub trait PeekPrev: CursorIterator {
    /// Peek at the previous element of the collection, and decide based on that element whether
    /// to move the iterator's position back one element.
    ///
    /// The iterator's position is changed if the callback returns `true`, and remains unchanged
    /// if the callback returns `false`.
    ///
    /// The callback is provided with `None` if the iterator is at the first entry.
    ///
    /// Some implementations may have worse performance for backwards iteration than forwards
    /// iteration, so prefer to not use `peek_prev_and_commit_if`.
    fn peek_prev_and_commit_if<F>(&mut self, f: F) where F: Fn(Option<&Self::Item>) -> bool;
}

/// Extend a `CursorLendingIterator` with the ability to peek at the next element and choose based
/// on that element whether to move the iterator's position.
pub trait PeekNextLend: CursorLendingIterator {
    /// Peek at the next element of the collection, and decide based on that element whether
    /// to move the iterator's position forward one element.
    ///
    /// The iterator's position is changed if the callback returns `true`, and remains unchanged
    /// if the callback returns `false`.
    ///
    /// The callback is provided with `None` if the iterator is at the last entry.
    fn peek_next_and_commit_if<F>(&mut self, f: F)
    where
        F: Fn(Option<LentItem<'_, Self>>) -> bool;
}

/// Extend a `CursorLendingIterator` with the ability to peek at the previous element and choose
/// based on that element whether to move the iterator's position.
pub trait PeekPrevLend: CursorLendingIterator {
    /// Peek at the previous element of the collection, and decide based on that element whether
    /// to move the iterator's position back one element.
    ///
    /// The iterator's position is changed if the callback returns `true`, and remains unchanged
    /// if the callback returns `false`.
    ///
    /// The callback is provided with `None` if the iterator is at the first entry.
    ///
    /// Some implementations may have worse performance for backwards iteration than forwards
    /// iteration, so prefer to not use `peek_prev_and_commit_if`.
    fn peek_prev_and_commit_if<F>(&mut self, f: F)
    where
        F: for<'a> Fn(Option<LentItem<'a, Self>>) -> bool;
}

/// Extend a `CursorPooledIterator` with the ability to peek at the next element and choose based
/// on that element whether to move the iterator's position.
pub trait PeekNextPooled: CursorPooledIterator {
    /// Peek at the next element of the collection, and decide based on that element whether
    /// to move the iterator's position forward one element.
    ///
    /// The iterator's position is changed if the callback returns `true`, and remains unchanged
    /// if the callback returns `false`.
    ///
    /// The callback is provided with `None` if the iterator is at the last entry.
    fn peek_next_and_commit_if<F>(&mut self, f: F) where F: Fn(Option<&Self::Item>) -> bool;
}

/// Extend a `CursorPooledIterator` with the ability to peek at the previous element and choose
/// based on that element whether to move the iterator's position.
pub trait PeekPrevPooled: CursorPooledIterator {
    /// Peek at the previous element of the collection, and decide based on that element whether
    /// to move the iterator's position back one element.
    ///
    /// The iterator's position is changed if the callback returns `true`, and remains unchanged
    /// if the callback returns `false`.
    ///
    /// The callback is provided with `None` if the iterator is at the first entry.
    ///
    /// Some implementations may have worse performance for backwards iteration than forwards
    /// iteration, so prefer to not use `peek_prev_and_commit_if`.
    fn peek_prev_and_commit_if<F>(&mut self, f: F) where F: Fn(Option<&Self::Item>) -> bool;
}
