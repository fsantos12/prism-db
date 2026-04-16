use std::sync::Arc;

use async_trait::async_trait;

use crate::{driver::{DbExecutor, DbTransaction}, types::DbResult};

/// Pool-level database driver.
///
/// `DbDriver` extends [`DbExecutor`] with connection management: it can start
/// a transaction via [`begin`](DbDriver::begin), returning a
/// [`DbTransaction`] handle that is pinned to a single dedicated connection.
///
/// Direct calls to `find`, `insert`, `update`, and `delete` on a `DbDriver`
/// run outside any transaction, each using a pooled connection.
///
/// # Object safety
///
/// The trait is object-safe. Backends are injected at runtime as
/// `Arc<dyn DbDriver>`, keeping application code fully decoupled from the
/// concrete database implementation.
///
/// # Example
///
/// ```rust,ignore
/// let driver: Arc<dyn DbDriver> = Arc::new(SqliteDriver::new(pool));
///
/// // Non-transactional query
/// let mut cursor = driver.find(Query::find("users")).await?;
///
/// // Managed transaction — commit on success, rollback on error
/// driver.transaction(|tx| async move {
///     tx.insert(add_user).await?;
///     tx.update(update_balance).await?;
///     Ok(())
/// }).await?;
/// ```
#[async_trait]
pub trait DbDriver: DbExecutor {
    /// Checks out a connection from the pool and begins a transaction on it.
    ///
    /// The returned [`DbTransaction`] must be committed or rolled back.
    /// Prefer [`Arc<dyn DbDriver>::transaction`] for automatic lifecycle management.
    async fn begin(&self) -> DbResult<Arc<dyn DbTransaction>>;

    /// Verifies that the backend is reachable.
    ///
    /// Default implementation is a no-op that always returns `Ok(())`.
    async fn ping(&self) -> DbResult<()> {
        Ok(())
    }
}
