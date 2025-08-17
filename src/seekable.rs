use crate::comparator::Comparator;
use crate::lending_iterator_support::{LendItem, LentItem};


/// A trait adding seek functionality to one of the cursor iterator traits.
///
/// See [`CursorIterator`], [`CursorLendingIterator`], or [`CursorPooledIterator`] for more.
///
/// Additionally, the keys of implementors must be sorted by a comparator of the indicated
/// [`Comparator`] type.
///
/// Implementors of `Seekable` and [`CursorLendingIterator`] should strongly consider implementing
/// [`ItemToKey`] as well.
///
/// [`CursorIterator`]: crate::cursor::CursorIterator
/// [`CursorLendingIterator`]: crate::cursor::CursorLendingIterator
/// [`CursorPooledIterator`]: crate::cursor::CursorPooledIterator
pub trait Seekable<Key: ?Sized, Cmp: ?Sized + Comparator<Key>> {
    /// Reset the iterator to its initial position, before the first entry and after the last
    /// entry (if there are any entries in the collection).
    ///
    /// The iterator becomes `!valid()`, and is conceptually one position before the first entry
    /// and one position after the last entry (if there are any entries in the collection).
    fn reset(&mut self);

    /// Move the iterator to the smallest key which is greater or equal than the provided
    /// `min_bound`.
    ///
    /// If there is no such key, the iterator becomes `!valid()`, and is conceptually
    /// one position before the first entry and one position after the last entry (if there are
    /// any entries in the collection).
    fn seek(&mut self, min_bound: &Key);

    /// Move the iterator to the greatest key which is strictly less than the provided
    /// `strict_upper_bound`.
    ///
    /// If there is no such key, the iterator becomes `!valid()`, and is conceptually
    /// one position before the first entry and one position after the last entry (if there are
    /// any entries in the collection).
    ///
    /// Some implementations may have worse performance for `seek_before` than [`seek`].
    ///
    /// [`seek`]: Seekable::seek
    fn seek_before(&mut self, strict_upper_bound: &Key);

    /// Move the iterator to the smallest key in the collection.
    ///
    /// If the collection is empty, the iterator is `!valid()`.
    fn seek_to_first(&mut self);

    /// Move the iterator to the greatest key in the collection.
    ///
    /// If the collection is empty, the iterator is `!valid()`.
    fn seek_to_last(&mut self);
}

/// Convert one of the items of an iterator into a `Key` reference, intended for use with
/// [`Seekable`].
///
/// This conversion is expected to be cheap.
pub trait ItemToKey<Key: ?Sized>: for<'lend> LendItem<'lend> {
    /// Convert one of the items of an iterator into a `Key` reference, intended for use with
    /// [`Seekable`].
    ///
    /// This conversion is expected to be cheap.
    fn item_to_key(item: LentItem<'_, Self>) -> &'_ Key;
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
            fn seek(&mut self, min_bound: &Key) {
                self.$field.seek(min_bound);
            }

            #[inline]
            fn seek_before(&mut self, strict_upper_bound: &Key) {
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
