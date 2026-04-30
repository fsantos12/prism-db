//! # prism-db-postgres
//!
//! PostgreSQL driver implementation for prism-db.
//!
//! Provides:
//! - `PostgresDriver` for connection pooling and query execution
//! - Transaction support via `PostgresTransaction`
//! - Type mappings for PostgreSQL-specific data types (DECIMAL, JSON, arrays, etc.)
//! - Parameterized query support with $1, $2, ... placeholders
//! - Integration with sqlx for result handling

mod types;
mod builders;
mod queries;
mod driver;

pub use driver::{PostgresDriver, PostgresTransaction};
pub use sqlx::postgres::PgPoolOptions;
pub use sqlx::PgPool;
