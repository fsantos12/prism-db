use std::future::Future;
use std::sync::Arc;

use async_trait::async_trait;

use crate::driver::{DbDriver, DbExecutor};
use crate::types::DbResult;

/// A pinned connection with an open transaction.
///
/// `DbTransaction` extends [`DbExecutor`] with `commit` and `rollback`.
/// Every query executed through this handle runs on the same dedicated
/// connection, guaranteeing atomicity.
///
/// Obtain a handle via [`DbDriver::begin`](crate::DbDriver::begin). Use
/// [`DbTransactionExt::transaction`] for automatic lifecycle management.
///
/// # Example (manual)
///
/// ```rust,ignore
/// let tx = driver.begin().await?;
/// tx.insert(query1).await?;
/// tx.update(query2).await?;
/// tx.commit().await?;
/// ```
#[async_trait]
pub trait DbTransaction: DbExecutor {
    /// Commits all operations executed through this handle and ends the transaction.
    async fn commit(&self) -> DbResult<()>;

    /// Rolls back all operations executed through this handle and ends the transaction.
    async fn rollback(&self) -> DbResult<()>;
}

// ---------------------------------------------------------------------------
// Managed transaction helper
// ---------------------------------------------------------------------------

/// Extension trait on `Arc<dyn DbDriver>` for managed transactions.
///
/// # Why a separate trait?
///
/// `transaction<F, Fut, T>` is a generic method. Generic methods break object
/// safety, so they cannot live directly on `DbDriver` when it is used as
/// `dyn DbDriver`. Implementing this as an extension trait on
/// `Arc<dyn DbDriver>` is the idiomatic Rust solution — the same pattern used
/// by `Iterator` adapters in the standard library.
///
/// # Example
///
/// ```rust,ignore
/// use simple_db_driver::DbTransactionExt;
///
/// driver.transaction(|tx| async move {
///     tx.insert(Query::insert("orders").insert(row)).await?;
///     tx.update(Query::update("inventory").set("stock", stock - 1)).await?;
///     Ok(())
/// }).await?;
/// ```
#[async_trait]
pub trait DbTransactionExt {
    async fn transaction<F, Fut, T>(&self, f: F) -> DbResult<T>
    where
        F: FnOnce(Arc<dyn DbTransaction>) -> Fut + Send,
        Fut: Future<Output = DbResult<T>> + Send,
        T: Send;
}

#[async_trait]
impl DbTransactionExt for Arc<dyn DbDriver> {
    async fn transaction<F, Fut, T>(&self, f: F) -> DbResult<T>
    where F: FnOnce(Arc<dyn DbTransaction>) -> Fut + Send, Fut: Future<Output = DbResult<T>> + Send, T: Send, {
        let tx = self.begin().await?;
        let result = f(tx.clone()).await;
        match result {
            Ok(value) => {
                tx.commit().await?;
                Ok(value)
            }
            Err(e) => {
                let _ = tx.rollback().await;
                Err(e)
            }
        }
    }
}
