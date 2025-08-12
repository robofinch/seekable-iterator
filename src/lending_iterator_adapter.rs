use crate::cursor::CursorPooledIterator;


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LendingIteratorAdapter<I>(I);

impl<I> LendingIteratorAdapter<I> {
    #[inline]
    #[must_use]
    pub(crate) const fn new(iter: I) -> Self {
        Self(iter)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PooledLendingIteratorAdapter<I: CursorPooledIterator>(
    Option<(I, Option<I::Item>)>
);

impl<I: CursorPooledIterator> PooledLendingIteratorAdapter<I> {
    #[inline]
    #[must_use]
    pub(crate) fn new(iter: I) -> Self {
        if iter.buffer_pool_size() == 0 {
            Self(None)
        } else {
            Self(Some((iter, None)))
        }
    }
}

mod lint_and_glob_scope {
    use lending_iterator::prelude::*;

    use crate::cursor::{CursorLendingIterator, CursorPooledIterator};
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
    impl<I: CursorPooledIterator> LendingIterator for PooledLendingIteratorAdapter<I> {
        type Item<'next> = &'next I::Item;

        #[inline]
        fn next(&mut self) -> Option<Item<'_, Self>> {
            if let Some((iter, item)) = &mut self.0 {
                // Make sure any previous item is dropped
                *item = None;
                *item = iter.next();
                item.as_ref()
            } else {
                None
            }
        }
    }
}
