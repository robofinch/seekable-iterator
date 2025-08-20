// See https://linebender.org/blog/doc-include for this README inclusion strategy
// File links are not supported by rustdoc
//!
//! [LICENSE-APACHE]: https://github.com/robofinch/seekable-iterator/blob/main/LICENSE-APACHE
//! [LICENSE-MIT]: https://github.com/robofinch/seekable-iterator/blob/main/LICENSE-MIT
//!
//! [`PooledIterator`]: PooledIterator
//!
//! [`SeekableIterator`]: SeekableIterator
//! [`SeekableLendingIterator`]: SeekableLendingIterator
//! [`SeekablePooledIterator`]: SeekablePooledIterator
//!
//! [`CursorIterator`]: CursorIterator
//! [`CursorLendingIterator`]: CursorLendingIterator
//! [`CursorPooledIterator`]: CursorPooledIterator
//!
//! [`Seekable`]: Seekable
//! [`Comparator`]: Comparator
//! [`DefaultComparator`]: DefaultComparator
//!
//! [`Ord`]: Ord
//! [`FusedIterator`]: core::iter::FusedIterator
#![cfg_attr(feature = "lender", doc = " [`lender::Lender`]: lender::Lender")]
#![cfg_attr(
    feature = "lending-iterator",
    doc = " [`lending_iterator::LendingIterator`]: lending_iterator::LendingIterator",
)]
#![cfg_attr(
    feature = "alloc",
    doc = " [`MergingIter`]: merging_iter::MergingIter",
)]
#![cfg_attr(
    feature = "alloc",
    doc = " [`PooledIter`]: pooled_iter::PooledIter",
)]
#![cfg_attr(
    feature = "std",
    doc = " [`ThreadsafePooledIter`]: threadsafe_pooled_iter::ThreadsafePooledIter",
)]
//!
//! <style>
//! .rustdoc-hidden { display: none; }
//! </style>
#![cfg_attr(doc, doc = include_str!("../README.md"))]

#![cfg_attr(
    feature = "lending-iterator",
    expect(
        non_ascii_idents, clippy::disallowed_script_idents,
        reason = "`gat` uses non-ascii character",
    ),
)]

#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

mod comparator;
mod cursor;
mod pooled;
mod seekable;
mod seekable_iterators;

mod lending_iterator_support;

#[cfg(feature = "alloc")]
mod merging_iter;
#[cfg(feature = "alloc")]
mod pooled_iter;
#[cfg(feature = "std")]
mod threadsafe_pooled_iter;

// TODO: adapter for cursor traits and `Seekable` that applies `Borrow::borrow` to input keys.
// Note sure if it's useful though.

#[cfg(feature = "lender")]
mod lender_adapter;
#[cfg(feature = "lending-iterator")]
mod lending_iterator_adapter;


pub use self::{
    comparator::{Comparator, DefaultComparator},
    cursor::{CursorIterator, CursorLendingIterator, CursorPooledIterator},
    lending_iterator_support::{ImplyBound, LendItem, LentItem},
    pooled::{OutOfBuffers, PooledIterator},
    seekable::{ItemToKey, Seekable},
    seekable_iterators::{SeekableIterator, SeekableLendingIterator, SeekablePooledIterator},
};

#[cfg(feature = "lender")]
pub use self::lender_adapter::{LenderAdapter, PooledLenderAdapter};
#[cfg(feature = "lending-iterator")]
pub use self::lending_iterator_adapter::{LendingIteratorAdapter, PooledLendingIteratorAdapter};

#[cfg(feature = "alloc")]
pub use self::merging_iter::MergingIter;
#[cfg(feature = "alloc")]
pub use self::pooled_iter::{PooledIter, PoolItem};
#[cfg(feature = "std")]
pub use self::threadsafe_pooled_iter::{ThreadsafePooledIter, ThreadsafePoolItem};
