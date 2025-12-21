use core::{cmp::Ordering, marker::PhantomData};
use core::fmt::{Debug, Formatter, Result as FmtResult};

use crate::lending_iterator_support::ImplyBound;


/// Trait for indicating the type of key used by a [`Seekable`] iterator, for a particular
/// lifetime. The key is expected to be cheap to `Clone`, and likely `Copy`.
///
/// [`Seekable`]: crate::seekable::Seekable
pub trait KeyWithLifetime<'key, __ImplyBound: ImplyBound = &'key Self> {
    /// The key used by a [`Seekable`] iterator, with a particular lifetime.
    /// It is expected to be cheap to `Clone`, and likely `Copy`.
    ///
    /// [`Seekable`]: crate::seekable::Seekable
    type Key: 'key + Clone;
}

/// The key used by a [`Seekable`] iterator, with a particular lifetime.
///
/// [`Seekable`]: crate::seekable::Seekable
pub type KeyOf<'key, K> = <K as KeyWithLifetime<'key>>::Key;

/// A trait for indicating the type of keys used by a [`Seekable`] iterator, which may vary
/// with a lifetime. The keys are expected to be cheap to `Clone`, and likely `Copy`.
///
/// [`Seekable`]: crate::seekable::Seekable
pub trait KeyKind: for<'a> KeyWithLifetime<'a> {}

/// A version of [`Ord`] which allows keys with different lifetimes to be compared.
pub trait OrdKeyKind: KeyKind {
    /// A version of [`Ord::cmp`] which allows keys with different lifetimes to be compared.
    #[must_use]
    fn cmp(lhs: KeyOf<'_, Self>, rhs: KeyOf<'_, Self>) -> Ordering;
}

/// Indicate that a [`Seekable`] iterator uses the same key type for all lifetimes.
/// The key type `T` should be cheap to `Clone`, and likely `Copy`.
///
/// [`Seekable`]: crate::seekable::Seekable
// Covariant over `T`
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct TKey<T: ?Sized>(PhantomData<fn() -> T>);

impl<T: Clone> KeyWithLifetime<'_> for TKey<T> {
    type Key = T;
}

impl<T: Clone> KeyKind for TKey<T> {}

impl<T: Clone + Ord> OrdKeyKind for TKey<T> {
    #[inline]
    fn cmp(lhs: KeyOf<'_, Self>, rhs: KeyOf<'_, Self>) -> Ordering {
        Ord::cmp(&lhs, &rhs)
    }
}

impl<T: ?Sized> Debug for TKey<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_tuple("TKey").field(&self.0).finish()
    }
}

impl<T: ?Sized> Copy for TKey<T> {}

impl<T: ?Sized> Clone for TKey<T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

/// Indicate that a [`Seekable`] iterator uses a `&'key T` key type.
///
/// [`Seekable`]: crate::seekable::Seekable
// Covariant over `T`, just like `&'_ T`.
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct RefKey<T: ?Sized>(PhantomData<fn() -> T>);

impl<'key, T: ?Sized> KeyWithLifetime<'key> for RefKey<T> {
    type Key = &'key T;
}

impl<T: ?Sized> KeyKind for RefKey<T> {}

impl<T: ?Sized + Ord> OrdKeyKind for RefKey<T> {
    #[inline]
    fn cmp(lhs: KeyOf<'_, Self>, rhs: KeyOf<'_, Self>) -> Ordering {
        Ord::cmp(lhs, rhs)
    }
}

impl<T: ?Sized> Debug for RefKey<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_tuple("RefKey").field(&self.0).finish()
    }
}

impl<T: ?Sized> Copy for RefKey<T> {}

impl<T: ?Sized> Clone for RefKey<T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}
