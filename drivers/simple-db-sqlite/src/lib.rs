//! # simple-db-sqlite
//!
//! SQLite driver implementation for simple-db.
//!
//! Provides:
//! - `SqliteDriver` for connection pooling and query execution
//! - Transaction support via `SqliteTransaction`
//! - In-memory and file-based database support
//! - Type mappings for SQLite's dynamic type system
//! - Integration with sqlx for parameter binding and result handling

mod types;
mod builders;
mod queries;
mod driver;

pub use driver::{SqliteDriver, SqliteTransaction};
pub use sqlx::sqlite::SqlitePoolOptions;
pub use sqlx::SqlitePool;
