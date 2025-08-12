use core::error::Error;
use core::fmt::{Display, Formatter, Result as FmtResult};


pub trait PooledIterator {
    type Item;

    fn next(&mut self) -> Option<Self::Item>;

    #[must_use]
    fn would_block(&self) -> bool;

    /// # Errors
    /// Errors if the operation would have blocked, due to no buffers being available.
    fn try_next(&mut self) -> Result<Option<Self::Item>, WouldBlock>;

    #[must_use]
    fn buffer_pool_size(&self) -> usize;

    #[must_use]
    fn available_buffers(&self) -> usize;
}

#[derive(Debug, Clone, Copy)]
pub struct WouldBlock;

impl Display for WouldBlock {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "an operation was not performed because it would be blocking")
    }
}

impl Error for WouldBlock {}
