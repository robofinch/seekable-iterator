use crate::{comparator::Comparator, seekable::Seekable};
use crate::cursor::{CursorIterator, CursorLendingIterator, CursorPooledIterator};


/// An [`Iterator`] with cursor methods from [`CursorIterator`] and the ability to seek from
/// [`Seekable`].
///
/// See [`CursorIterator`] for more.
pub trait SeekableIterator<Key, Cmp>
where
    Key: ?Sized,
    Cmp: ?Sized + Comparator<Key>,
    Self: CursorIterator + Seekable<Key, Cmp>,
{}

impl<Key, Cmp, I> SeekableIterator<Key, Cmp> for I
where
    Key: ?Sized,
    Cmp: ?Sized + Comparator<Key>,
    I: CursorIterator + Seekable<Key, Cmp>,
{}

/// A lending iterator with cursor methods from [`CursorLendingIterator`] and the ability to seek
/// from [`Seekable`].
///
/// As a lending iterator, only one entry can be accessed at a time.
///
/// See [`CursorLendingIterator`] for more.
pub trait SeekableLendingIterator<Key, Cmp>
where
    Key: ?Sized,
    Cmp: ?Sized + Comparator<Key>,
    Self: CursorLendingIterator + Seekable<Key, Cmp>,
{}

impl<Key, Cmp, I> SeekableLendingIterator<Key, Cmp> for I
where
    Key: ?Sized,
    Cmp: ?Sized + Comparator<Key>,
    I: CursorLendingIterator + Seekable<Key, Cmp>,
{}

/// A [`PooledIterator`] with cursor methods from [`CursorPooledIterator`] and the ability
/// to seek from [`Seekable`].
///
/// The iterator is similar to a lending iterator (which can lend one item at a time), but can
/// make use of a buffer pool to lend out multiple items at a time.
///
/// See [`CursorPooledIterator`] for more.
///
/// [`PooledIterator`]: crate::pooled::PooledIterator
pub trait SeekablePooledIterator<Key, Cmp>
where
    Key: ?Sized,
    Cmp: ?Sized + Comparator<Key>,
    Self: CursorPooledIterator + Seekable<Key, Cmp>,
{}

impl<Key, Cmp, I> SeekablePooledIterator<Key, Cmp> for I
where
    Key: ?Sized,
    Cmp: ?Sized + Comparator<Key>,
    I: CursorPooledIterator + Seekable<Key, Cmp>,
{}
