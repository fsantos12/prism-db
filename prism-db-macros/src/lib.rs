//! Procedural macros for the prism-db ORM.
//!
//! Provides `#[derive(DbEntity)]` for automatic `DbEntityTrait` implementation.
//!
//! # Example
//!
//! ```ignore
//! use prism_db_macros::DbEntity;
//!
//! #[derive(DbEntity, Clone)]
//! #[db(table = "users")]
//! pub struct User {
//!     #[db(primary_key)]
//!     pub id: i64,
//!     pub name: String,
//!     pub email: String,
//! }
//! ```
//!
//! The derive macro generates:
//! - `table_name()` returning `"users"`
//! - `primary_key()` returning the `id` field
//! - `to_db()` serializing all fields as database values
//! - `from_db()` deserializing from rows
//!
//! # Field Attributes
//!
//! - `#[db(primary_key)]`: Marks a field as part of the primary key (required at least once)
//! - `#[db(ignore)]`: Excludes a field from `to_db()` and `from_db()`
//! - `#[db(name = "column_name")]`: Maps the field to a different database column name

mod entity;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

/// Derives `DbEntityTrait` for a struct.
///
/// At least one field must be marked with `#[db(primary_key)]`.
/// A table name must be specified via `#[db(table = "...")]`.
#[proc_macro_derive(DbEntity, attributes(db))]
pub fn derive_db_entity(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    entity::derive(input).into()
}