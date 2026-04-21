use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use rust_decimal::Decimal;
use serde_json::Value as JsonValue;
use simple_db_core::types::{DbRow, DbValue};
use sqlx::{postgres::PgRow, Column, Row, TypeInfo, ValueRef};
use uuid::Uuid;

/// Adapter that wraps a [`PgRow`] and exposes it through the [`DbRow`] interface.
///
/// Maps each PostgreSQL OID type name to its exact [`DbValue`] variant — no
/// lossy string coercions. Unknown types fall back to NULL.
pub struct PostgresDbRow {
    row: PgRow,
}

impl PostgresDbRow {
    /// Creates a new adapter wrapping the given raw PostgreSQL row.
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
            // --- integers ---
            "INT2" | "SMALLINT" => {
                let val: i16 = self.row.try_get(index).ok()?;
                Some(DbValue::from_i16(val))
            }
            "INT4" | "INT" | "SERIAL" => {
                let val: i32 = self.row.try_get(index).ok()?;
                Some(DbValue::from_i32(val))
            }
            "INT8" | "BIGINT" | "BIGSERIAL" => {
                let val: i64 = self.row.try_get(index).ok()?;
                Some(DbValue::from_i64(val))
            }

            // --- floats ---
            "FLOAT4" | "REAL" => {
                let val: f32 = self.row.try_get(index).ok()?;
                Some(DbValue::from_f32(val))
            }
            "FLOAT8" | "DOUBLE PRECISION" => {
                let val: f64 = self.row.try_get(index).ok()?;
                Some(DbValue::from_f64(val))
            }

            // --- decimal (NUMERIC columns and aggregate results like AVG) ---
            "NUMERIC" | "DECIMAL" => {
                let val: Decimal = self.row.try_get(index).ok()?;
                Some(DbValue::from_decimal(val))
            }

            // --- boolean ---
            "BOOL" => {
                let val: bool = self.row.try_get(index).ok()?;
                Some(DbValue::from_bool(val))
            }

            // --- binary ---
            "BYTEA" => {
                let val: Vec<u8> = self.row.try_get(index).ok()?;
                Some(DbValue::from_bytes(val))
            }

            // --- text ---
            "TEXT" | "VARCHAR" | "BPCHAR" | "NAME" => {
                let val: String = self.row.try_get(index).ok()?;
                Some(DbValue::from_string(val))
            }

            // --- uuid ---
            "UUID" => {
                let val: Uuid = self.row.try_get(index).ok()?;
                Some(DbValue::from_uuid(val))
            }

            // --- json ---
            "JSON" | "JSONB" => {
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
            "TIMESTAMP" => {
                let val: NaiveDateTime = self.row.try_get(index).ok()?;
                Some(DbValue::from_timestamp(val))
            }
            "TIMESTAMPTZ" => {
                let val: DateTime<Utc> = self.row.try_get(index).ok()?;
                Some(DbValue::from_timestampz(val))
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
