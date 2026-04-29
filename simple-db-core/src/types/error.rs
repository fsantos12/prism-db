/// All errors that can be returned by the database layer.
#[derive(Debug, thiserror::Error)]
pub enum DbError {
    /// The stored column type did not match the requested Rust type.
    #[error("type mismatch: expected {expected}, found {found}")]
    TypeMismatch { expected: String, found: String },

    /// A column was accessed by an index that is out of range for this row.
    #[error("column index out of bounds: {0}")]
    ColumnIndexOutOfBounds(usize),

    /// A column was accessed by name but no column with that name exists in this row.
    #[error("column not found: '{0}'")]
    ColumnNotFound(String),

    /// An error propagated from the underlying database driver.
    #[error("driver error: {0}")]
    Driver(#[source] Box<dyn std::error::Error + Send + Sync>),

    /// An unexpected internal error that should not occur during normal operation.
    #[error("internal error: {0}")]
    Internal(String),
}

impl DbError {
    /// Wraps any driver-level error into [`DbError::Driver`].
    pub fn driver(e: impl std::error::Error + Send + Sync + 'static) -> Self {
        DbError::Driver(Box::new(e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_mismatch_error() {
        let err = DbError::TypeMismatch {
            expected: "String".to_string(),
            found: "i64".to_string(),
        };
        assert!(err.to_string().contains("type mismatch"));
    }

    #[test]
    fn test_column_index_out_of_bounds() {
        let err = DbError::ColumnIndexOutOfBounds(10);
        assert!(err.to_string().contains("out of bounds: 10"));
    }

    #[test]
    fn test_column_not_found() {
        let err = DbError::ColumnNotFound("username".to_string());
        assert!(err.to_string().contains("username"));
    }

    #[test]
    fn test_internal_error() {
        let err = DbError::Internal("unexpected state".to_string());
        assert!(err.to_string().contains("internal error"));
    }
}
