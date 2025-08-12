use lender::{Lend, Lender, Lending};

use crate::{cursor::CursorLendingIterator, pooled::PooledIterator};


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LenderAdapter<I>(I);

impl<I> LenderAdapter<I> {
    #[inline]
    #[must_use]
    pub(crate) const fn new(iter: I) -> Self {
        Self(iter)
    }
}

impl<'lend, I: CursorLendingIterator> Lending<'lend> for LenderAdapter<I> {
    type Lend = I::Item<'lend>;
}

impl<I: CursorLendingIterator> Lender for LenderAdapter<I> {
    #[inline]
    fn next(&mut self) -> Option<Lend<'_, Self>> {
        self.0.next()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PooledLenderAdapter<I: PooledIterator> {
    iter: I,
    item: Option<I::Item>,
}

impl<I: PooledIterator> PooledLenderAdapter<I> {
    #[inline]
    #[must_use]
    pub(crate) const fn new(iter: I) -> Self {
        Self {
            iter,
            item: None,
        }
    }
}

impl<'lend, I: PooledIterator> Lending<'lend> for PooledLenderAdapter<I> {
    type Lend = &'lend I::Item;
}

impl<I: PooledIterator> Lender for PooledLenderAdapter<I> {
    #[inline]
    fn next(&mut self) -> Option<Lend<'_, Self>> {
        // Make sure any previous item is dropped
        self.item = None;
        self.item = self.iter.next();
        self.item.as_ref()
    }
}
