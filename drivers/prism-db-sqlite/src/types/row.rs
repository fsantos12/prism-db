use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use prism_db_core::types::{DbRow, DbValue};
use sqlx::{sqlite::SqliteRow, Column, Row, TypeInfo, ValueRef};

pub(crate) struct SqliteDbRow {
    row: SqliteRow,
}

impl SqliteDbRow {
    pub fn new(row: SqliteRow) -> Self {
        Self { row }
    }
}

impl DbRow for SqliteDbRow {
    fn get_by_index(&self, index: usize) -> Option<DbValue> {
        let raw_value = self.row.try_get_raw(index).ok()?;
        if raw_value.is_null() { return Some(DbValue::from_null()); }

        let type_name = raw_value.type_info().name().to_uppercase();
        match type_name.as_str() {
            // --- integers ---
            "INTEGER" => {
                let val: i64 = self.row.try_get(index).ok()?;
                Some(DbValue::from(val))
            }

            // --- floats ---
            "REAL" => {
                let val: f64 = self.row.try_get(index).ok()?;
                Some(DbValue::from(val))
            }

            // --- boolean (SQLite stores as 0/1) ---
            "BOOLEAN" | "BOOL" => {
                let val: bool = self.row.try_get(index).ok()?;
                Some(DbValue::from(val))
            }

            // --- binary ---
            "BLOB" => {
                let val: Vec<u8> = self.row.try_get(index).ok()?;
                Some(DbValue::from(val))
            }

            // --- text ---
            "TEXT" => {
                let val: String = self.row.try_get(index).ok()?;
                Some(DbValue::from(val))
            }

            // --- date / time ---
            "DATE" => {
                let val: NaiveDate = self.row.try_get(index).ok()?;
                Some(DbValue::from(val))
            }
            "TIME" => {
                let val: NaiveTime = self.row.try_get(index).ok()?;
                Some(DbValue::from(val))
            }
            "DATETIME" | "TIMESTAMP" => {
                let val: NaiveDateTime = self.row.try_get(index).ok()?;
                Some(DbValue::from(val))
            }

            _ => Some(DbValue::from_null()),
        }
    }

    fn get_by_name(&self, name: &str) -> Option<DbValue> {
        let column = self.row.try_column(name).ok()?;
        self.get_by_index(column.ordinal())
    }

    fn len(&self) -> usize {
        self.row.columns().len()
    }
}
