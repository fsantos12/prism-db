use std::sync::Arc;
use core::future::Future;

use async_trait::async_trait;
use tokio::sync::Mutex;

use crate::{driver::{driver::DbDriver, executor::DbExecutor}, types::{DbError, DbResult}};

#[async_trait]
pub trait DbTransaction: DbExecutor {
    async fn commit(&self) -> DbResult<()>;
    async fn rollback(&self) -> DbResult<()>;
}

#[async_trait]
pub trait DbTransactionExt {
    async fn transaction<F, Fut, T>(&self, f: F) -> DbResult<T>
    where F: FnOnce(Arc<dyn DbTransaction>) -> Fut + Send, Fut: Future<Output = DbResult<T>> + Send, T: Send;
}

pub async fn close_transaction<T, F, Fut, E>(
    shared_tx: &Arc<Mutex<Option<T>>>,
    closed_error: &'static str,
    action: F,
) -> DbResult<()>
where
    T: Send,
    F: FnOnce(T) -> Fut + Send,
    Fut: Future<Output = Result<(), E>> + Send,
    E: std::error::Error + Send + Sync + 'static,
{
    let mut guard = shared_tx.lock().await;
    let tx = match guard.take() {
        Some(tx) => tx,
        None => return Err(DbError::Internal(closed_error.into())),
    };

    drop(guard);
    action(tx).await.map_err(DbError::driver)
}

#[async_trait]
impl DbTransactionExt for Arc<dyn DbDriver> {
    async fn transaction<F, Fut, T>(&self, f: F) -> DbResult<T>
    where F: FnOnce(Arc<dyn DbTransaction>) -> Fut + Send, Fut: Future<Output = DbResult<T>> + Send, T: Send, {
        let tx = self.begin().await?;
        let result = f(tx.clone()).await;
        match result {
            Ok(value) => {
                tx.commit().await?;
                Ok(value)
            }
            Err(e) => {
                let _ = tx.rollback().await;
                Err(e)
            }
        }
    }
}
