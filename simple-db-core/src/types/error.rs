/// Database operation errors.
///
/// Distinguishes type mismatches, missing columns, and driver-level failures.
#[derive(Debug, thiserror::Error)]
pub enum DbError {
    /// Type mismatch between expected and actual column value.
    #[error("type mismatch: expected {expected}, found {found}")]
    TypeMismatch { expected: String, found: String },

    /// Column index exceeds the row's column count.
    #[error("column index out of bounds: {0}")]
    ColumnIndexOutOfBounds(usize),

    /// No column exists with the requested name.
    #[error("column not found: '{0}'")]
    ColumnNotFound(String),

    /// Error from the underlying database driver.
    #[error("driver error: {0}")]
    Driver(#[source] Box<dyn std::error::Error + Send + Sync>),

    /// Unexpected internal error (should not occur).
    #[error("internal error: {0}")]
    Internal(String),
}

impl DbError {
    /// Wraps a driver error.
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
