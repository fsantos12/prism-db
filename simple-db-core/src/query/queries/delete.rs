//! Delete query builder and trait for database record deletion operations.

use async_trait::async_trait;

use crate::{query::{FilterBuilder, FilterDefinition}, types::DbResult};

/// Represents a query to delete records from a table.
///
/// This query builder allows you to construct a DELETE operation using a fluent API.
/// You can add filter conditions to narrow down which records should be deleted.
///
/// # Example
/// ```ignore
/// let query = DeleteQuery::new("users")
///     .with_filter_builder(|f| f.eq("status", "inactive"));
/// ```
#[derive(Debug, Clone)]
pub struct DeleteQuery {
    /// The name of the table to delete from.
    pub table: String,
    /// The conditions to determine which records are deleted.
    pub filters: FilterDefinition,
}

impl DeleteQuery {
    /// Creates a new `DeleteQuery` for the specified table.
    pub fn new(table: impl Into<String>) -> Self {
        Self {
            table: table.into(),
            filters: FilterDefinition::new(),
        }
    }

    /// Adds pre-constructed filter conditions to this query.
    pub fn filter(mut self, filters: FilterDefinition) -> Self {
        self.filters.extend(filters);
        self
    }

    /// Builds filter conditions using a closure and adds them to this query.
    pub fn with_filter_builder<F>(mut self, build: F) -> Self
    where F: FnOnce(FilterBuilder) -> FilterBuilder,
    {
        self.filters.extend(build(FilterBuilder::new()).build());
        self
    }
}

/// Driver-specific representation of a compiled `DeleteQuery`.
///
/// This trait is implemented by database drivers (SQLite, PostgreSQL, MySQL)
/// to execute the query in their respective SQL dialects.
#[async_trait]
pub trait PreparedDeleteQuery: Send + Sync {
    /// Executes the delete query and returns the number of deleted rows.
    async fn execute(&self) -> DbResult<u64>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delete_query_new() {
        let query = DeleteQuery::new("users");
        assert_eq!(query.table, "users");
    }

    #[test]
    fn test_delete_query_with_filter_builder() {
        let query = DeleteQuery::new("orders")
            .with_filter_builder(|f| f.eq("status", "cancelled"));
        assert_eq!(query.table, "orders");
        // Filter is built, structure verified by builder tests
    }
}