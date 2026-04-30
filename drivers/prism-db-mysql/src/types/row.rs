use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use rust_decimal::Decimal;
use serde_json::Value as JsonValue;
use prism_db_core::types::{DbRow, DbValue};
use sqlx::{mysql::MySqlRow, Column, Row, TypeInfo, ValueRef};

pub(crate) struct MySqlDbRow {
    row: MySqlRow,
}

impl MySqlDbRow {
    pub fn new(row: MySqlRow) -> Self {
        Self { row }
    }
}

impl DbRow for MySqlDbRow {
    fn get_by_index(&self, index: usize) -> Option<DbValue> {
        let raw_value = self.row.try_get_raw(index).ok()?;
        if raw_value.is_null() { return Some(DbValue::from_null()); }

        let type_name = raw_value.type_info().name().to_uppercase();
        match type_name.as_str() {
            // --- integers ---
            "TINYINT" => {
                let val: i8 = self.row.try_get(index).ok()?;
                Some(DbValue::from(val))
            }
            "SMALLINT" => {
                let val: i16 = self.row.try_get(index).ok()?;
                Some(DbValue::from(val))
            }
            "MEDIUMINT" | "INT" | "INTEGER" => {
                let val: i32 = self.row.try_get(index).ok()?;
                Some(DbValue::from(val))
            }
            "BIGINT" => {
                let val: i64 = self.row.try_get(index).ok()?;
                Some(DbValue::from(val))
            }
            "TINYINT UNSIGNED" => {
                let val: u8 = self.row.try_get(index).ok()?;
                Some(DbValue::from(val))
            }
            "SMALLINT UNSIGNED" => {
                let val: u16 = self.row.try_get(index).ok()?;
                Some(DbValue::from(val))
            }
            "MEDIUMINT UNSIGNED" | "INT UNSIGNED" | "INTEGER UNSIGNED" => {
                let val: u32 = self.row.try_get(index).ok()?;
                Some(DbValue::from(val))
            }
            "BIGINT UNSIGNED" => {
                let val: u64 = self.row.try_get(index).ok()?;
                Some(DbValue::from(val))
            }

            // --- floats ---
            "FLOAT" => {
                let val: f32 = self.row.try_get(index).ok()?;
                Some(DbValue::from(val))
            }
            "DOUBLE" => {
                let val: f64 = self.row.try_get(index).ok()?;
                Some(DbValue::from(val))
            }

            // --- decimal ---
            "DECIMAL" | "NEWDECIMAL" => {
                let val: Decimal = self.row.try_get(index).ok()?;
                Some(DbValue::from(val))
            }

            // --- boolean ---
            "BOOL" | "BOOLEAN" => {
                let val: bool = self.row.try_get(index).ok()?;
                Some(DbValue::from(val))
            }

            // --- binary ---
            "BLOB" | "TINYBLOB" | "MEDIUMBLOB" | "LONGBLOB" | "VARBINARY" | "BINARY" => {
                let val: Vec<u8> = self.row.try_get(index).ok()?;
                Some(DbValue::from(val))
            }

            // --- text ---
            "CHAR" | "VARCHAR" | "TINYTEXT" | "TEXT" | "MEDIUMTEXT" | "LONGTEXT" | "ENUM" | "SET" => {
                let val: String = self.row.try_get(index).ok()?;
                Some(DbValue::from(val))
            }

            // --- json ---
            "JSON" => {
                let val: JsonValue = self.row.try_get(index).ok()?;
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
