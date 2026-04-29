use std::sync::Arc;

use async_trait::async_trait;

use crate::{driver::{driver::DbDriver, executor::DbExecutor}, types::DbResult};

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
