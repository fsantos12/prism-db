use std::sync::Arc;
use crate::driver::{DbDriver, DbTransaction};
use crate::query::{FindQuery, InsertQuery, UpdateQuery, DeleteQuery};
use crate::types::{DbCursor, DbResult};

/// A context for database operations that wraps a `DbDriver`.
///
/// Delegates all operations to the underlying driver. Can be cloned cheaply
/// since it holds an `Arc<dyn DbDriver>`.
pub struct DbContext {
    driver: Arc<dyn DbDriver>,
}

impl DbContext {
    /// Creates a new context wrapping a driver.
    pub fn new(driver: Arc<dyn DbDriver>) -> Self {
        Self { driver }
    }

    /// Executes a find query.
    pub async fn find(&self, query: FindQuery) -> DbResult<Box<dyn DbCursor>> {
        self.driver.find(query).await
    }

    /// Executes an insert query.
    pub async fn insert(&self, query: InsertQuery) -> DbResult<u64> {
        self.driver.insert(query).await
    }

    /// Executes an update query.
    pub async fn update(&self, query: UpdateQuery) -> DbResult<u64> {
        self.driver.update(query).await
    }

    /// Executes a delete query.
    pub async fn delete(&self, query: DeleteQuery) -> DbResult<u64> {
        self.driver.delete(query).await
    }

    /// Begins a transaction.
    pub async fn begin(&self) -> DbResult<Arc<dyn DbTransaction>> {
        self.driver.begin().await
    }

    /// Pings the database to verify connectivity.
    pub async fn ping(&self) -> DbResult<()> {
        self.driver.ping().await
    }
}

impl Clone for DbContext {
    fn clone(&self) -> Self {
        Self {
            driver: Arc::clone(&self.driver),
        }
    }
}
