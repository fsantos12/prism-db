use async_trait::async_trait;

use crate::types::{DbResult, DbRow};

/// An async iterator over the rows returned by a query.
///
/// Call [`next`](DbCursor::next) repeatedly until it returns `Ok(None)` to exhaust the cursor.
#[async_trait]
pub trait DbCursor: Send {
    /// Advances the cursor and returns the next row, or `Ok(None)` when exhausted.
    async fn next(&mut self) -> DbResult<Option<Box<dyn DbRow>>>;
}
