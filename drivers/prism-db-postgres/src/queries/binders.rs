use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use rust_decimal::Decimal;
use serde_json::Value as JsonValue;
use prism_db_core::types::DbValue;
use sqlx::{Postgres, postgres::PgArguments, query::Query};
use uuid::Uuid;

pub(crate) fn bind_values<'q>(q: Query<'q, Postgres, PgArguments>, values: &[DbValue]) -> Query<'q, Postgres, PgArguments> {
    values.iter().fold(q, |acc, value| bind_value(acc, value))
}

fn bind_value<'q>(q: Query<'q, Postgres, PgArguments>, value: &DbValue) -> Query<'q, Postgres, PgArguments> {
    if value.is_null()                  { q.bind(None::<i64>) }
    else if value.is::<bool>()          { q.bind(value.get::<bool>().unwrap()) }
    else if value.is::<i8>()            { q.bind(value.get::<i8>().unwrap() as i16) }
    else if value.is::<i16>()           { q.bind(value.get::<i16>().unwrap()) }
    else if value.is::<i32>()           { q.bind(value.get::<i32>().unwrap()) }
    else if value.is::<i64>()           { q.bind(value.get::<i64>().unwrap()) }
    else if value.is::<i128>()          { q.bind(value.get::<i128>().unwrap().to_string()) }
    else if value.is::<u8>()            { q.bind(value.get::<u8>().unwrap() as i16) }
    else if value.is::<u16>()           { q.bind(value.get::<u16>().unwrap() as i32) }
    else if value.is::<u32>()           { q.bind(value.get::<u32>().unwrap() as i64) }
    else if value.is::<u64>()           { q.bind(value.get::<u64>().unwrap() as i64) }
    else if value.is::<u128>()          { q.bind(value.get::<u128>().unwrap().to_string()) }
    else if value.is::<f32>()           { q.bind(value.get::<f32>().unwrap()) }
    else if value.is::<f64>()           { q.bind(value.get::<f64>().unwrap()) }
    else if value.is::<Decimal>()       { q.bind(value.get::<Decimal>().unwrap()) }
    else if value.is::<char>()          { q.bind(value.get::<char>().unwrap().to_string()) }
    else if value.is::<String>()        { q.bind(value.get::<String>().unwrap().to_owned()) }
    else if value.is::<Vec<u8>>()       { q.bind(value.get::<Vec<u8>>().unwrap().to_owned()) }
    else if value.is::<Uuid>()          { q.bind(value.get::<Uuid>().unwrap()) }
    else if value.is::<JsonValue>()     { q.bind(value.get::<JsonValue>().unwrap().to_string()) }
    else if value.is::<NaiveDate>()     { q.bind(value.get::<NaiveDate>().unwrap()) }
    else if value.is::<NaiveTime>()     { q.bind(value.get::<NaiveTime>().unwrap()) }
    else if value.is::<NaiveDateTime>() { q.bind(value.get::<NaiveDateTime>().unwrap()) }
    else if value.is::<DateTime<Utc>>() { q.bind(value.get::<DateTime<Utc>>().unwrap()) }
    else                                { q.bind(None::<i64>) }
}
