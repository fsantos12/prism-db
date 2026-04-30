//! # simple-db-core
//!
//! Core query builder, type system, and driver abstractions for simple-db.
//!
//! This crate provides:
//! - A backend-agnostic query builder API (`FindQuery`, `InsertQuery`, `UpdateQuery`, `DeleteQuery`)
//! - Efficient tagged-pointer value type (`DbValue`) for column data
//! - Row and cursor abstractions for result streaming
//! - A high-level context wrapper (`DbContext`) around drivers
//! - Driver traits (`DbDriver`, `DbExecutor`, `DbTransaction`) for implementing backends

pub mod types;
pub mod query;
pub mod driver;
mod context;

pub use context::DbContext;