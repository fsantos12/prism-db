//! UPDATE query builder.

use async_trait::async_trait;

use crate::{query::{FilterBuilder, FilterDefinition}, types::{DbResult, DbValue}};

/// An UPDATE query builder.
///
/// Specify column updates and filter conditions to target specific rows.
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
    /// Creates a new UPDATE query for the given table.
    pub fn new(table: impl Into<String>) -> Self {
        Self {
            table: table.into(),
            updates: Vec::new(),
            filters: FilterDefinition::new(),
        }
    }

    /// Specifies a column update.
    pub fn set<F: Into<String>, V: Into<DbValue>>(mut self, field: F, value: V) -> Self {
        self.updates.push((field.into(), value.into()));
        self
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

/// Compiled, ready-to-execute UPDATE query.
#[async_trait]
pub trait PreparedUpdateQuery: Send + Sync {
    /// Executes the query and returns the number of modified rows.
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