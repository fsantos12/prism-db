use std::sync::Arc;
use tokio::sync::Mutex;
use sqlx::{Postgres, PgPool, Transaction, postgres::{PgArguments, PgQueryResult}, query::Query};

pub(crate) enum PostgresExecutor {
    Pool(PgPool),
    Transaction(Arc<Mutex<Option<Transaction<'static, Postgres>>>>),
}

impl PostgresExecutor {
    pub(crate) async fn execute(&self, query: Query<'_, Postgres, PgArguments>) -> sqlx::Result<PgQueryResult> {
        match self {
            PostgresExecutor::Pool(pool) => query.execute(pool).await,
            PostgresExecutor::Transaction(shared_tx) => {
                let mut guard = shared_tx.lock().await;
                if let Some(tx) = guard.as_mut() {
                    query.execute(&mut **tx).await
                } else {
                    Err(sqlx::Error::WorkerCrashed)
                }
            }
        }
    }

    pub(crate) async fn fetch_all(&self, query: Query<'_, Postgres, PgArguments>) -> sqlx::Result<Vec<sqlx::postgres::PgRow>> {
        match self {
            PostgresExecutor::Pool(pool) => query.fetch_all(pool).await,
            PostgresExecutor::Transaction(shared_tx) => {
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
