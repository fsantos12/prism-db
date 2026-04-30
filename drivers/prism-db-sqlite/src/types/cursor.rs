use futures::{stream::BoxStream, StreamExt};
use prism_db_core::types::{DbCursor, DbError, DbResult};
use sqlx::sqlite::SqliteRow;

use crate::types::row::SqliteDbRow;

pub(crate) struct SqliteDbCursor {
    stream: BoxStream<'static, Result<SqliteRow, sqlx::Error>>,
}

impl SqliteDbCursor {
    pub fn new(stream: BoxStream<'static, Result<SqliteRow, sqlx::Error>>) -> Self {
        Self { stream }
    }
}

#[async_trait::async_trait]
impl DbCursor for SqliteDbCursor {
    async fn next(&mut self) -> DbResult<Option<Box<dyn prism_db_core::types::DbRow>>> {
        match self.stream.next().await {
            Some(Ok(row)) => Ok(Some(Box::new(SqliteDbRow::new(row)))),
            Some(Err(err)) => Err(DbError::driver(err)),
            None => Ok(None),
        }
    }
}
