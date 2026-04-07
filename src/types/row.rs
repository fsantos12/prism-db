use std::collections::HashMap;

use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use serde_json::Value as JsonValue;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::types::{DbError, value::DbValue};

#[derive(Debug, Clone, Default)]
pub struct DbRow(pub HashMap<String, DbValue>);

macro_rules! impl_type_helpers {
    // Branch para tipos Boxed (ex: String, Json)
    ($suffix:ident, $variant:ident, $type:ty, boxed) => {
        paste::paste! {
            /// Get a reference to a boxed field. Rust's deref coercion handles &Box<T> -> &T.
            pub fn [<get_ $suffix>](&self, key: &str) -> Result<&$type, DbError> {
                match self.get(key) {
                    Some(DbValue::$variant(Some(v))) => Ok(v), // Deref coercion automatically works here
                    Some(DbValue::$variant(None)) => Err(DbError::MappingError(format!("Field '{}' is NULL", key))),
                    Some(other) => Err(DbError::TypeError { 
                        expected: stringify!($variant).to_string(), 
                        found: format!("{:?}", other) 
                    }),
                    None => Err(DbError::NotFound),
                }
            }

            /// Takes ownership and automatically unboxes the value.
            pub fn [<take_ $suffix>](&mut self, key: &str) -> Result<$type, DbError> {
                match self.take(key) {
                    Some(DbValue::$variant(Some(v))) => Ok(*v), // The '*' unboxes the value
                    Some(DbValue::$variant(None)) => Err(DbError::MappingError(format!("Field '{}' is NULL", key))),
                    Some(other) => Err(DbError::TypeError { 
                        expected: stringify!($variant).to_string(), 
                        found: format!("{:?}", other) 
                    }),
                    None => Err(DbError::NotFound),
                }
            }
        }
    };

    // Branch para tipos simples na Stack (ex: i32, bool)
    ($suffix:ident, $variant:ident, $type:ty) => {
        paste::paste! {
            pub fn [<get_ $suffix>](&self, key: &str) -> Result<&$type, DbError> {
                match self.get(key) {
                    Some(DbValue::$variant(Some(v))) => Ok(v),
                    Some(DbValue::$variant(None)) => Err(DbError::MappingError(format!("Field '{}' is NULL", key))),
                    Some(other) => Err(DbError::TypeError { 
                        expected: stringify!($variant).to_string(), 
                        found: format!("{:?}", other) 
                    }),
                    None => Err(DbError::NotFound),
                }
            }

            pub fn [<take_ $suffix>](&mut self, key: &str) -> Result<$type, DbError> {
                match self.take(key) {
                    Some(DbValue::$variant(Some(v))) => Ok(v),
                    _ => Err(DbError::MappingError(format!("Invalid or missing field '{}'", key))),
                }
            }
        }
    };
}


impl DbRow {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    /// Insert a value into the row
    pub fn insert<K: Into<String>, V: Into<DbValue>>(&mut self, key: K, value: V) {
        self.0.insert(key.into(), value.into());
    }

    /// Get a reference to a value
    pub fn get(&self, key: &str) -> Option<&DbValue> {
        self.0.get(key)
    }

    /// Take ownership of a value (removes it from the row)
    /// This is very efficient for mapping to models
    pub fn take(&mut self, key: &str) -> Option<DbValue> {
        self.0.remove(key)
    }

    // Implement type-specific helper methods for common types
    // Primitive types
    impl_type_helpers!(i8, I8, i8);
    impl_type_helpers!(i16, I16, i16);
    impl_type_helpers!(i32, I32, i32);
    impl_type_helpers!(i64, I64, i64);
    impl_type_helpers!(i128, I128, i128);
    impl_type_helpers!(u8, U8, u8);
    impl_type_helpers!(u16, U16, u16);
    impl_type_helpers!(u32, U32, u32);
    impl_type_helpers!(u64, U64, u64);
    impl_type_helpers!(u128, U128, u128);
    impl_type_helpers!(f32, F32, f32);
    impl_type_helpers!(f64, F64, f64);
    impl_type_helpers!(bool, Bool, bool);
    impl_type_helpers!(char, Char, char);

    // Temporal types
    impl_type_helpers!(date, Date, NaiveDate);
    impl_type_helpers!(time, Time, NaiveTime);
    impl_type_helpers!(timestamp, Timestamp, NaiveDateTime);
    impl_type_helpers!(timestamptz, Timestamptz, DateTime<Utc>);

    // Large types (boxed for efficiency)
    impl_type_helpers!(decimal, Decimal, Decimal, boxed);
    impl_type_helpers!(string, String, String, boxed);
    impl_type_helpers!(bytes, Bytes, Vec<u8>, boxed);
    impl_type_helpers!(uuid, Uuid, Uuid, boxed);
    impl_type_helpers!(json, Json, JsonValue, boxed);
}

// This allows: .collect::<DbRow>()
impl FromIterator<(String, DbValue)> for DbRow {
    fn from_iter<I: IntoIterator<Item = (String, DbValue)>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

pub trait FromDbRow: Sized {
    fn from_db_row(row: DbRow) -> Result<Self, DbError>;
}