use core::cmp::Ordering;

#[cfg(feature = "clone-behavior")]
use clone_behavior::{IndependentClone, MirroredClone, NearInstant, NonRecursive};
#[cfg(feature = "generic-container")]
use generic_container::{FragileContainer, GenericContainer};


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
pub trait Comparator<Key: ?Sized> {
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
    fn cmp(&self, lhs: &Key, rhs: &Key) -> Ordering;
}

#[cfg(feature = "generic-container")]
impl<Key: ?Sized, C: FragileContainer<dyn Comparator<Key>>> Comparator<Key> for C {
    #[inline]
    fn cmp(&self, lhs: &Key, rhs: &Key) -> Ordering {
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
    Key: ?Sized,
{
    #[inline]
    fn cmp(&self, lhs: &Key, rhs: &Key) -> Ordering {
        let inner = self.container.get_ref();
        let inner: &T = &inner;
        inner.cmp(lhs, rhs)
    }
}

/// A [`Comparator`] which uses keys' [`Ord`] implementations.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DefaultComparator;

impl<Key: ?Sized + Ord> Comparator<Key> for DefaultComparator {
    /// Equivalent to `Ord::cmp(lhs, rhs)`.
    #[inline]
    fn cmp(&self, lhs: &Key, rhs: &Key) -> Ordering {
        Ord::cmp(lhs, rhs)
    }
}

#[cfg(feature = "clone-behavior")]
impl NonRecursive for DefaultComparator {}

#[cfg(feature = "clone-behavior")]
impl IndependentClone<NearInstant> for DefaultComparator {
    #[inline]
    fn independent_clone(&self) -> Self {
        Self
    }
}

#[cfg(feature = "clone-behavior")]
impl MirroredClone<NearInstant> for DefaultComparator {
    #[inline]
    fn mirrored_clone(&self) -> Self {
        Self
    }
}
