use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use rust_decimal::Decimal;
use serde_json::Value as JsonValue;
use simple_db_core::types::{DbRow, DbValue};
use sqlx::{mysql::MySqlRow, Column, Row, TypeInfo, ValueRef};

/// Adapter that wraps a [`MySqlRow`] and exposes it through the [`DbRow`] interface.
///
/// Maps MySQL type names to the appropriate [`DbValue`] variants.
/// Unknown types are mapped to NULL.
pub struct MysqlDbRow {
    row: MySqlRow,
}

impl MysqlDbRow {
    /// Creates a new adapter wrapping the given raw MySQL row.
    pub fn new(row: MySqlRow) -> Self {
        Self { row }
    }
}

impl DbRow for MysqlDbRow {
    fn get_by_index(&self, index: usize) -> Option<DbValue> {
        let raw_value = self.row.try_get_raw(index).ok()?;
        if raw_value.is_null() { return Some(DbValue::from_null()); }

        let type_name = raw_value.type_info().name().to_uppercase();
        match type_name.as_str() {
            // --- integers ---
            "TINYINT" => {
                let val: i8 = self.row.try_get(index).ok()?;
                Some(DbValue::from_i8(val))
            }
            "SMALLINT" => {
                let val: i16 = self.row.try_get(index).ok()?;
                Some(DbValue::from_i16(val))
            }
            "MEDIUMINT" | "INT" | "INTEGER" => {
                let val: i32 = self.row.try_get(index).ok()?;
                Some(DbValue::from_i32(val))
            }
            "BIGINT" => {
                let val: i64 = self.row.try_get(index).ok()?;
                Some(DbValue::from_i64(val))
            }
            "TINYINT UNSIGNED" => {
                let val: u8 = self.row.try_get(index).ok()?;
                Some(DbValue::from_u8(val))
            }
            "SMALLINT UNSIGNED" => {
                let val: u16 = self.row.try_get(index).ok()?;
                Some(DbValue::from_u16(val))
            }
            "MEDIUMINT UNSIGNED" | "INT UNSIGNED" | "INTEGER UNSIGNED" => {
                let val: u32 = self.row.try_get(index).ok()?;
                Some(DbValue::from_u32(val))
            }
            "BIGINT UNSIGNED" => {
                let val: u64 = self.row.try_get(index).ok()?;
                Some(DbValue::from_u64(val))
            }

            // --- floats ---
            "FLOAT" => {
                let val: f32 = self.row.try_get(index).ok()?;
                Some(DbValue::from_f32(val))
            }
            "DOUBLE" => {
                let val: f64 = self.row.try_get(index).ok()?;
                Some(DbValue::from_f64(val))
            }

            // --- decimal ---
            "DECIMAL" | "NEWDECIMAL" => {
                let val: Decimal = self.row.try_get(index).ok()?;
                Some(DbValue::from_decimal(val))
            }

            // --- boolean ---
            "BOOL" | "BOOLEAN" => {
                let val: bool = self.row.try_get(index).ok()?;
                Some(DbValue::from_bool(val))
            }

            // --- binary ---
            "BLOB" | "TINYBLOB" | "MEDIUMBLOB" | "LONGBLOB" | "VARBINARY" | "BINARY" => {
                let val: Vec<u8> = self.row.try_get(index).ok()?;
                Some(DbValue::from_bytes(val))
            }

            // --- text ---
            "CHAR" | "VARCHAR" | "TINYTEXT" | "TEXT" | "MEDIUMTEXT" | "LONGTEXT" | "ENUM" | "SET" => {
                let val: String = self.row.try_get(index).ok()?;
                Some(DbValue::from_string(val))
            }

            // --- json ---
            "JSON" => {
                let val: JsonValue = self.row.try_get(index).ok()?;
                Some(DbValue::from_json(val))
            }

            // --- date / time ---
            "DATE" => {
                let val: NaiveDate = self.row.try_get(index).ok()?;
                Some(DbValue::from_date(val))
            }
            "TIME" => {
                let val: NaiveTime = self.row.try_get(index).ok()?;
                Some(DbValue::from_time(val))
            }
            "DATETIME" | "TIMESTAMP" => {
                let val: NaiveDateTime = self.row.try_get(index).ok()?;
                Some(DbValue::from_timestamp(val))
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
