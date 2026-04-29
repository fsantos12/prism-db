use async_trait::async_trait;

use crate::types::{DbResult, DbRow};

#[async_trait]
pub trait DbCursor: Send {
    async fn next(&mut self) -> DbResult<Option<Box<dyn DbRow>>>;
}