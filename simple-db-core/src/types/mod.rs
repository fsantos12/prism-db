mod error;
mod value;
mod row;
mod cursor;

pub use error::DbError;
pub use value::DbValue;
pub use row::{DbRow, DbRowExt};
pub use cursor::DbCursor;

// =============================================================================
// Result alias
// =============================================================================
/// Shorthand for `Result<T, DbError>`.
///
/// Used as the return type for every fallible operation in the simple-db
/// ecosystem.
pub type DbResult<T> = Result<T, DbError>;