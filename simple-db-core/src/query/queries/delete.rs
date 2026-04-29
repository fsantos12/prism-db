//! DELETE query builder.

use async_trait::async_trait;

use crate::{query::{FilterBuilder, FilterDefinition}, types::DbResult};

/// A DELETE query builder.
///
/// Filter conditions determine which rows are deleted.
#[derive(Debug, Clone)]
pub struct DeleteQuery {
    /// The name of the table to delete from.
    pub table: String,
    /// The conditions to determine which records are deleted.
    pub filters: FilterDefinition,
}

impl DeleteQuery {
    /// Creates a new DELETE query for the given table.
    pub fn new(table: impl Into<String>) -> Self {
        Self {
            table: table.into(),
            filters: FilterDefinition::new(),
        }
    }

    /// Appends filter conditions.
    pub fn filter(mut self, filters: FilterDefinition) -> Self {
        self.filters.extend(filters);
        self
    }

    /// Builds and appends filter conditions via a closure.
    pub fn with_filter_builder<F>(mut self, build: F) -> Self
    where F: FnOnce(FilterBuilder) -> FilterBuilder,
    {
        self.filters.extend(build(FilterBuilder::new()).build());
        self
    }
}

/// Compiled, ready-to-execute DELETE query.
#[async_trait]
pub trait PreparedDeleteQuery: Send + Sync {
    /// Executes the query and returns the number of deleted rows.
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