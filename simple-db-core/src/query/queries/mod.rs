mod find;
mod insert;
mod update;
mod delete;

pub use find::{FindQuery, PreparedFindQuery};
pub use insert::{InsertQuery, PreparedInsertQuery};
pub use update::{UpdateQuery, PreparedUpdateQuery};
pub use delete::{DeleteQuery, PreparedDeleteQuery};

pub struct Query;

impl Query {
    pub fn find(table: impl Into<String>) -> FindQuery {
        FindQuery::new(table)
    }

    pub fn insert(table: impl Into<String>) -> InsertQuery {
        InsertQuery::new(table)
    }

    pub fn update(table: impl Into<String>) -> UpdateQuery {
        UpdateQuery::new(table)
    }

    pub fn delete(table: impl Into<String>) -> DeleteQuery {
        DeleteQuery::new(table)
    }
}