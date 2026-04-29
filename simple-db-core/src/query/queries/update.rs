//! Update query builder and trait for database record modification operations.

use async_trait::async_trait;

use crate::{query::{FilterBuilder, FilterDefinition}, types::{DbResult, DbValue}};

/// Represents a query to modify existing records in a table.
///
/// This query builder allows you to specify which columns to update and
/// which records to target via filter conditions.
///
/// # Example
/// ```ignore
/// let query = UpdateQuery::new("users")
///     .set("status", "active")
///     .with_filter_builder(|f| f.eq("id", 123));
/// ```
#[derive(Debug, Clone)]
pub struct UpdateQuery {
    /// The table whose records are being modified.
    pub table: String,
    /// Column-value pairs specifying the updates to apply.
    pub updates: Vec<(String, DbValue)>,
    /// Filter conditions to select which records are updated.
    pub filters: FilterDefinition,
}

impl UpdateQuery {
    /// Creates a new `UpdateQuery` for the specified table.
    pub fn new(table: impl Into<String>) -> Self {
        Self {
            table: table.into(),
            updates: Vec::new(),
            filters: FilterDefinition::new(),
        }
    }

    /// Adds a column update to the query.
    pub fn set<F: Into<String>, V: Into<DbValue>>(mut self, field: F, value: V) -> Self {
        self.updates.push((field.into(), value.into()));
        self
    }

    /// Adds pre-constructed filter conditions to the query.
    pub fn filter(mut self, filters: FilterDefinition) -> Self {
        self.filters.extend(filters);
        self
    }

    /// Builds filter conditions using a closure and adds them to the query.
    pub fn with_filter_builder<F>(mut self, build: F) -> Self
    where F: FnOnce(FilterBuilder) -> FilterBuilder,
    {
        self.filters.extend(build(FilterBuilder::new()).build());
        self
    }
}

/// Driver-specific representation of a compiled `UpdateQuery`.
///
/// This trait is implemented by database drivers to execute the query
/// in their respective SQL dialects.
#[async_trait]
pub trait PreparedUpdateQuery: Send + Sync {
    /// Executes the update query and returns the number of modified rows.
    async fn execute(&self) -> DbResult<u64>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_query_new() {
        let query = UpdateQuery::new("accounts");
        assert_eq!(query.table, "accounts");
        assert!(query.updates.is_empty());
    }

    #[test]
    fn test_update_query_set() {
        let query = UpdateQuery::new("users")
            .set("status", "verified")
            .set("updated_at", "2026-04-29");
        
        assert_eq!(query.updates.len(), 2);
        assert_eq!(query.updates[0].0, "status");
        assert_eq!(query.updates[1].0, "updated_at");
    }

    #[test]
    fn test_update_query_with_filter() {
        let query = UpdateQuery::new("users")
            .set("role", "admin")
            .with_filter_builder(|f| f.eq("id", 42));
        
        assert_eq!(query.table, "users");
        assert_eq!(query.updates.len(), 1);
    }
}