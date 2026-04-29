use std::sync::Arc;

use async_trait::async_trait;
use simple_db_core::{driver::{executor::DbExecutor, transaction::{close_transaction, DbTransaction}}, query::{FindQuery, InsertQuery, PreparedDeleteQuery, PreparedFindQuery, PreparedInsertQuery, PreparedUpdateQuery, UpdateQuery}, types::DbResult};
use sqlx::{Sqlite, Transaction};
use tokio::sync::Mutex;

use crate::{driver::executor::SqliteExecutor, queries::{find::SqlitePreparedFindQuery, insert::SqlitePreparedInsertQuery, update::SqlitePreparedUpdateQuery}};

/// A SQLite database transaction.
///
/// Represents an active SQLite transaction that can be committed or rolled back.
/// All queries executed within a transaction use the transaction's connection rather
/// than the pool, ensuring isolation.
pub struct SqliteTransaction {
    /// The executor wrapping the sqlx transaction.
    executor: SqliteExecutor,
}

impl SqliteTransaction {
    /// Creates a new transaction from an sqlx `Transaction<Sqlite>`.
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
            close_transaction(inner, "Transaction already closed", |tx: Transaction<'static, Sqlite>| async move { tx.commit().await }).await
        } else {
            unreachable!("SqliteTransaction must always hold a Transaction variant")
        }
    }

    async fn rollback(&self) -> DbResult<()> {
        if let SqliteExecutor::Transaction(inner) = &self.executor {
            close_transaction(inner, "Transaction already closed", |tx: Transaction<'static, Sqlite>| async move { tx.rollback().await }).await
        } else {
            unreachable!()
        }
    }
}
