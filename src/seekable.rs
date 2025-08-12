use core::borrow::Borrow;

use crate::comparator::Comparator;


/// A trait adding seek functionality to one of the cursor iterator traits.
///
/// See [`CursorIterator`], [`CursorLendingIterator`], or [`CursorPooledIterator`] for more.
///
/// [`CursorIterator`]: crate::cursor::CursorIterator
/// [`CursorLendingIterator`]: crate::cursor::CursorLendingIterator
/// [`CursorPooledIterator`]: crate::cursor::CursorPooledIterator
pub trait Seekable<Key: ?Sized, Cmp: ?Sized + Comparator<Key>> {
    /// Reset the iterator to its initial position, before the first entry and after the last
    /// entry (if there are any entries in the collection).
    ///
    /// The iterator will then not be `!valid()`.
    fn reset(&mut self);

    /// Move the iterator to the smallest key which is greater or equal than the provided
    /// `min_bound`.
    ///
    /// If there is no such key, the iterator becomes `!valid()`, and is conceptually
    /// one position before the first entry and one position after the last entry (if there are
    /// any entries in the collection).
    fn seek<K>(&mut self, min_bound: K) where Key: Borrow<K>;

    /// Move the iterator to the greatest key which is strictly less than the provided
    /// `strict_upper_bound`.
    ///
    /// If there is no such key, the iterator becomes `!valid()`, and is conceptually
    /// one position before the first entry and one position after the last entry (if there are
    /// any entries in the collection).
    fn seek_before<K>(&mut self, strict_upper_bound: K) where Key: Borrow<K>;

    /// Move the iterator to the smallest key in the collection.
    ///
    /// If the collection is empty, the iterator is `!valid()`.
    fn seek_to_first(&mut self);

    /// Move the iterator to the greatest key in the collection.
    ///
    /// If the collection is empty, the iterator is `!valid()`.
    fn seek_to_last(&mut self);
}

#[cfg(any(feature = "lender", feature = "lending-iterator"))]
macro_rules! delegate_seekable {
    ($struct_name:ident.$field:tt $($extra_i_bounds:tt)*) => {
        impl<Key, Cmp, I> Seekable<Key, Cmp> for $struct_name<I>
        where
            Key: ?Sized,
            Cmp: ?Sized + Comparator<Key>,
            I:   Seekable<Key, Cmp> + $($extra_i_bounds)*,
        {
            #[inline]
            fn reset(&mut self) {
                self.$field.reset();
            }

            #[inline]
            fn seek<K>(&mut self, min_bound: K) where Key: Borrow<K> {
                self.$field.seek(min_bound);
            }

            #[inline]
            fn seek_before<K>(&mut self, strict_upper_bound: K) where Key: Borrow<K> {
                self.$field.seek_before(strict_upper_bound);
            }

            #[inline]
            fn seek_to_first(&mut self) {
                self.$field.seek_to_first();
            }

            #[inline]
            fn seek_to_last(&mut self) {
                self.$field.seek_to_last();
            }
        }
    };
}

#[cfg(any(feature = "lender", feature = "lending-iterator"))]
pub(crate) use delegate_seekable;
