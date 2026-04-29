use async_trait::async_trait;

use crate::{query::{FilterBuilder, FilterDefinition}, types::DbResult};

#[derive(Debug, Clone)]
pub struct DeleteQuery {
    pub table: String,
    pub filters: FilterDefinition,
}

impl DeleteQuery {
    pub fn new(table: impl Into<String>) -> Self {
        Self {
            table: table.into(),
            filters: FilterDefinition::new(),
        }
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
pub trait PreparedDeleteQuery: Send + Sync {
    async fn execute(&self) -> DbResult<u64>;
}