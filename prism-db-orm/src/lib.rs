//! # prism-db-orm
//!
//! ORM (Object-Relational Mapping) layer for prism-db.
//!
//! Provides:
//! - `DbEntity<T>` wrapper for tracking entity state (untracked, tracked, detached)
//! - `DbEntityTrait` for mapping Rust types to database rows
//! - Entity change tracking for automatic UPDATE generation
//! - `DbCursorEntityExt` for hydrating entities from query results
//!
//! # Example
//!
//! ```ignore
//! #[derive(Clone)]
//! struct User {
//!     id: i64,
//!     name: String,
//! }
//!
//! impl DbEntityTrait for User {
//!     fn table_name() -> &'static str { "users" }
//!     fn primary_key(&self) -> Vec<(&'static str, DbValue)> {
//!         vec![("id", DbValue::from(self.id))]
//!     }
//!     fn to_db(&self) -> Vec<(&'static str, DbValue)> {
//!         vec![
//!             ("id", DbValue::from(self.id)),
//!             ("name", DbValue::from(self.name.clone())),
//!         ]
//!     }
//!     fn from_db(row: &dyn DbRow) -> Self {
//!         User {
//!             id: row.get("id").and_then(|v| v.as_i64()).unwrap_or(0),
//!             name: row.get("name").and_then(|v| v.as_string()).unwrap_or_default(),
//!         }
//!     }
//! }
//!
//! // Create and save entity
//! let mut user = DbEntity::new(User { id: 1, name: "Alice".to_string() });
//! user.save(&driver).await?;
//! ```

mod entity;
mod cursor;

pub use entity::{DbEntityTrait, DbEntity, TrackingState};
pub use cursor::DbCursorEntityExt;