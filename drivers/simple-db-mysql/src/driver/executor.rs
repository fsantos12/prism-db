use std::sync::Arc;
use tokio::sync::Mutex;
use sqlx::{MySql, MySqlPool, Transaction, mysql::{MySqlArguments, MySqlQueryResult}, query::Query};

pub(crate) enum MySqlExecutor {
    Pool(MySqlPool),
    Transaction(Arc<Mutex<Option<Transaction<'static, MySql>>>>),
}

impl MySqlExecutor {
    pub(crate) async fn execute(&self, query: Query<'_, MySql, MySqlArguments>) -> sqlx::Result<MySqlQueryResult> {
        match self {
            MySqlExecutor::Pool(pool) => query.execute(pool).await,
            MySqlExecutor::Transaction(shared_tx) => {
                let mut guard = shared_tx.lock().await;
                if let Some(tx) = guard.as_mut() {
                    query.execute(&mut **tx).await
                } else {
                    Err(sqlx::Error::WorkerCrashed)
                }
            }
        }
    }

    pub(crate) async fn fetch_all(&self, query: Query<'_, MySql, MySqlArguments>) -> sqlx::Result<Vec<sqlx::mysql::MySqlRow>> {
        match self {
            MySqlExecutor::Pool(pool) => query.fetch_all(pool).await,
            MySqlExecutor::Transaction(shared_tx) => {
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
