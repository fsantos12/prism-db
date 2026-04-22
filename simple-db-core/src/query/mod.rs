//! # simple-db-query
//!
//! Type-safe, driver-agnostic query builder for the simple-db ecosystem.
//!
//! ## Quick Start
//!
//! ```rust
//! use simple_db_core::query::Query;
//! use simple_db_core::{filter, project, sort};
//!
//! // SELECT with filters, sorts, and pagination
//! let q = Query::find("users")
//!     .project(project!(field("name"), field("email")))
//!     .filter(filter!(gt("age", 18i32)))
//!     .order_by(sort!(asc("name")))
//!     .limit(10);
//!
//! // INSERT
//! let q = Query::insert("users")
//!     .insert(vec![("name", "Alice"), ("email", "alice@example.com")]);
//!
//! // UPDATE
//! let q = Query::update("users")
//!     .set("active", false)
//!     .filter(filter!(lt("last_login_days", 90i32)));
//!
//! // DELETE
//! let q = Query::delete("users")
//!     .filter(filter!(eq("archived", true)));
//! ```
//!
//! ## Modules
//!
//! - [`queries`] — [`Query`] entry point and CRUD query types
//! - [`builders`] — filter, projection, sort, and group-by builders

mod builders;
mod queries;

// Re-export the most commonly used types at the crate root for convenience.
pub use queries::{Query, Collection, FindQuery, InsertQuery, UpdateQuery, DeleteQuery};
pub use builders::{
    Filter, FilterBuilder, FilterDefinition,
    Projection, ProjectionBuilder, ProjectionDefinition,
    Sort, SortBuilder, SortDefinition,
    GroupBuilder, GroupDefinition,
};
