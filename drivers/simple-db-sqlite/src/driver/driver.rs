use std::sync::Arc;

use async_trait::async_trait;
use simple_db_core::{driver::{driver::DbDriver, executor::DbExecutor, transaction::DbTransaction}, query::{FindQuery, InsertQuery, PreparedDeleteQuery, PreparedFindQuery, PreparedInsertQuery, PreparedUpdateQuery, UpdateQuery}, types::{DbError, DbResult}};
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};

use crate::{SqliteTransaction, driver::executor::SqliteExecutor, queries::{find::SqlitePreparedFindQuery, insert::SqlitePreparedInsertQuery, update::SqlitePreparedUpdateQuery}};

pub struct SqliteDriver {
    executor: SqliteExecutor 
}

impl SqliteDriver {
    pub fn new(pool: SqlitePool) -> Self {
        Self { 
            executor: SqliteExecutor::Pool(pool) 
        }
    }

    pub async fn connect(url: &str) -> DbResult<Self> {
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(url)
            .await
            .map_err(DbError::driver)?;
        Ok(Self::new(pool))
    }

    pub async fn execute_raw(&self, sql: &str) -> DbResult<()> {
        let query = sqlx::query(sql);
        self.executor.execute(query)
            .await
            .map_err(DbError::driver)?;
        Ok(())
    }
}

#[async_trait]
impl DbExecutor for SqliteDriver {
    fn prepare_find(&self, query: FindQuery) -> DbResult<Box<dyn PreparedFindQuery + '_>> {
        Ok(Box::new(SqlitePreparedFindQuery::new(&self.executor, query)))
    }

    fn prepare_insert(&self, query: InsertQuery) -> DbResult<Box<dyn PreparedInsertQuery + '_>> {
        Ok(Box::new(SqlitePreparedInsertQuery::new(&self.executor, query)))
    }

    fn prepare_update(&self, query: UpdateQuery) -> DbResult<Box<dyn PreparedUpdateQuery + '_>> {
        Ok(Box::new(SqlitePreparedUpdateQuery::new(&self.executor, query)))
    }

    fn prepare_delete(&self, query: simple_db_core::query::DeleteQuery) -> DbResult<Box<dyn PreparedDeleteQuery + '_>> {
        Ok(Box::new(crate::queries::delete::SqlitePreparedDeleteQuery::new(&self.executor, query)))
    }
}

#[async_trait]
impl DbDriver for SqliteDriver {
    async fn begin(&self) -> DbResult<Arc<dyn DbTransaction>> {
        if let SqliteExecutor::Pool(pool) = &self.executor {
            let tx = pool.begin().await.map_err(DbError::driver)?;
            let sqlite_tx = SqliteTransaction::new(tx);
            Ok(Arc::new(sqlite_tx))
        } else {
            Err(DbError::Internal("Cannot start a transaction from an existing transaction".into()))
        }
    }

    async fn ping(&self) -> DbResult<()> {
        if let SqliteExecutor::Pool(pool) = &self.executor {
            pool.acquire().await.map_err(DbError::driver)?;
        }
        Ok(())
    }
}
