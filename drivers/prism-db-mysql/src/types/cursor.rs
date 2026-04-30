use futures::{stream::BoxStream, StreamExt};
use prism_db_core::types::{DbCursor, DbError, DbResult};
use sqlx::mysql::MySqlRow;

use crate::types::row::MySqlDbRow;

pub(crate) struct MySqlDbCursor {
    stream: BoxStream<'static, Result<MySqlRow, sqlx::Error>>,
}

impl MySqlDbCursor {
    pub fn new(stream: BoxStream<'static, Result<MySqlRow, sqlx::Error>>) -> Self {
        Self { stream }
    }
}

#[async_trait::async_trait]
impl DbCursor for MySqlDbCursor {
    async fn next(&mut self) -> DbResult<Option<Box<dyn prism_db_core::types::DbRow>>> {
        match self.stream.next().await {
            Some(Ok(row)) => Ok(Some(Box::new(MySqlDbRow::new(row)))),
            Some(Err(err)) => Err(DbError::driver(err)),
            None => Ok(None),
        }
    }
}
