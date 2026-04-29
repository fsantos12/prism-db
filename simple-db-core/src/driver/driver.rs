use std::sync::Arc;

use async_trait::async_trait;

use crate::{driver::{executor::DbExecutor, transaction::DbTransaction}, types::DbResult};

/// Top-level database driver trait.
///
/// Extends [`DbExecutor`] to provide transaction management and connectivity checks.
/// Typically wrapped in an [`Arc`] and used via [`DbContext`](crate::DbContext).
#[async_trait]
pub trait DbDriver: DbExecutor {
    /// Begins a new database transaction and returns a handle to it.
    async fn begin(&self) -> DbResult<Arc<dyn DbTransaction>>;

    /// Checks the connection to the database server. Default implementation always succeeds.
    async fn ping(&self) -> DbResult<()> { Ok(()) }
}
