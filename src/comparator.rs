use core::cmp::Ordering;

#[cfg(feature = "clone-behavior")]
use clone_behavior::{DeepClone, MirroredClone, Speed};
#[cfg(feature = "generic-container")]
use generic_container::{FragileContainer, GenericContainer};

use crate::key_kind::{KeyKind, KeyOf, OrdKeyKind};


/// Interface for comparing keys (or entries) of a sorted collection.
///
/// The comparison function should provide a total order, just as [`Ord`] would. Additionally,
/// any clones of a comparator value should behave identically to the source comparator.
///
/// Note that none of the axioms that define a total order require that two elements which compare
/// as equal are "*truly*" equal in some more fundamental sense; that is, keys which are distinct
/// (perhaps according to an [`Eq`] implementation) may compare as equal in the provided total
/// order and corresponding equivalence relation.
///
/// Unsafe code is *not* allowed to rely on the correctness of implementations; that is, an
/// incorrect `Comparator` implementation may cause severe logic errors, but must not cause
/// memory unsafety.
pub trait Comparator<Key> where Key: KeyKind {
    /// Compare two keys (or entries) in a sorted collection.
    ///
    /// This method is analogous to [`Ord::cmp`], and should provide a total order.
    ///
    /// Note that none of the axioms that define a total order require that two elements which
    /// compare as equal are "*truly*" equal in some more fundamental sense; that is, keys which
    /// are distinct (perhaps according to an [`Eq`] implementation) may compare as equal in
    /// the provided total order and corresponding equivalence relation.
    ///
    /// Unsafe code is *not* allowed to rely on the correctness of implementations; that is, an
    /// incorrect implementation may cause severe logic errors, but must not cause
    /// memory unsafety.
    #[must_use]
    fn cmp(&self, lhs: KeyOf<'_, Key>, rhs: KeyOf<'_, Key>) -> Ordering;
}

#[cfg(feature = "generic-container")]
impl<Key: KeyKind, C: FragileContainer<dyn Comparator<Key>>> Comparator<Key> for C {
    #[inline]
    fn cmp(&self, lhs: KeyOf<'_, Key>, rhs: KeyOf<'_, Key>) -> Ordering {
        // I'm slightly paranoid about the type coercion coercing to the wrong thing,
        // but doing this line-by-line is probably unnecessary.
        let inner = self.get_ref();
        let inner: &dyn Comparator<Key> = &*inner;
        inner.cmp(lhs, rhs)
    }
}

#[cfg(feature = "generic-container")]
impl<T, C, Key> Comparator<Key> for GenericContainer<T, C>
where
    T:   ?Sized + Comparator<Key>,
    C:   ?Sized + FragileContainer<T>,
    Key: KeyKind,
{
    #[inline]
    fn cmp(&self, lhs: KeyOf<'_, Key>, rhs: KeyOf<'_, Key>) -> Ordering {
        let inner = self.container.get_ref();
        let inner: &T = &inner;
        inner.cmp(lhs, rhs)
    }
}

/// A [`Comparator`] which uses keys' [`Ord`] implementations via the [`OrdKeyKind`] trait.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OrdComparator;

impl<Key: OrdKeyKind> Comparator<Key> for OrdComparator
{
    #[inline]
    fn cmp(&self, lhs: KeyOf<'_, Key>, rhs: KeyOf<'_, Key>) -> Ordering {
        Key::cmp(lhs, rhs)
    }
}

#[cfg(feature = "clone-behavior")]
impl<S: Speed> DeepClone<S> for OrdComparator {
    #[inline]
    fn deep_clone(&self) -> Self {
        Self
    }
}

#[cfg(feature = "clone-behavior")]
impl<S: Speed> MirroredClone<S> for OrdComparator {
    #[inline]
    fn mirrored_clone(&self) -> Self {
        Self
    }
}
