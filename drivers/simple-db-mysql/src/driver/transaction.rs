use std::sync::Arc;

use async_trait::async_trait;
use simple_db_core::{driver::{executor::DbExecutor, transaction::DbTransaction}, query::{FindQuery, InsertQuery, PreparedDeleteQuery, PreparedFindQuery, PreparedInsertQuery, PreparedUpdateQuery, UpdateQuery}, types::{DbError, DbResult}};
use sqlx::{MySql, Transaction};
use tokio::sync::Mutex;

use crate::{driver::executor::MySqlExecutor, queries::{find::MySqlPreparedFindQuery, insert::MySqlPreparedInsertQuery, update::MySqlPreparedUpdateQuery}};

pub struct MySqlTransaction {
    executor: MySqlExecutor,
}

impl MySqlTransaction {
    pub fn new(tx: Transaction<'static, MySql>) -> Self {
        Self {
            executor: MySqlExecutor::Transaction(Arc::new(Mutex::new(Some(tx)))),
        }
    }
}

#[async_trait]
impl DbExecutor for MySqlTransaction {
    fn prepare_find(&self, query: FindQuery) -> DbResult<Box<dyn PreparedFindQuery + '_>> {
        Ok(Box::new(MySqlPreparedFindQuery::new(&self.executor, query)))
    }

    fn prepare_insert(&self, query: InsertQuery) -> DbResult<Box<dyn PreparedInsertQuery + '_>> {
        Ok(Box::new(MySqlPreparedInsertQuery::new(&self.executor, query)))
    }

    fn prepare_update(&self, query: UpdateQuery) -> DbResult<Box<dyn PreparedUpdateQuery + '_>> {
        Ok(Box::new(MySqlPreparedUpdateQuery::new(&self.executor, query)))
    }

    fn prepare_delete(&self, query: simple_db_core::query::DeleteQuery) -> DbResult<Box<dyn PreparedDeleteQuery + '_>> {
        Ok(Box::new(crate::queries::delete::MySqlPreparedDeleteQuery::new(&self.executor, query)))
    }
}

#[async_trait]
impl DbTransaction for MySqlTransaction {
    async fn commit(&self) -> DbResult<()> {
        if let MySqlExecutor::Transaction(inner) = &self.executor {
            let mut guard = inner.lock().await;
            if let Some(tx) = guard.take() { // Extrai a transação, deixando None
                tx.commit().await.map_err(DbError::driver)?;
                Ok(())
            } else {
                Err(DbError::Internal("Transaction already closed".into()))
            }
        } else {
            unreachable!("MySqlTransaction must always hold a Transaction variant")
        }
    }

    async fn rollback(&self) -> DbResult<()> {
        if let MySqlExecutor::Transaction(inner) = &self.executor {
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