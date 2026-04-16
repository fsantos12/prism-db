use async_trait::async_trait;

use simple_db_core::{DbCursor, DbResult};
use simple_db_query::{DeleteQuery, FindQuery, InsertQuery, UpdateQuery};

/// Common CRUD interface shared by both pool drivers and transaction handles.
///
/// Any type that can execute queries implements this trait — whether it is a
/// connection pool ([`DbDriver`](crate::DbDriver)) or an open transaction
/// ([`DbTransaction`](crate::DbTransaction)). Application code that does not
/// need to control transaction boundaries should accept `&dyn DbExecutor`.
///
/// # Example
///
/// ```rust,ignore
/// async fn find_active_users(db: &dyn DbExecutor) -> DbResult<Box<dyn DbCursor>> {
///     db.find(Query::find("users").filter(|b| b.eq("active", true))).await
/// }
///
/// // Works with a pool:
/// find_active_users(driver.as_ref()).await?;
///
/// // Works inside a transaction:
/// driver.transaction(|tx| async move {
///     find_active_users(tx.as_ref()).await?;
///     Ok(())
/// }).await?;
/// ```
#[async_trait]
pub trait DbExecutor: Send + Sync {
    /// Executes a SELECT query and returns a streaming cursor over the matching rows.
    ///
    /// Returns `Ok` with an empty cursor when no rows match — never `Err(NotFound)`.
    async fn find(&self, query: FindQuery) -> DbResult<Box<dyn DbCursor>>;

    /// Executes an INSERT query. Returns the number of rows inserted.
    async fn insert(&self, query: InsertQuery) -> DbResult<u64>;

    /// Executes an UPDATE query. Returns the number of rows affected.
    async fn update(&self, query: UpdateQuery) -> DbResult<u64>;

    /// Executes a DELETE query. Returns the number of rows deleted.
    async fn delete(&self, query: DeleteQuery) -> DbResult<u64>;
}
