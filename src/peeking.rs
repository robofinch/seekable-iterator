use crate::lending_iterator_support::LentItem;
use crate::cursor::{CursorIterator, CursorLendingIterator, CursorPooledIterator};


/// Extend a `CursorIterator` with the ability to peek at the next element and choose based
/// on that element whether to move the iterator's position.
pub trait PeekNext: CursorIterator {
    /// Peek at the next element of the collection (or `None` if the iterator is at the last
    /// entry), and decide based on that element whether to move the iterator's position forward
    /// one element.
    ///
    /// If the callback returns `true`, the iterator's position is changed and the peeked value
    /// is returned wrapped in `Some`. If the callback returns `false`, the iterator's position
    /// remains unchanged and `None` is returned.
    fn next_if<F>(&mut self, f: F) -> Option<Option<Self::Item>>
    where
        F: FnOnce(Option<&Self::Item>) -> bool;

    /// Peek at the next element of the collection, and decide based on that element whether to
    /// move the iterator's position forward one element. If the iterator is at the last entry
    /// -- in other words, if the advanced iterator would not be [valid] --
    /// the iterator is not advanced and `None` is returned.
    ///
    /// If the callback returns `true`, the iterator's position is changed and the peeked value
    /// is returned wrapped in `Some`. Otherwise, the iterator's position remains unchanged
    /// and `None` is returned.
    ///
    /// [valid]: CursorIterator::valid
    #[inline]
    fn next_if_valid_and<F>(&mut self, f: F) -> Option<Self::Item>
    where
        F: FnOnce(&Self::Item) -> bool
    {
        // This default implementation is not necessarily the most efficient.

        let mut return_some = false;
        let maybe_peeked_value = self.next_if(|maybe_item| {
            if let Some(next_item) = maybe_item {
                return_some = f(next_item);
                return_some
            } else {
                false
            }
        });

        if return_some {
            #[expect(
                clippy::expect_used,
                reason = "if `self.next_if` is implemented correctly, this doesn't panic",
            )]
            let next_item = maybe_peeked_value
                .expect("next_if's callback returned true, so it should return the peeked value")
                .expect("the peeked value, which should be returned from next_if, was Some");
            Some(next_item)
        } else {
            None
        }
    }
}

/// Extend a `CursorIterator` with the ability to peek at the previous element and choose based
/// on that element whether to move the iterator's position.
pub trait PeekPrev: CursorIterator {
    /// Peek at the previous element of the collection (or `None` if the iterator is at the first
    /// entry), and decide based on that element whether to move the iterator's position backward
    /// one element.
    ///
    /// If the callback returns `true`, the iterator's position is changed and the peeked value
    /// is returned wrapped in `Some`. If the callback returns `false`, the iterator's position
    /// remains unchanged and `None` is returned.
    ///
    /// Some implementations may have worse performance for backwards iteration than forwards
    /// iteration, so prefer to not use `prev_if`.
    fn prev_if<F>(&mut self, f: F) -> Option<Option<Self::Item>>
    where
        F: FnOnce(Option<&Self::Item>) -> bool;

    /// Peek at the previous element of the collection, and decide based on that element whether to
    /// move the iterator's position forward one element. If the iterator is at the last entry
    /// -- in other words, if the advanced iterator would not be [valid] --
    /// the iterator is not advanced and `None` is returned.
    ///
    /// If the callback returns `true`, the iterator's position is changed and the peeked value
    /// is returned wrapped in `Some`. Otherwise, the iterator's position remains unchanged
    /// and `None` is returned.
    ///
    /// Some implementations may have worse performance for backwards iteration than forwards
    /// iteration, so prefer to not use `prev_if_valid_and`.
    ///
    /// [valid]: CursorIterator::valid
    #[inline]
    fn prev_if_valid_and<F>(&mut self, f: F) -> Option<Self::Item>
    where
        F: FnOnce(&Self::Item) -> bool
    {
        // This default implementation is not necessarily the most efficient.

        let mut return_some = false;
        let maybe_peeked_value = self.prev_if(|maybe_item| {
            if let Some(next_item) = maybe_item {
                return_some = f(next_item);
                return_some
            } else {
                false
            }
        });

        if return_some {
            #[expect(
                clippy::expect_used,
                reason = "if `self.prev_if` is implemented correctly, this doesn't panic",
            )]
            let next_item = maybe_peeked_value
                .expect("prev_if's callback returned true, so it should return the peeked value")
                .expect("the peeked value, which should be returned from prev_if, was Some");
            Some(next_item)
        } else {
            None
        }
    }
}

/// Extend a `CursorLendingIterator` with the ability to peek at the next element and choose based
/// on that element whether to move the iterator's position.
pub trait PeekNextLend: CursorLendingIterator {
    /// Peek at the next element of the collection (or `None` if the iterator is at the last
    /// entry), and decide based on that element whether to move the iterator's position forward
    /// one element.
    ///
    /// If the callback returns `true`, the iterator's position is changed and the peeked value
    /// is returned wrapped in `Some`. If the callback returns `false`, the iterator's position
    /// remains unchanged and `None` is returned.
    fn next_if<F>(&mut self, f: F) -> Option<Option<LentItem<'_, Self>>>
    where
        F: FnOnce(Option<LentItem<'_, Self>>) -> bool;

    /// Peek at the next element of the collection, and decide based on that element whether to
    /// move the iterator's position forward one element. If the iterator is at the last entry
    /// -- in other words, if the advanced iterator would not be [valid] --
    /// the iterator is not advanced and `None` is returned.
    ///
    /// If the callback returns `true`, the iterator's position is changed and the peeked value
    /// is returned wrapped in `Some`. Otherwise, the iterator's position remains unchanged
    /// and `None` is returned.
    ///
    /// [valid]: CursorLendingIterator::valid
    #[inline]
    fn next_if_valid_and<F>(&mut self, f: F) -> Option<LentItem<'_, Self>>
    where
        F: FnOnce(LentItem<'_, Self>) -> bool
    {
        // This default implementation is not necessarily the most efficient.

        let mut return_some = false;
        let maybe_peeked_value = self.next_if(|maybe_item| {
            if let Some(next_item) = maybe_item {
                return_some = f(next_item);
                return_some
            } else {
                false
            }
        });

        if return_some {
            #[expect(
                clippy::expect_used,
                reason = "if `self.next_if` is implemented correctly, this doesn't panic",
            )]
            let next_item = maybe_peeked_value
                .expect("next_if's callback returned true, so it should return the peeked value")
                .expect("the peeked value, which should be returned from next_if, was Some");
            Some(next_item)
        } else {
            None
        }
    }
}

/// Extend a `CursorLendingIterator` with the ability to peek at the previous element and choose
/// based on that element whether to move the iterator's position.
pub trait PeekPrevLend: CursorLendingIterator {
    /// Peek at the previous element of the collection (or `None` if the iterator is at the first
    /// entry), and decide based on that element whether to move the iterator's position backward
    /// one element.
    ///
    /// If the callback returns `true`, the iterator's position is changed and the peeked value
    /// is returned wrapped in `Some`. If the callback returns `false`, the iterator's position
    /// remains unchanged and `None` is returned.
    ///
    /// Some implementations may have worse performance for backwards iteration than forwards
    /// iteration, so prefer to not use `prev_if`.
    fn prev_if<F>(&mut self, f: F) -> Option<Option<LentItem<'_, Self>>>
    where
        F: FnOnce(Option<LentItem<'_, Self>>) -> bool;

    /// Peek at the previous element of the collection, and decide based on that element whether to
    /// move the iterator's position forward one element. If the iterator is at the last entry
    /// -- in other words, if the advanced iterator would not be [valid] --
    /// the iterator is not advanced and `None` is returned.
    ///
    /// If the callback returns `true`, the iterator's position is changed and the peeked value
    /// is returned wrapped in `Some`. Otherwise, the iterator's position remains unchanged
    /// and `None` is returned.
    ///
    /// Some implementations may have worse performance for backwards iteration than forwards
    /// iteration, so prefer to not use `prev_if_valid_and`.
    ///
    /// [valid]: CursorLendingIterator::valid
    #[inline]
    fn prev_if_valid_and<F>(&mut self, f: F) -> Option<LentItem<'_, Self>>
    where
        F: FnOnce(LentItem<'_, Self>) -> bool
    {
        // This default implementation is not necessarily the most efficient.

        let mut return_some = false;
        let maybe_peeked_value = self.prev_if(|maybe_item| {
            if let Some(next_item) = maybe_item {
                return_some = f(next_item);
                return_some
            } else {
                false
            }
        });

        if return_some {
            #[expect(
                clippy::expect_used,
                reason = "if `self.prev_if` is implemented correctly, this doesn't panic",
            )]
            let next_item = maybe_peeked_value
                .expect("prev_if's callback returned true, so it should return the peeked value")
                .expect("the peeked value, which should be returned from prev_if, was Some");
            Some(next_item)
        } else {
            None
        }
    }
}

/// Extend a `CursorPooledIterator` with the ability to peek at the next element and choose based
/// on that element whether to move the iterator's position.
pub trait PeekNextPooled: CursorPooledIterator {
    /// Peek at the next element of the collection (or `None` if the iterator is at the last
    /// entry), and decide based on that element whether to move the iterator's position forward
    /// one element.
    ///
    /// If the callback returns `true`, the iterator's position is changed and the peeked value
    /// is returned wrapped in `Some`. If the callback returns `false`, the iterator's position
    /// remains unchanged and `None` is returned.
    fn next_if<F>(&mut self, f: F) -> Option<Option<Self::Item>>
    where
        F: FnOnce(Option<&Self::Item>) -> bool;

    /// Peek at the next element of the collection, and decide based on that element whether to
    /// move the iterator's position forward one element. If the iterator is at the last entry
    /// -- in other words, if the advanced iterator would not be [valid] --
    /// the iterator is not advanced and `None` is returned.
    ///
    /// If the callback returns `true`, the iterator's position is changed and the peeked value
    /// is returned wrapped in `Some`. Otherwise, the iterator's position remains unchanged
    /// and `None` is returned.
    ///
    /// [valid]: CursorPooledIterator::valid
    #[inline]
    fn next_if_valid_and<F>(&mut self, f: F) -> Option<Self::Item>
    where
        F: FnOnce(&Self::Item) -> bool
    {
        // This default implementation is not necessarily the most efficient.

        let mut return_some = false;
        let maybe_peeked_value = self.next_if(|maybe_item| {
            if let Some(next_item) = maybe_item {
                return_some = f(next_item);
                return_some
            } else {
                false
            }
        });

        if return_some {
            #[expect(
                clippy::expect_used,
                reason = "if `self.next_if` is implemented correctly, this doesn't panic",
            )]
            let next_item = maybe_peeked_value
                .expect("next_if's callback returned true, so it should return the peeked value")
                .expect("the peeked value, which should be returned from next_if, was Some");
            Some(next_item)
        } else {
            None
        }
    }
}

/// Extend a `CursorPooledIterator` with the ability to peek at the previous element and choose
/// based on that element whether to move the iterator's position.
pub trait PeekPrevPooled: CursorPooledIterator {
    /// Peek at the previous element of the collection (or `None` if the iterator is at the first
    /// entry), and decide based on that element whether to move the iterator's position backward
    /// one element.
    ///
    /// If the callback returns `true`, the iterator's position is changed and the peeked value
    /// is returned wrapped in `Some`. If the callback returns `false`, the iterator's position
    /// remains unchanged and `None` is returned.
    ///
    /// Some implementations may have worse performance for backwards iteration than forwards
    /// iteration, so prefer to not use `prev_if`.
    fn prev_if<F>(&mut self, f: F) -> Option<Option<Self::Item>>
    where
        F: FnOnce(Option<&Self::Item>) -> bool;

    /// Peek at the previous element of the collection, and decide based on that element whether to
    /// move the iterator's position forward one element. If the iterator is at the last entry
    /// -- in other words, if the advanced iterator would not be [valid] --
    /// the iterator is not advanced and `None` is returned.
    ///
    /// If the callback returns `true`, the iterator's position is changed and the peeked value
    /// is returned wrapped in `Some`. Otherwise, the iterator's position remains unchanged
    /// and `None` is returned.
    ///
    /// Some implementations may have worse performance for backwards iteration than forwards
    /// iteration, so prefer to not use `prev_if_valid_and`.
    ///
    /// [valid]: CursorPooledIterator::valid
    #[inline]
    fn prev_if_valid_and<F>(&mut self, f: F) -> Option<Self::Item>
    where
        F: FnOnce(&Self::Item) -> bool
    {
        // This default implementation is not necessarily the most efficient.

        let mut return_some = false;
        let maybe_peeked_value = self.prev_if(|maybe_item| {
            if let Some(next_item) = maybe_item {
                return_some = f(next_item);
                return_some
            } else {
                false
            }
        });

        if return_some {
            #[expect(
                clippy::expect_used,
                reason = "if `self.prev_if` is implemented correctly, this doesn't panic",
            )]
            let next_item = maybe_peeked_value
                .expect("prev_if's callback returned true, so it should return the peeked value")
                .expect("the peeked value, which should be returned from prev_if, was Some");
            Some(next_item)
        } else {
            None
        }
    }
}
