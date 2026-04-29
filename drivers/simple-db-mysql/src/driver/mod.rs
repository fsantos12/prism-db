//! MySQL driver implementation module.
//!
//! Provides [`MySqlDriver`] for connection pooling and [`MySqlTransaction`] for transactional queries.

pub mod executor;
mod driver;
mod transaction;

pub use driver::MySqlDriver;
pub use transaction::MySqlTransaction;