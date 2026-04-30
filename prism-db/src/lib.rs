//! # prism-db
//!
//! A comprehensive, modular ORM and query builder for Rust supporting multiple SQL backends.
//!
//! ## Architecture
//!
//! **prism-db** is designed as a **backend-agnostic** core that can support any database via pluggable drivers:
//!
//! - **prism-db-core**: Database-independent abstractions (traits, builders, types)
//! - **prism-db-orm**: Entity tracking, change detection, and persistence helpers
//! - **prism-db-macros**: Procedural macros for automatic trait derivation
//! - **Drivers** (sqlite, postgres, mysql): Backend-specific implementations using sqlx
//!
//! ## Features
//!
//! - **Query builders**: Type-safe SQL construction via `FindQuery`, `InsertQuery`, `UpdateQuery`, `DeleteQuery`
//! - **Change tracking**: Automatic INSERT/UPDATE/DELETE via `DbEntity<T>` change detection
//! - **Transactions**: ACID semantics with `DbTransaction` and `DbTransactionExt::transaction()`
//! - **Async/await**: Built on tokio async runtime
//! - **Type safety**: `DbValue` tagged pointer for type-safe column data
//! - **Connection pooling**: sqlx pool management for all backends
//!
//! ## Getting Started
//!
//! ### 1. Define an Entity
//!
//! ```ignore
//! use prism_db::{DbEntity, DeriveDbEntity};
//!
//! #[derive(DeriveDbEntity, Clone)]
//! #[db(table = "users")]
//! pub struct User {
//!     #[db(primary_key)]
//!     pub id: i64,
//!     pub name: String,
//!     pub email: String,
//! }
//! ```
//!
//! ### 2. Connect to a Database
//!
//! ```ignore
//! use prism_db::SqliteDriver;
//!
//! let driver = SqliteDriver::connect("sqlite::memory:").await?;
//! ```
//!
//! ### 3. Query
//!
//! ```ignore
//! use prism_db::query::{FindQuery, FilterBuilder};
//!
//! let query = FindQuery::new("users")
//!     .filter(FilterBuilder::new().eq("email", "alice@example.com").build());
//! let mut cursor = driver.find(query).await?;
//! ```
//!
//! ### 4. Persist
//!
//! ```ignore
//! let mut user = DbEntity::new(User {
//!     id: 1,
//!     name: "Alice".to_string(),
//!     email: "alice@example.com".to_string(),
//! });
//! user.save(&driver).await?;  // INSERT
//! ```
//!
//! ## Features
//!
//! Enable in `Cargo.toml`:
//! ```toml
//! [dependencies]
//! prism-db = { version = "0.1", features = ["sqlite", "orm", "macros"] }
//! ```
//!
//! - `sqlite`: SQLite driver
//! - `postgres`: PostgreSQL driver
//! - `mysql`: MySQL driver
//! - `orm`: Entity tracking and persistence
//! - `macros`: Automatic trait derivation

pub use prism_db_core::DbContext;
pub use prism_db_core::driver;
pub use prism_db_core::query;
pub use prism_db_core::types;
pub use prism_db_core::{filter, project, sort, group};

#[cfg(feature = "orm")]
pub use prism_db_orm::{DbEntity, DbEntityTrait, TrackingState};

#[cfg(feature = "orm")]
pub use prism_db_macros::DbEntity as DeriveDbEntity;

#[cfg(feature = "sqlite")]
pub use prism_db_sqlite::{SqlitePool, SqlitePoolOptions, SqliteDriver, SqliteTransaction};

#[cfg(feature = "postgres")]
pub use prism_db_postgres::{PgPool, PgPoolOptions, PostgresDriver, PostgresTransaction};

#[cfg(feature = "mysql")]
pub use prism_db_mysql::{MySqlPool, MySqlPoolOptions, MySqlDriver, MySqlTransaction};