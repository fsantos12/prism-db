use std::sync::Arc;
use futures::lock::Mutex;
use sqlx::{Sqlite, SqlitePool, Transaction, sqlite::{SqliteArguments, SqliteQueryResult}, query::Query};

/// Execution context that abstracts over pool and transaction.
///
/// Allows query builders to remain agnostic about whether queries run against
/// a connection pool or an active transaction.
pub(crate) enum SqliteExecutor {
    /// Executes queries against the connection pool.
    Pool(SqlitePool),
    /// Executes queries against an active transaction.
    Transaction(Arc<Mutex<Option<Transaction<'static, Sqlite>>>>),
}

impl SqliteExecutor {
    /// Executes a query against the pool or transaction.
    pub(crate) async fn execute<'q>(&self, query: Query<'q, Sqlite, SqliteArguments<'q>>) -> sqlx::Result<SqliteQueryResult> {
        match self {
            SqliteExecutor::Pool(pool) => query.execute(pool).await,
            SqliteExecutor::Transaction(shared_tx) => {
                let mut guard = shared_tx.lock().await;
                if let Some(tx) = guard.as_mut() {
                    query.execute(&mut **tx).await
                } else {
                    Err(sqlx::Error::WorkerCrashed)
                }
            }
        }
    }

    /// Fetches all result rows.
    pub(crate) async fn fetch_all<'q>(&self, query: Query<'q, Sqlite, SqliteArguments<'q>>) -> sqlx::Result<Vec<sqlx::sqlite::SqliteRow>> {
        match self {
            SqliteExecutor::Pool(pool) => query.fetch_all(pool).await,
            SqliteExecutor::Transaction(shared_tx) => {
                let mut guard = shared_tx.lock().await;
                if let Some(tx) = guard.as_mut() {
                    query.fetch_all(&mut **tx).await
                } else {
                    Err(sqlx::Error::WorkerCrashed)
                }
            }
        }
    }
}
