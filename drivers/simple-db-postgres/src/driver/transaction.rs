use std::sync::Arc;

use async_trait::async_trait;
use simple_db_core::{driver::{executor::DbExecutor, transaction::DbTransaction}, query::{FindQuery, InsertQuery, PreparedDeleteQuery, PreparedFindQuery, PreparedInsertQuery, PreparedUpdateQuery, UpdateQuery}, types::{DbError, DbResult}};
use sqlx::{Postgres, Transaction};
use tokio::sync::Mutex;

use crate::{driver::executor::PostgresExecutor, queries::{find::PostgresPreparedFindQuery, insert::PostgresPreparedInsertQuery, update::PostgresPreparedUpdateQuery}};

pub struct PostgresTransaction {
    executor: PostgresExecutor,
}

impl PostgresTransaction {
    pub fn new(tx: Transaction<'static, Postgres>) -> Self {
        Self {
            executor: PostgresExecutor::Transaction(Arc::new(Mutex::new(Some(tx)))),
        }
    }
}

#[async_trait]
impl DbExecutor for PostgresTransaction {
    fn prepare_find(&self, query: FindQuery) -> DbResult<Box<dyn PreparedFindQuery + '_>> {
        Ok(Box::new(PostgresPreparedFindQuery::new(&self.executor, query)))
    }

    fn prepare_insert(&self, query: InsertQuery) -> DbResult<Box<dyn PreparedInsertQuery + '_>> {
        Ok(Box::new(PostgresPreparedInsertQuery::new(&self.executor, query)))
    }

    fn prepare_update(&self, query: UpdateQuery) -> DbResult<Box<dyn PreparedUpdateQuery + '_>> {
        Ok(Box::new(PostgresPreparedUpdateQuery::new(&self.executor, query)))
    }

    fn prepare_delete(&self, query: simple_db_core::query::DeleteQuery) -> DbResult<Box<dyn PreparedDeleteQuery + '_>> {
        Ok(Box::new(crate::queries::delete::PostgresPreparedDeleteQuery::new(&self.executor, query)))
    }
}

#[async_trait]
impl DbTransaction for PostgresTransaction {
    async fn commit(&self) -> DbResult<()> {
        if let PostgresExecutor::Transaction(inner) = &self.executor {
            let mut guard = inner.lock().await;
            if let Some(tx) = guard.take() {
                tx.commit().await.map_err(DbError::driver)?;
                Ok(())
            } else {
                Err(DbError::Internal("Transaction already closed".into()))
            }
        } else {
            unreachable!("PostgresTransaction must always hold a Transaction variant")
        }
    }

    async fn rollback(&self) -> DbResult<()> {
        if let PostgresExecutor::Transaction(inner) = &self.executor {
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
