<div align="center" class="rustdoc-hidden">
<h1> Seekable Iterator </h1>
</div>

[<img alt="github" src="https://img.shields.io/badge/github-seekable--iterator-08f?logo=github" height="20">](https://github.com/robofinch/seekable-iterator/)
[![Latest version](https://img.shields.io/crates/v/seekable-iterator.svg)](https://crates.io/crates/seekable-iterator)
[![Documentation](https://img.shields.io/docsrs/thread-checked-lock)](https://docs.rs/seekable-iterator/0)
[![Apache 2.0 or MIT license.](https://img.shields.io/badge/license-Apache--2.0_OR_MIT-blue.svg)](#license)

# Traits

Provides:
  - [`SeekableIterator`], [`SeekableLendingIterator`], and [`SeekablePooledIterator`] traits, for
    circular iterators that can move backwards or forwards and seek.
  - [`PooledIterator`] trait, for iterators that would normally be a lending iterator, but use a
    buffer pool to lend out multiple items at the same time.
  - [`CursorIterator`], [`CursorLendingIterator`], and [`CursorPooledIterator`] traits, for
    circular iterators that can move backwards or forwards by one element.
  - [`Seekable`] trait, with all the seeking methods required by the `Seekable*Iterator` traits.
  - [`Comparator`] trait, for comparisons done to seek.

Adapters to [`lender::Lender`] and [`lending_iterator::LendingIterator`] are provided for the
lending iterator trait.

# Semantics

The `PooledIterator` trait makes roughly the same semantic assumptions about the iterator as
a normal iterator.

However, the `Cursor*Iterator` and `Seekable*Iterator` traits assume that an implementor is a
circular iterator over some ordered collection; the iterator is made circular by adding a phantom
element before the first element and after the last element of the ordered collection. The iterator
is thus not a [`FusedIterator`], as after iteration over the collection is completed, the iterator
wraps back around to the start.

The `PooledIterator` and `Cursor*Iterator` traits do not expose any comparator that the ordered
collection and iterator might be using, but the [`Seekable`] and `Seekable*Iterator` traits _do_
expose it via a [`Comparator`] generic. A [`DefaultComparator`] struct is provided that can compare
keys that implement [`Ord`], using their [`Ord`] implementation.

## License

Licensed under either of

* Apache License, Version 2.0 ([LICENSE-APACHE][])
* MIT license ([LICENSE-MIT][])

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
this crate by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without
any additional terms or conditions.

[LICENSE-APACHE]: LICENSE-APACHE
[LICENSE-MIT]: LICENSE-MIT

[`PooledIterator`]: https://docs.rs/seekable-iterator/0/seekable_iterator/trait.PooledIterator.html

[`SeekableIterator`]: https://docs.rs/seekable-iterator/0/seekable_iterator/trait.SeekableIterator.html
[`SeekableLendingIterator`]: https://docs.rs/seekable-iterator/0/seekable_iterator/trait.SeekableLendingIterator.html
[`SeekablePooledIterator`]: https://docs.rs/seekable-iterator/0/seekable_iterator/trait.SeekablePooledIterator.html

[`CursorIterator`]: https://docs.rs/seekable-iterator/0/seekable_iterator/trait.CursorIterator.html
[`CursorLendingIterator`]: https://docs.rs/seekable-iterator/0/seekable_iterator/trait.CursorLendingIterator.html
[`CursorPooledIterator`]: https://docs.rs/seekable-iterator/0/seekable_iterator/trait.CursorPooledIterator.html

[`Seekable`]: https://docs.rs/seekable-iterator/0/seekable_iterator/trait.Seekable.html
[`Comparator`]: https://docs.rs/seekable-iterator/0/seekable_iterator/trait.Comparator.html
[`DefaultComparator`]: https://docs.rs/seekable-iterator/0/seekable_iterator/struct.DefaultComparator.html

[`Ord`]: https://doc.rust-lang.org/std/cmp/trait.Ord.html
[`FusedIterator`]: https://doc.rust-lang.org/std/iter/trait.FusedIterator.html
[`lender::Lender`]: https://docs.rs/lender/0.3.2/lender/trait.Lender.html
[`lending_iterator::LendingIterator`]: https://docs.rs/lending-iterator/0.1.7/lending_iterator/trait.LendingIterator.html
