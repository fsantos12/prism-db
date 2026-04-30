mod find;
mod insert;
mod update;
mod delete;

pub use find::{FindQuery, PreparedFindQuery};
pub use insert::{InsertQuery, PreparedInsertQuery};
pub use update::{UpdateQuery, PreparedUpdateQuery};
pub use delete::{DeleteQuery, PreparedDeleteQuery};

/// Convenience constructors for query builders.
pub struct Query;

impl Query {
    /// Creates a SELECT query builder.
    pub fn find(table: impl Into<String>) -> FindQuery {
        FindQuery::new(table)
    }

    /// Creates an INSERT query builder.
    pub fn insert(table: impl Into<String>) -> InsertQuery {
        InsertQuery::new(table)
    }

    /// Creates an UPDATE query builder.
    pub fn update(table: impl Into<String>) -> UpdateQuery {
        UpdateQuery::new(table)
    }

    /// Creates a DELETE query builder.
    pub fn delete(table: impl Into<String>) -> DeleteQuery {
        DeleteQuery::new(table)
    }
}