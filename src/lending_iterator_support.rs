#[doc(hidden)]
pub trait ImplyBound: sealed::Sealed {}

mod sealed {
    #[expect(unnameable_types, reason = "this is intentional, to create a sealed trait")]
    pub trait Sealed {}
}

impl<T: ?Sized> ImplyBound for &T {}
impl<T: ?Sized> sealed::Sealed for &T {}

/// Trait for getting the item of a lending iterator.
///
/// The lifetime can force the consumer to drop the item before obtaining a new item (which
/// requires a mutable borrow to the iterator, invalidating the borrow of the previous item).
///
/// See [`lender`] for why this strategy is used instead of a simple GAT.
#[cfg_attr(not(feature = "lender"), doc = " [`lender`]: https://docs.rs/lender/0.3/lender")]
#[cfg_attr(not(feature = "lender"), allow(clippy::doc_markdown, reason = "complains about link"))]
pub trait LendItem<'lend, __ImplyBound: ImplyBound = &'lend Self> {
    /// The item of a lending iterator, with a particular lifetime.
    ///
    /// The lifetime can force the consumer to drop the item before obtaining a new item (which
    /// requires a mutable borrow to the iterator, invalidating the borrow of the previous item).
    type Item: 'lend;
}

/// The item of a lending iterator, with a particular lifetime.
///
/// The lifetime can force the consumer to drop the item before obtaining a new item (which
/// requires a mutable borrow to the iterator, invalidating the borrow of the previous item).
pub type LentItem<'lend, L> = <L as LendItem<'lend>>::Item;
