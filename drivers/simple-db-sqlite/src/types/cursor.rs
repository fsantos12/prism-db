use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::StreamExt;
use simple_db_core::{DbCursor, DbError, DbRow}; 
use sqlx::sqlite::SqliteRow;
use thiserror::Error;

use crate::types::row::SqliteDbRow;

#[derive(Debug, Error)]
pub enum SqliteDbCursorError {
    #[error("Erro no SQLite: {0}")]
    SqlxError(#[from] sqlx::Error),
}

impl DbError for SqliteDbCursorError {}

pub struct SqliteDbCursor {
    stream: BoxStream<'static, Result<SqliteRow, sqlx::Error>>,
}

impl SqliteDbCursor {
    pub fn new(stream: BoxStream<'static, Result<SqliteRow, sqlx::Error>>) -> Self {
        Self { stream }
    }
}

#[async_trait]
impl DbCursor for SqliteDbCursor {
    async fn next(&mut self) -> Result<Option<Box<dyn DbRow>>, Box<dyn DbError>> {
        let next_item = self.stream.next().await;
        match next_item {
            Some(Ok(sqlite_row)) => {
                let my_row = SqliteDbRow::new(sqlite_row);
                Ok(Some(Box::new(my_row) as Box<dyn DbRow>))
            },
            Some(Err(e)) => {
                let driver_error = SqliteDbCursorError::SqlxError(e);
                Err(Box::new(driver_error))
            },
            None => Ok(None),
        }
    }
}