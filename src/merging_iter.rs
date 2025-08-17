use core::{cmp::Ordering, marker::PhantomData, num::NonZero};
use alloc::vec::Vec;

use crate::comparator::Comparator;
use crate::cursor::CursorLendingIterator;
use crate::lending_iterator_support::{LendItem, LentItem};
use crate::seekable::{ItemToKey, Seekable};
use crate::seekable_iterators::SeekableLendingIterator;


#[derive(Debug, Clone, Copy)]
enum Direction {
    Forwards,
    Backwards,
}

/// A [`MergingIter`] takes several [`SeekableLendingIterator`]s as input, and iterates over the
/// sorted union of their entries.
///
/// The given iterators may have overlap in their keys, and can be provided in any order.
///
/// Conceptually, each [`SeekableLendingIterator`] is a circular iterator over the entries of some
/// sorted collection; this also holds of [`MergingIter`]. The collection corresponding to a
/// [`MergingIter`] is the sorted union (without de-duplication) of its given iterators'
/// collections. However, note that the presence of duplicate keys across different iterators can
/// cause unexpected behavior in certain well-defined scenarios (see below). Ideally, the
/// iterators that are merged should not have duplicate keys.
///
/// # Note on backwards iteration
/// Some [`SeekableLendingIterator`] implementations have better performance for
/// forwards iteration than backwards iteration. `MergingIter` itself otherwise has roughly equal
/// performance in either direction, but has overhead for switching the direction of iteration
/// (see below for more information). Moreover, switching direction does not play well with
/// duplicate keys. Therefore, [`MergingIter::prev`], [`MergingIter::seek_before`], and
/// [`MergingIter::seek_to_last`] (the three methods that use backwards iteration) should be
/// avoided if possible.
///
/// # Warning for duplicate keys
/// If a key is present in multiple iterators, then repeatedly calling `next` or repeatedly
/// calling `prev` will yield all items with that key. That is, as expected, iterating through the
/// entire collection associated with a [`MergingIter`] is possible, and can be done in either the
/// forwards or backwards direction.
///
/// However, switching between `next` and `prev` will return at least one but not necessarily all of
/// the items with the key returned by [`MergingIter::current`] at the time of the switch in
/// direction. Note, though, that no key is _ever_ skipped, regardless of whether duplicate keys
/// are present.
///
/// To be precise, the items with duplicate keys may be skipped whenever the `MergingIter` changes
/// which "direction" (forwards or backwards) that it is iterating in. When switching direction,
/// some of the items whose keys compare equal to [`MergingIter::current`] may be skipped over.
///
/// The following methods need to switch direction if necessary, and iterate in a certain direction:
/// - Forwards:
///   - [`MergingIter::next`]
/// - Backwards:
///   - [`MergingIter::prev`]
///
/// The following methods are not impacted by the direction, but set the direction:
/// - Set direction to forwards:
///   - [`MergingIter::new`]
///   - [`MergingIter::reset`]
///   - [`MergingIter::seek`]
///   - [`MergingIter::seek_to_first`]
/// - Set direction to backwards:
///   - [`MergingIter::seek_before`]
///   - [`MergingIter::seek_to_last`]
///
/// The following methods do not impact and are not impacted by the direction:
/// - [`MergingIter::valid`]
/// - [`MergingIter::current`]
#[derive(Debug)]
pub struct MergingIter<Key: ?Sized, Cmp, Iter> {
    iterators:    Vec<Iter>,
    cmp:          Cmp,
    /// Ensures that the implementation of the iterator and comparator aren't switched
    /// mid-iteration by a pathological user
    _key:         PhantomData<Key>,
    /// If `Some`, the value should be 1 more than the index of the current iterator.
    ///
    /// Additionally, an invariant is: after calling any public method of `Self` (notably
    /// including `CursorLendingIterator` and `Seekable` methods), either `self.current_iter`
    /// is `None`, or the iterator it refers to is `valid()`.
    ///
    /// In the former case, no iterator in `self.iterators` should be `valid()`.
    current_iter: Option<NonZero<usize>>,
    /// If `current_iter` is `Some` and `direction` is `Forwards`, then the non-`current_iter`
    /// iterators are non-strictly in front of `current_iter`. If `Backwards`, the
    /// non-`current_iter` iterators are non-strictly behind `current_iter`.
    ///
    /// (Non-strictly is specified to clarify behavior for duplicate keys.)
    direction:    Direction,
}

impl<Key, Cmp, Iter> MergingIter<Key, Cmp, Iter>
where
    Key:  ?Sized,
    Cmp:  Comparator<Key>,
    Iter: SeekableLendingIterator<Key, Cmp> + ItemToKey<Key>,
{
    /// Create a new [`MergingIter`]. See the type-level documentation for details on behavior.
    ///
    /// # Comparator requirements
    /// The [`Comparator`]s used by each of the provided iterators must all behave identically
    /// to each other and to the provided `cmp` value. In particular, this requirement is met
    /// if the `Cmp` generic is a ZST, or if all the comparators were cloned from some common
    /// source.
    ///
    /// # Panics
    /// Panics if the length of `iterators` is `usize::MAX`. Any other number of iterators
    /// can, theoretically, be merged.
    #[inline]
    #[must_use]
    pub fn new(iterators: Vec<Iter>, cmp: Cmp) -> Self {
        assert_ne!(
            iterators.len(),
            usize::MAX,
            "Cannot create a MergingIter over `usize::MAX`-many iterators",
        );

        Self {
            iterators,
            cmp,
            _key:         PhantomData,
            current_iter: None,
            direction:    Direction::Forwards,
        }
    }
}

impl<Key, Cmp, Iter> MergingIter<Key, Cmp, Iter>
where
    Key:  ?Sized,
    Cmp:  Comparator<Key>,
    Iter: SeekableLendingIterator<Key, Cmp> + ItemToKey<Key>,
{
    #[must_use]
    fn get_current_iter_ref(&self) -> Option<&Iter> {
        let current_idx = self.current_iter?.get() - 1;

        #[expect(
            clippy::indexing_slicing,
            reason = "`self.iterators` is never truncated, \
                      and `self.current_idx` is always a valid idx if `Some`",
        )]
        Some(&self.iterators[current_idx])
    }

    /// Set `self.current_iter` to the iterator with the smallest `current` key, among the
    /// iterators in `self.iterators` which are valid.
    fn find_smallest_iter(&mut self) {
        let mut smallest: Option<(usize, &Key)> = None;

        for (idx, iter) in self.iterators.iter().enumerate() {
            if let Some(curr_item) = iter.current() {
                let curr_key = Iter::item_to_key(curr_item);
                if let Some((_, smallest_key)) = smallest {
                    if self.cmp.cmp(curr_key, smallest_key) == Ordering::Less {
                        // `curr_key` is smaller than the previous `smallest`'s key
                        smallest = Some((idx, curr_key));
                    }
                } else {
                    // de-facto `smallest`, nothing was previously found
                    smallest = Some((idx, curr_key));
                }
            } else {
                // The iterator was `!valid()`, so continue.
            }
        }

        #[expect(clippy::unwrap_used, reason = "MergingIter cannot have `usize::MAX` iterators")]
        {
            self.current_iter = smallest.map(|(idx, _)| NonZero::new(idx + 1).unwrap());
        }
    }

    /// Set `self.current_iter` to the iterator with the largest `current` key, among the
    /// iterators in `self.iterators` which are valid.
    fn find_largest_iter(&mut self) {
        let mut largest: Option<(usize, &Key)> = None;

        for (idx, iter) in self.iterators.iter().enumerate().rev() {
            if let Some(curr_item) = iter.current() {
                let curr_key = Iter::item_to_key(curr_item);
                if let Some((_, largest_key)) = largest {
                    if self.cmp.cmp(curr_key, largest_key) == Ordering::Greater {
                        // `curr_key` is smaller than the previous `largest`'s key
                        largest = Some((idx, curr_key));
                    }
                } else {
                    // de-facto `largest`, nothing was previously found
                    largest = Some((idx, curr_key));
                }
            } else {
                // The iterator was `!valid()`, so continue.
            }
        }

        #[expect(clippy::unwrap_used, reason = "MergingIter cannot have `usize::MAX` iterators")]
        {
            self.current_iter = largest.map(|(idx, _)| NonZero::new(idx + 1).unwrap());
        }
    }

    /// For use in `self.next()`, and nothing else.
    ///
    /// Move all non-`current_iter` iterators one entry strictly in front of `current_iter`.
    fn switch_to_forwards(&mut self, current_idx: NonZero<usize>) -> &mut Iter {
        let current_idx = current_idx.get() - 1;

        // Do a little game to satisfy borrowck and aliasing rules
        let (iters, current_and_later) = self.iterators.split_at_mut(current_idx);
        let (current_iter, other_iters) = current_and_later.split_at_mut(1);
        #[expect(clippy::indexing_slicing, reason = "`current_idx` is a valid index")]
        let current_iter = &mut current_iter[0];
        #[expect(
            clippy::unwrap_used,
            reason = "the current iterator is `valid()` as an invariant",
        )]
        let current_key = Iter::item_to_key(current_iter.current().unwrap());

        for iter in iters {
            iter.seek(current_key);

            // `seek` provides a `geq` order, we want a strict greater-than order.
            if iter.current().is_some_and(|item| {
                self.cmp.cmp(current_key, Iter::item_to_key(item)) == Ordering::Equal
            }) {
                iter.next();
            }
        }

        for iter in other_iters {
            iter.seek(current_key);

            if iter.current().is_some_and(|item| {
                self.cmp.cmp(current_key, Iter::item_to_key(item)) == Ordering::Equal
            }) {
                iter.next();
            }
        }

        self.direction = Direction::Forwards;

        current_iter
    }

    /// For use in `self.prev()`, and nothing else.
    ///
    /// Move all non-`current_iter` iterators one entry strictly behind `current_iter`.
    fn switch_to_backwards(&mut self, current_idx: NonZero<usize>) -> &mut Iter {
        let current_idx = current_idx.get() - 1;

        // Do a little game to satisfy borrowck and aliasing rules
        let (iters, current_and_later) = self.iterators.split_at_mut(current_idx);
        let (current_iter, other_iters) = current_and_later.split_at_mut(1);
        #[expect(clippy::indexing_slicing, reason = "`current_idx` is a valid index")]
        let current_iter = &mut current_iter[0];
        #[expect(
            clippy::unwrap_used,
            reason = "the current iterator is `valid()` as an invariant",
        )]
        let current_key = Iter::item_to_key(current_iter.current().unwrap());

        for iter in iters {
            iter.seek_before(current_key);
        }
        for iter in other_iters {
            iter.seek_before(current_key);
        }

        self.direction = Direction::Backwards;

        current_iter
    }
}

impl<'lend, Key, Cmp, Iter> LendItem<'lend> for MergingIter<Key, Cmp, Iter>
where
    Key: ?Sized,
    Iter: LendItem<'lend>,
{
    type Item = Iter::Item;
}

impl<Key, Cmp, Iter> CursorLendingIterator for MergingIter<Key, Cmp, Iter>
where
    Key:  ?Sized,
    Cmp:  Comparator<Key>,
    Iter: SeekableLendingIterator<Key, Cmp> + ItemToKey<Key>,
{
    #[inline]
    fn valid(&self) -> bool {
        self.current_iter.is_some()
    }

    fn next(&mut self) -> Option<LentItem<'_, Self>> {
        if let Some(current_idx) = self.current_iter {
            let current_iter = if matches!(self.direction, Direction::Backwards) {
                self.switch_to_forwards(current_idx)
            } else {
                #[expect(clippy::indexing_slicing, reason = "we know that it's a valid index")]
                &mut self.iterators[current_idx.get() - 1]
            };

            // Before this call, `current_iter` is the (non-strictly) smallest iter.
            // Move it forwards...
            current_iter.next();
            // And find the new smallest iter.
            self.find_smallest_iter();

        } else {
            // In this branch, we're `!valid()`. This means that _every_ iterator is currently
            // `!valid()`.
            // Move every iterator forwards one, and find the smallest.
            for iter in &mut self.iterators {
                iter.next();
            }

            self.find_smallest_iter();
            self.direction = Direction::Forwards;
        }

        self.current()
    }

    #[inline]
    fn current(&self) -> Option<LentItem<'_, Self>> {
        self.get_current_iter_ref()?.current()
    }

    /// Move the iterator one position back, and return the entry at that position.
    /// Returns `None` if the iterator was at the first entry.
    ///
    /// The inner `Iter` iterators may have worse performance for backwards iteration than forwards
    /// iteration, so prefer to not use `prev`. Additionally, [`MergingIter`] has overhead
    /// for switching between backwards and forwards iteration; check the type-level documentation
    /// if you wish to use `prev`.
    fn prev(&mut self) -> Option<LentItem<'_, Self>> {
        if let Some(current_idx) = self.current_iter {
            let current_iter = if matches!(self.direction, Direction::Forwards) {
                self.switch_to_backwards(current_idx)
            } else {
                #[expect(clippy::indexing_slicing, reason = "we know that it's a valid index")]
                &mut self.iterators[current_idx.get() - 1]
            };

            // Before this call, `current_iter` is the largest iter. Move it backwards...
            current_iter.prev();
            // And find the new largest iter.
            self.find_largest_iter();

        } else {
            // In this branch, we're `!valid()`. This means that _every_ iterator is currently
            // `!valid()`.
            // Move every iterator backwards one, and find the largest.
            for iter in &mut self.iterators {
                iter.prev();
            }

            self.find_largest_iter();
            self.direction = Direction::Backwards;
        }

        self.current()
    }
}

impl<Key, Cmp, Iter> Seekable<Key, Cmp> for MergingIter<Key, Cmp, Iter>
where
    Key:  ?Sized,
    Cmp:  Comparator<Key>,
    Iter: SeekableLendingIterator<Key, Cmp> + ItemToKey<Key>,
{
    fn reset(&mut self) {
        for iter in &mut self.iterators {
            iter.reset();
        }
        self.current_iter = None;
        self.direction = Direction::Forwards;
    }

    fn seek(&mut self, min_bound: &Key) {
        for iter in &mut self.iterators {
            iter.seek(min_bound);
        }

        self.find_smallest_iter();
        self.direction = Direction::Forwards;
    }

    /// Move the iterator to the greatest key which is strictly less than the provided
    /// `strict_upper_bound`.
    ///
    /// If there is no such key, the iterator becomes `!valid()`, and is conceptually
    /// one position before the first entry and one position after the last entry (if there are
    /// any entries in the collection).
    ///
    /// The inner `Iter` iterators may have worse performance for `seek_before` than [`seek`].
    /// Additionally, [`MergingIter`] has overhead for switching between backwards and forwards
    /// iteration; check the type-level documentation if you wish to use `seek_before`.
    ///
    /// [`seek`]: MergingIter::seek
    fn seek_before(&mut self, strict_upper_bound: &Key) {
        for iter in &mut self.iterators {
            iter.seek_before(strict_upper_bound);
        }

        self.find_largest_iter();
        self.direction = Direction::Backwards;
    }

    fn seek_to_first(&mut self) {
        for iter in &mut self.iterators {
            iter.seek_to_first();
        }

        self.find_smallest_iter();
        self.direction = Direction::Forwards;
    }

    /// Move the iterator to the greatest key in the collection.
    ///
    /// If the collection is empty, the iterator is `!valid()`.
    ///
    /// [`MergingIter`] has overhead for switching between backwards and forwards
    /// iteration; check the type-level documentation if you wish to use `seek_before`.
    fn seek_to_last(&mut self) {
        for iter in &mut self.iterators {
            iter.seek_to_last();
        }

        self.find_largest_iter();
        self.direction = Direction::Backwards;
    }
}
