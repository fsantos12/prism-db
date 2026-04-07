use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use serde_json::Value as JsonValue;
use rust_decimal::Decimal;
use uuid::Uuid;

/// Helper macro to generate the type definition inside the variant.
/// It wraps the type in Box if the 'boxed' keyword is provided.
macro_rules! wrap_type {
    ($type:ty, boxed) => { Box<$type> };
    ($type:ty,) => { $type };
}

/// Helper macro to wrap the value during conversion.
macro_rules! wrap_val {
    ($val:expr, boxed) => { Box::new($val) };
    ($val:expr,) => { $val };
}

macro_rules! define_db_value {
    ($( $variant:ident($type:ty $(, $boxed:ident)?) ),* $(,)?) => {
        #[derive(Debug, Clone, PartialEq)]
        pub enum DbValue {
            $(
                $variant(Option<wrap_type!($type, $($boxed)?)>),
            )*
        }

        impl DbValue {
            /// Returns true if the inner value is None, regardless of the variant.
            pub fn is_null(&self) -> bool {
                match self {
                    $( DbValue::$variant(v) => v.is_none(), )*
                }
            }
        }

        $(
            // implementation of From<T> for DbValue
            impl From<$type> for DbValue {
                fn from(val: $type) -> Self {
                    DbValue::$variant(Some(wrap_val!(val, $($boxed)?)))
                }
            }

            // implementation of From<Option<T>> for DbValue
            impl From<Option<$type>> for DbValue {
                fn from(val: Option<$type>) -> Self {
                    DbValue::$variant(val.map(|v| wrap_val!(v, $($boxed)?)))
                }
            }
        )*
    };
}

// --- FULL IMPLEMENTATION ---
define_db_value! {
    // Primitive types
    I8(i8), I16(i16), I32(i32), I64(i64), I128(i128),
    U8(u8), U16(u16), U32(u32), U64(u64), U128(u128),
    F32(f32), F64(f64), 
    Bool(bool),
    Char(char),

    // Temporal types
    Date(NaiveDate), 
    Time(NaiveTime), 
    Timestamp(NaiveDateTime), 
    Timestamptz(DateTime<Utc>),

    // Large types marked as 'boxed' for memory efficiency [1, 2]
    Decimal(Decimal, boxed),
    String(String, boxed),
    Bytes(Vec<u8>, boxed),
    Uuid(Uuid, boxed),
    Json(JsonValue, boxed),
}

/// Manual implementations for string slices (&str) to improve ergonomics.
/// We don't include this in the macro to avoid conflicting with the String variant.
impl From<&str> for DbValue {
    fn from(val: &str) -> Self {
        DbValue::String(Some(Box::new(val.to_string())))
    }
}

impl From<Option<&str>> for DbValue {
    fn from(val: Option<&str>) -> Self {
        DbValue::String(val.map(|s| Box::new(s.to_string())))
    }
}