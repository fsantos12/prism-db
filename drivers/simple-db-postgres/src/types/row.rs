use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use rust_decimal::Decimal;
use serde_json::Value as JsonValue;
use simple_db_core::types::{DbRow, DbValue};
use sqlx::{postgres::PgRow, Column, Row, TypeInfo, ValueRef};
use uuid::Uuid;

pub(crate) struct PostgresDbRow {
    row: PgRow,
}

impl PostgresDbRow {
    pub fn new(row: PgRow) -> Self {
        Self { row }
    }
}

impl DbRow for PostgresDbRow {
    fn get_by_index(&self, index: usize) -> Option<DbValue> {
        let raw_value = self.row.try_get_raw(index).ok()?;
        if raw_value.is_null() { return Some(DbValue::from_null()); }

        let type_name = raw_value.type_info().name().to_uppercase();
        match type_name.as_str() {
            "BOOL" | "BOOLEAN" => {
                let val: bool = self.row.try_get(index).ok()?;
                Some(DbValue::from(val))
            }
            "INT2" | "SMALLINT" | "SMALLSERIAL" => {
                let val: i16 = self.row.try_get(index).ok()?;
                Some(DbValue::from(val))
            }
            "INT4" | "INT" | "INTEGER" | "SERIAL" => {
                let val: i32 = self.row.try_get(index).ok()?;
                Some(DbValue::from(val))
            }
            "INT8" | "BIGINT" | "BIGSERIAL" => {
                let val: i64 = self.row.try_get(index).ok()?;
                Some(DbValue::from(val))
            }
            "FLOAT4" | "REAL" => {
                let val: f32 = self.row.try_get(index).ok()?;
                Some(DbValue::from(val))
            }
            "FLOAT8" | "DOUBLE PRECISION" => {
                let val: f64 = self.row.try_get(index).ok()?;
                Some(DbValue::from(val))
            }
            "NUMERIC" | "DECIMAL" => {
                let val: Decimal = self.row.try_get(index).ok()?;
                Some(DbValue::from(val))
            }
            "BYTEA" => {
                let val: Vec<u8> = self.row.try_get(index).ok()?;
                Some(DbValue::from(val))
            }
            "TEXT" | "VARCHAR" | "BPCHAR" | "CHAR" | "NAME" | "CITEXT" => {
                let val: String = self.row.try_get(index).ok()?;
                Some(DbValue::from(val))
            }
            "JSON" | "JSONB" => {
                let val: JsonValue = self.row.try_get(index).ok()?;
                Some(DbValue::from(val))
            }
            "UUID" => {
                let val: Uuid = self.row.try_get(index).ok()?;
                Some(DbValue::from(val))
            }
            "DATE" => {
                let val: NaiveDate = self.row.try_get(index).ok()?;
                Some(DbValue::from(val))
            }
            "TIME" | "TIMETZ" => {
                let val: NaiveTime = self.row.try_get(index).ok()?;
                Some(DbValue::from(val))
            }
            "TIMESTAMP" => {
                let val: NaiveDateTime = self.row.try_get(index).ok()?;
                Some(DbValue::from(val))
            }
            "TIMESTAMPTZ" => {
                let val: DateTime<Utc> = self.row.try_get(index).ok()?;
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
