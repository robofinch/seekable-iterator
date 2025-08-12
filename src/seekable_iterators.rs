use crate::{comparator::Comparator, seekable::Seekable};
use crate::cursor::{CursorIterator, CursorLendingIterator, CursorPooledIterator};


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
