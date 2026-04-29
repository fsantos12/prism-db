#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("type mismatch: expected {expected}, found {found}")]
    TypeMismatch { expected: String, found: String },

    #[error("column index out of bounds: {0}")]
    ColumnIndexOutOfBounds(usize),

    #[error("column not found: '{0}'")]
    ColumnNotFound(String),

    #[error("driver error: {0}")]
    Driver(#[source] Box<dyn std::error::Error + Send + Sync>),

    #[error("internal error: {0}")]
    Internal(String),
}

impl DbError {
    pub fn driver(e: impl std::error::Error + Send + Sync + 'static) -> Self {
        DbError::Driver(Box::new(e))
    }
}