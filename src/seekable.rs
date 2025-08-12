use core::borrow::Borrow;

use crate::comparator::Comparator;


pub trait Seekable<Key: ?Sized, Cmp: ?Sized + Comparator<Key>> {
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
