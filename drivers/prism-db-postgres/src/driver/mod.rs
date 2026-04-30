//! PostgreSQL driver implementation module.
//!
//! Provides [`PostgresDriver`] for connection pooling and [`PostgresTransaction`] for transactional queries.

pub mod executor;
mod driver;
mod transaction;

pub use driver::PostgresDriver;
pub use transaction::PostgresTransaction;
