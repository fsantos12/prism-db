mod entity;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

/// Derives `DbEntityTrait` for a struct.
///
/// # Struct attributes
/// - `#[db(collection = "table_name")]` — required, sets the collection/table name
///
/// # Field attributes
/// - `#[db(primary_key)]` — marks the field as part of the primary key (at least one required)
/// - `#[db(column = "col_name")]` — overrides the column name (defaults to field name)
/// - `#[db(ignore)]` — excludes the field from all DB operations; uses `Default::default()` when loading
///
/// # Requirements
///
/// Each non-ignored field type must implement:
/// - `Into<DbValue>` / `From<FieldType> for DbValue` — for `to_db()` and `primary_key()`
/// - `TryFrom<DbValue>` + `Default` — for `from_db()`
///
/// # Example
///
/// ```rust,ignore
/// #[derive(Debug, Clone, PartialEq, DbEntity)]
/// #[db(collection = "users")]
/// pub struct User {
///     #[db(primary_key)]
///     id: i64,
///     name: String,
///     #[db(column = "email_address")]
///     email: String,
///     #[db(ignore)]
///     cached_token: String,
/// }
/// ```
#[proc_macro_derive(DbEntity, attributes(db))]
pub fn derive_db_entity(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    entity::derive(input).into()
}
