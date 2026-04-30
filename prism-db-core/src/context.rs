use std::sync::Arc;

use async_trait::async_trait;

use crate::{driver::{driver::DbDriver, executor::DbExecutor, transaction::DbTransaction}, query::{DeleteQuery, FindQuery, InsertQuery, PreparedDeleteQuery, PreparedFindQuery, PreparedInsertQuery, PreparedUpdateQuery, UpdateQuery}, types::DbResult};

/// The primary entry point for running queries against a database.
///
/// `DbContext` wraps an [`Arc<dyn DbDriver>`] and implements [`DbExecutor`], so callers use
/// the same query API regardless of which underlying driver is configured.
pub struct DbContext {
    driver: Arc<dyn DbDriver>,
}

impl DbContext {
    /// Creates a new context backed by `driver`.
    pub fn new(driver: Arc<dyn DbDriver>) -> Self {
        DbContext { driver }
    }
}

#[async_trait]
impl DbExecutor for DbContext {
    fn prepare_find(&self, query: FindQuery) -> DbResult<Box<dyn PreparedFindQuery + '_>> {
        self.driver.prepare_find(query)
    }

    fn prepare_insert(&self, query: InsertQuery) -> DbResult<Box<dyn PreparedInsertQuery + '_>> {
        self.driver.prepare_insert(query)
    }

    fn prepare_update(&self, query: UpdateQuery) -> DbResult<Box<dyn PreparedUpdateQuery + '_>> {
        self.driver.prepare_update(query)
    }

    fn prepare_delete(&self, query: DeleteQuery) -> DbResult<Box<dyn PreparedDeleteQuery + '_>> {
        self.driver.prepare_delete(query)
    }
}

#[async_trait]
impl DbDriver for DbContext {
    async fn begin_transaction(&self) -> DbResult<Arc<dyn DbTransaction>> {
        self.driver.begin_transaction().await
    }

    async fn ping(&self) -> DbResult<()> {
        self.driver.ping().await
    }
}