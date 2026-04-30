//! # simple-db-mysql
//!
//! MySQL driver implementation for simple-db.
//!
//! Provides:
//! - `MySqlDriver` for connection pooling and query execution
//! - Transaction support via `MySqlTransaction`
//! - Type mappings for MySQL-specific data types (DECIMAL, JSON, DATE, etc.)
//! - Integration with sqlx for parameter binding and result handling

mod types;
mod builders;
mod queries;
mod driver;

pub use driver::{MySqlDriver, MySqlTransaction};
pub use sqlx::mysql::MySqlPoolOptions;
pub use sqlx::MySqlPool;