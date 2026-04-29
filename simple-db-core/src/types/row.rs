use crate::types::{DbError, DbResult, DbValue};

/// A single row returned from a database query.
///
/// Columns can be accessed by zero-based index or by name. Both methods return an
/// owned [`DbValue`] cloned from the underlying storage.
pub trait DbRow: Send + Sync {
    /// Returns the value at the given column index, or `None` if out of range.
    fn get_by_index(&self, index: usize) -> Option<DbValue>;

    /// Returns the value for the column with the given name, or `None` if not found.
    fn get_by_name(&self, name: &str) -> Option<DbValue>;

    /// Returns the number of columns in this row.
    fn len(&self) -> usize;

    /// Returns `true` if this row has no columns.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Extension methods on [`DbRow`] for typed column extraction.
pub trait DbRowExt: DbRow {
    /// Extracts the value at `index` and converts it to `T`, returning an error on
    /// out-of-bounds access or type mismatch.
    fn get_by_index_as<T>(&self, index: usize) -> DbResult<T> where T: TryFrom<DbValue, Error = DbError> {
        let value = self.get_by_index(index).ok_or(DbError::ColumnIndexOutOfBounds(index))?;
        T::try_from(value)
    }

    /// Extracts the value for column `name` and converts it to `T`, returning an error if
    /// the column is missing or the type does not match.
    fn get_by_name_as<T>(&self, name: &str) -> DbResult<T> where T: TryFrom<DbValue, Error = DbError> {
        let value = self.get_by_name(name).ok_or_else(|| DbError::ColumnNotFound(name.to_string()))?;
        T::try_from(value)
    }
}

impl<T: DbRow + ?Sized> DbRowExt for T {}
