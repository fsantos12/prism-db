use async_trait::async_trait;

use crate::types::{DbResult, DbValue};

#[derive(Debug, Clone)]
pub struct InsertQuery {
    pub table: String,
    pub values: Vec<Vec<(String, DbValue)>>,
}

impl InsertQuery {
    pub fn new(table: impl Into<String>) -> Self {
        Self {
            table: table.into(),
            values: Vec::new(),
        }
    }

    pub fn insert<I, K, V>(mut self, row: I) -> Self
    where I: IntoIterator<Item = (K, V)>, K: Into<String>, V: Into<DbValue> {
        let db_row: Vec<(String, DbValue)> = row.into_iter().map(|(k, v)| (k.into(), v.into())).collect();
        self.values.push(db_row);
        self
    }

    pub fn bulk_insert<I, R, K, V>(mut self, rows: I) -> Self
    where I: IntoIterator<Item = R>, R: IntoIterator<Item = (K, V)>, K: Into<String>, V: Into<DbValue> {
        for row in rows {
            let db_row: Vec<(String, DbValue)> = row.into_iter().map(|(k, v)| (k.into(), v.into())).collect();
            self.values.push(db_row);
        }
        self
    }
}

#[async_trait]
pub trait PreparedInsertQuery: Send + Sync {
    async fn execute(&self) -> DbResult<u64>;
}