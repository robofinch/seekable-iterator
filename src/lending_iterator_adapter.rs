use crate::pooled::PooledIterator;


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
pub struct PooledLendingIteratorAdapter<I: PooledIterator> {
    iter: I,
    item: Option<I::Item>,
}

impl<I: PooledIterator> PooledLendingIteratorAdapter<I> {
    #[inline]
    #[must_use]
    pub(crate) const fn new(iter: I) -> Self {
        Self {
            iter,
            item: None,
        }
    }
}

mod lint_and_glob_scope {
    use lending_iterator::prelude::*;

    use crate::{cursor::CursorLendingIterator, pooled::PooledIterator};
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
    impl<I: PooledIterator> LendingIterator for PooledLendingIteratorAdapter<I> {
        type Item<'next> = &'next I::Item;

        #[inline]
        fn next(&mut self) -> Option<Item<'_, Self>> {
            // Make sure any previous item is dropped
            self.item = None;
            self.item = self.iter.next();
            self.item.as_ref()
        }
    }
}
