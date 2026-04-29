mod find;
mod insert;
mod update;
mod delete;

pub use find::{FindQuery, PreparedFindQuery};
pub use insert::{InsertQuery, PreparedInsertQuery};
pub use update::{UpdateQuery, PreparedUpdateQuery};
pub use delete::{DeleteQuery, PreparedDeleteQuery};

/// Convenience namespace for constructing query builders.
///
/// All methods return the corresponding query builder (`FindQuery`, `InsertQuery`, etc.).
///
/// # Example
///
/// ```ignore
/// let find_qry = Query::find("users");
/// let insert_qry = Query::insert("users");
/// let update_qry = Query::update("users");
/// let delete_qry = Query::delete("users");
/// ```
pub struct Query;

impl Query {
    /// Starts building a SELECT query for the given table.
    pub fn find(table: impl Into<String>) -> FindQuery {
        FindQuery::new(table)
    }

    /// Starts building an INSERT query for the given table.
    pub fn insert(table: impl Into<String>) -> InsertQuery {
        InsertQuery::new(table)
    }

    /// Starts building an UPDATE query for the given table.
    pub fn update(table: impl Into<String>) -> UpdateQuery {
        UpdateQuery::new(table)
    }

    /// Starts building a DELETE query for the given table.
    pub fn delete(table: impl Into<String>) -> DeleteQuery {
        DeleteQuery::new(table)
    }
}