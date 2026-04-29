use crate::types::{DbError, DbResult, DbValue};

pub trait DbRow: Send + Sync {
    fn get_by_index(&self, index: usize) -> Option<DbValue>;
    fn get_by_name(&self, name: &str) -> Option<DbValue>;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub trait DbRowExt: DbRow {
    fn get_by_index_as<T>(&self, index: usize) -> DbResult<T> where T: TryFrom<DbValue, Error = DbError> {
        let value = self.get_by_index(index).ok_or(DbError::ColumnIndexOutOfBounds(index))?;
        T::try_from(value)
    }

    fn get_by_name_as<T>(&self, name: &str) -> DbResult<T> where T: TryFrom<DbValue, Error = DbError> {
        let value = self.get_by_name(name).ok_or_else(|| DbError::ColumnNotFound(name.to_string()))?;
        T::try_from(value)
    }
}

impl<T: DbRow + ?Sized> DbRowExt for T {}