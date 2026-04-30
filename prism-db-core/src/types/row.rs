use crate::types::{DbError, DbResult, DbValue};

/// A database query result row.
///
/// Columns are accessible by zero-based index or by name, returning owned [`DbValue`] instances.
pub trait DbRow: Send + Sync {
    /// Returns the column value at the given index.
    fn get_by_index(&self, index: usize) -> Option<DbValue>;

    /// Returns the column value by name.
    fn get_by_name(&self, name: &str) -> Option<DbValue>;

    /// Returns the column count.
    fn len(&self) -> usize;

    /// Returns `true` if the row is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Extension trait for typed column extraction from rows.
pub trait DbRowExt: DbRow {
    /// Extracts and converts a column by index, with type safety.
    fn get_by_index_as<T>(&self, index: usize) -> DbResult<T> where T: TryFrom<DbValue, Error = DbError> {
        let value = self.get_by_index(index).ok_or(DbError::ColumnIndexOutOfBounds(index))?;
        T::try_from(value)
    }

    /// Extracts and converts a column by name, with type safety.
    fn get_by_name_as<T>(&self, name: &str) -> DbResult<T> where T: TryFrom<DbValue, Error = DbError> {
        let value = self.get_by_name(name).ok_or_else(|| DbError::ColumnNotFound(name.to_string()))?;
        T::try_from(value)
    }
}

impl<T: DbRow + ?Sized> DbRowExt for T {}
