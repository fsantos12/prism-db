use async_trait::async_trait;

use crate::types::{DbResult, DbRow};

/// Async iterator over query result rows.
#[async_trait]
pub trait DbCursor: Send {
    /// Fetches the next row, or `None` when exhausted.
    async fn next(&mut self) -> DbResult<Option<Box<dyn DbRow>>>;
}
