//! SQLite driver implementation module.
//!
//! Provides [`SqliteDriver`] for connection pooling and [`SqliteTransaction`] for transactional queries.

pub mod executor;
mod driver;
mod transaction;

pub use driver::SqliteDriver;
pub use transaction::SqliteTransaction;
