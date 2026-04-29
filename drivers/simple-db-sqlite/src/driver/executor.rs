use std::sync::Arc;
use tokio::sync::Mutex;
use sqlx::{Sqlite, SqlitePool, Transaction, sqlite::{SqliteArguments, SqliteQueryResult}, query::Query};

pub(crate) enum SqliteExecutor {
    Pool(SqlitePool),
    Transaction(Arc<Mutex<Option<Transaction<'static, Sqlite>>>>),
}

impl SqliteExecutor {
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
