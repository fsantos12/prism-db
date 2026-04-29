use async_trait::async_trait;

use crate::{query::{FilterBuilder, FilterDefinition}, types::{DbResult, DbValue}};

#[derive(Debug, Clone)]
pub struct UpdateQuery {
    pub table: String,
    pub updates: Vec<(String, DbValue)>,
    pub filters: FilterDefinition,
}

impl UpdateQuery {
    pub fn new(table: impl Into<String>) -> Self {
        Self {
            table: table.into(),
            updates: Vec::new(),
            filters: FilterDefinition::new(),
        }
    }

    pub fn set<F: Into<String>, V: Into<DbValue>>(mut self, field: F, value: V) -> Self {
        self.updates.push((field.into(), value.into()));
        self
    }

    pub fn filter(mut self, filters: FilterDefinition) -> Self {
        self.filters.extend(filters);
        self
    }

    pub fn with_filter_builder<F>(mut self, build: F) -> Self
    where F: FnOnce(FilterBuilder) -> FilterBuilder,
    {
        self.filters.extend(build(FilterBuilder::new()).build());
        self
    }
}

#[async_trait]
pub trait PreparedUpdateQuery: Send + Sync {
    async fn execute(&self) -> DbResult<u64>;
}