use futures::{stream::BoxStream, StreamExt};
use prism_db_core::types::{DbCursor, DbError, DbResult};
use sqlx::postgres::PgRow;

use crate::types::row::PostgresDbRow;

pub(crate) struct PostgresDbCursor {
    stream: BoxStream<'static, Result<PgRow, sqlx::Error>>,
}

impl PostgresDbCursor {
    pub fn new(stream: BoxStream<'static, Result<PgRow, sqlx::Error>>) -> Self {
        Self { stream }
    }
}

#[async_trait::async_trait]
impl DbCursor for PostgresDbCursor {
    async fn next(&mut self) -> DbResult<Option<Box<dyn prism_db_core::types::DbRow>>> {
        match self.stream.next().await {
            Some(Ok(row)) => Ok(Some(Box::new(PostgresDbRow::new(row)))),
            Some(Err(err)) => Err(DbError::driver(err)),
            None => Ok(None),
        }
    }
}
