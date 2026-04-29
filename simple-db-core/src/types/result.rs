use crate::types::DbError;

pub type DbResult<T> = Result<T, DbError>;