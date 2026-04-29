use std::sync::Arc;

use async_trait::async_trait;
use simple_db_core::{driver::{executor::DbExecutor, transaction::DbTransaction}, query::{FindQuery, InsertQuery, PreparedDeleteQuery, PreparedFindQuery, PreparedInsertQuery, PreparedUpdateQuery, UpdateQuery}, types::{DbError, DbResult}};
use sqlx::{Sqlite, Transaction};
use tokio::sync::Mutex;

use crate::{driver::executor::SqliteExecutor, queries::{find::SqlitePreparedFindQuery, insert::SqlitePreparedInsertQuery, update::SqlitePreparedUpdateQuery}};

pub struct SqliteTransaction {
    executor: SqliteExecutor,
}

impl SqliteTransaction {
    pub fn new(tx: Transaction<'static, Sqlite>) -> Self {
        Self {
            executor: SqliteExecutor::Transaction(Arc::new(Mutex::new(Some(tx)))),
        }
    }
}

#[async_trait]
impl DbExecutor for SqliteTransaction {
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
impl DbTransaction for SqliteTransaction {
    async fn commit(&self) -> DbResult<()> {
        if let SqliteExecutor::Transaction(inner) = &self.executor {
            let mut guard = inner.lock().await;
            if let Some(tx) = guard.take() {
                tx.commit().await.map_err(DbError::driver)?;
                Ok(())
            } else {
                Err(DbError::Internal("Transaction already closed".into()))
            }
        } else {
            unreachable!("SqliteTransaction must always hold a Transaction variant")
        }
    }

    async fn rollback(&self) -> DbResult<()> {
        if let SqliteExecutor::Transaction(inner) = &self.executor {
            let mut guard = inner.lock().await;
            if let Some(tx) = guard.take() {
                tx.rollback().await.map_err(DbError::driver)?;
                Ok(())
            } else {
                Err(DbError::Internal("Transaction already closed".into()))
            }
        } else {
            unreachable!()
        }
    }
}
