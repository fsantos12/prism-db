use std::sync::Arc;

use async_trait::async_trait;

use crate::{driver::{executor::DbExecutor, transaction::DbTransaction}, types::DbResult};

#[async_trait]
pub trait DbDriver: DbExecutor {
    async fn begin(&self) -> DbResult<Arc<dyn DbTransaction>>;
    async fn ping(&self) -> DbResult<()> { Ok(()) }
}