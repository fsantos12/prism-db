use async_trait::async_trait;

use crate::{query::{DeleteQuery, FindQuery, InsertQuery, PreparedDeleteQuery, PreparedFindQuery, PreparedInsertQuery, PreparedUpdateQuery, UpdateQuery}, types::{DbCursor, DbResult}};

/// Executes CRUD queries against a database.
///
/// Query preparation (compiling to driver-specific statements) is separated from execution
/// to support prepared statement reuse and driver-specific optimizations.
#[async_trait]
pub trait DbExecutor: Send + Sync {
    /// Compiles a query into a driver-specific prepared statement.
    fn prepare_find(&self, query: FindQuery) -> DbResult<Box<dyn PreparedFindQuery + '_>>;

    /// Prepares and executes a query, returning a cursor over result rows.
    async fn find(&self, query: FindQuery) -> DbResult<Box<dyn DbCursor>>{
        self.prepare_find(query)?.execute().await
    }

    /// Compiles a query into a driver-specific prepared statement.
    fn prepare_insert(&self, query: InsertQuery) -> DbResult<Box<dyn PreparedInsertQuery + '_>>;

    /// Prepares and executes a query, returning the number of rows inserted.
    async fn insert(&self, query: InsertQuery) -> DbResult<u64> {
        self.prepare_insert(query)?.execute().await
    }

    /// Compiles a query into a driver-specific prepared statement.
    fn prepare_update(&self, query: UpdateQuery) -> DbResult<Box<dyn PreparedUpdateQuery + '_>>;

    /// Prepares and executes a query, returning the number of rows affected.
    async fn update(&self, query: UpdateQuery) -> DbResult<u64> {
        self.prepare_update(query)?.execute().await
    }

    /// Compiles a query into a driver-specific prepared statement.
    fn prepare_delete(&self, query: DeleteQuery) -> DbResult<Box<dyn PreparedDeleteQuery + '_>>;

    /// Prepares and executes a query, returning the number of rows deleted.
    async fn delete(&self, query: DeleteQuery) -> DbResult<u64> {
        self.prepare_delete(query)?.execute().await
    }
}
