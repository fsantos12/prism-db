use crate::types::DbError;

/// Alias for `Result<T, DbError>` used throughout the database layer.
pub type DbResult<T> = Result<T, DbError>;
