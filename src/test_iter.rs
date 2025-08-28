#![expect(clippy::redundant_pub_crate, reason = "emphasize that this is internal")]

use crate::{comparator::DefaultComparator, cursor::CursorLendingIterator};
use crate::{
    lending_iterator_support::{LendItem, LentItem},
    seekable::{ItemToKey, Seekable},
};


pub(crate) struct TestIter<'a> {
    data:   &'a [u8],
    cursor: Option<usize>,
}

impl<'a> TestIter<'a> {
    /// Checks that `data` is sorted and has no duplicate elements.
    pub(crate) fn new(data: &'a [u8]) -> Option<Self> {
        #[expect(
            clippy::indexing_slicing,
            clippy::missing_asserts_for_indexing,
            reason = "window size is 2",
        )]
        if data.is_sorted() && data.windows(2).all(|window| window[0] != window[1]) {
            Some(Self {
                data,
                cursor: None,
            })
        } else {
            None
        }
    }
}

impl<'lend> LendItem<'lend> for TestIter<'_> {
    type Item = &'lend u8;
}

impl CursorLendingIterator for TestIter<'_> {
    fn valid(&self) -> bool {
        self.cursor.is_some()
    }

    fn next(&mut self) -> Option<LentItem<'_, Self>> {
        let next_idx = if let Some(idx) = self.cursor {
            idx + 1
        } else {
            0
        };

        self.cursor = if next_idx < self.data.len() {
            Some(next_idx)
        } else {
            None
        };

        self.current()
    }

    fn current(&self) -> Option<LentItem<'_, Self>> {
        #[expect(clippy::indexing_slicing, reason = "cursor must be in-bounds")]
        Some(&self.data[self.cursor?])
    }

    fn prev(&mut self) -> Option<LentItem<'_, Self>> {
        let current_cursor_idx = if let Some(idx) = self.cursor {
            idx
        } else {
            self.data.len()
        };

        self.cursor = current_cursor_idx.checked_sub(1);

        self.current()
    }
}

impl ItemToKey<u8> for TestIter<'_> {
    fn item_to_key(item: LentItem<'_, Self>) -> &'_ u8 {
        item
    }
}

impl Seekable<u8, DefaultComparator> for TestIter<'_> {
    fn reset(&mut self) {
        self.cursor = None;
    }

    fn seek(&mut self, min_bound: &u8) {
        match self.data.binary_search(min_bound) {
            Ok(found) => self.cursor = Some(found),
            Err(following_idx) => {
                self.cursor = if following_idx < self.data.len() {
                    Some(following_idx)
                } else {
                    None
                };
            }
        }
    }

    fn seek_before(&mut self, strict_upper_bound: &u8) {
        self.cursor = match self.data.binary_search(strict_upper_bound) {
            Ok(found)      => found,
            Err(following) => following,
        }.checked_sub(1);
    }

    fn seek_to_first(&mut self) {
        self.reset();
        self.next();
    }

    fn seek_to_last(&mut self) {
        self.reset();
        self.prev();
    }
}
