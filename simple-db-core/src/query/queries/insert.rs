use async_trait::async_trait;

use crate::types::{DbResult, DbValue};

/// An INSERT query builder.
///
/// Each row must have the same column schema.
#[derive(Debug, Clone)]
pub struct InsertQuery {
    /// Target table name.
    pub table: String,
    /// Rows to insert; each row is an ordered list of `(column, value)` pairs.
    pub values: Vec<Vec<(String, DbValue)>>,
}

impl InsertQuery {
    /// Creates a new INSERT query for the given table.
    pub fn new(table: impl Into<String>) -> Self {
        Self {
            table: table.into(),
            values: Vec::new(),
        }
    }

    /// Appends a single row to the query.
    pub fn insert<I, K, V>(mut self, row: I) -> Self
    where I: IntoIterator<Item = (K, V)>, K: Into<String>, V: Into<DbValue> {
        let db_row: Vec<(String, DbValue)> = row.into_iter().map(|(k, v)| (k.into(), v.into())).collect();
        self.values.push(db_row);
        self
    }

    /// Appends multiple rows to the query.
    pub fn bulk_insert<I, R, K, V>(mut self, rows: I) -> Self
    where I: IntoIterator<Item = R>, R: IntoIterator<Item = (K, V)>, K: Into<String>, V: Into<DbValue> {
        for row in rows {
            let db_row: Vec<(String, DbValue)> = row.into_iter().map(|(k, v)| (k.into(), v.into())).collect();
            self.values.push(db_row);
        }
        self
    }
}

/// Compiled, ready-to-execute INSERT query.
#[async_trait]
pub trait PreparedInsertQuery: Send + Sync {
    async fn execute(&self) -> DbResult<u64>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_sets_table_name() {
        let q = InsertQuery::new("users");
        assert_eq!(q.table, "users");
        assert!(q.values.is_empty());
    }

    #[test]
    fn insert_appends_one_row() {
        let q = InsertQuery::new("users").insert([("id", DbValue::from(1i32)), ("name", DbValue::from("alice".to_string()))]);
        assert_eq!(q.values.len(), 1);
        assert_eq!(q.values[0].len(), 2);
    }

    #[test]
    fn insert_chaining_appends_multiple_rows() {
        let q = InsertQuery::new("t")
            .insert([("x", DbValue::from(1i32))])
            .insert([("x", DbValue::from(2i32))]);
        assert_eq!(q.values.len(), 2);
    }

    #[test]
    fn bulk_insert_appends_all_rows() {
        let rows = vec![
            vec![("a", DbValue::from(1i32))],
            vec![("a", DbValue::from(2i32))],
            vec![("a", DbValue::from(3i32))],
        ];
        let q = InsertQuery::new("t").bulk_insert(rows);
        assert_eq!(q.values.len(), 3);
    }

    #[test]
    fn column_names_are_preserved() {
        let q = InsertQuery::new("t").insert([("col_a", DbValue::from(42i32)), ("col_b", DbValue::from("hello".to_string()))]);
        assert_eq!(q.values[0][0].0, "col_a");
        assert_eq!(q.values[0][1].0, "col_b");
    }
}