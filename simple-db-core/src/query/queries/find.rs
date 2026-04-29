use async_trait::async_trait;

use crate::{query::{FilterBuilder, FilterDefinition, GroupBuilder, GroupDefinition, ProjectionBuilder, ProjectionDefinition, SortBuilder, SortDefinition}, types::{DbCursor, DbResult}};

#[derive(Debug, Clone)]
pub struct FindQuery {
    pub table: String,
    pub projections: ProjectionDefinition,
    pub filters: FilterDefinition,
    pub sorts: SortDefinition,
    pub groups: GroupDefinition,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

impl FindQuery {
    pub fn new(table: impl Into<String>) -> Self {
        Self {
            table: table.into(),
            projections: ProjectionDefinition::new(),
            filters: FilterDefinition::new(),
            sorts: SortDefinition::new(),
            groups: GroupDefinition::new(),
            limit: None,
            offset: None,
        }
    }

    pub fn project(mut self, projections: ProjectionDefinition) -> Self {
        self.projections.extend(projections);
        self
    }

    pub fn with_projection_builder<F>(mut self, build: F) -> Self
    where F: FnOnce(ProjectionBuilder) -> ProjectionBuilder,
    {
        self.projections.extend(build(ProjectionBuilder::new()).build());
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

    pub fn order_by(mut self, sorts: SortDefinition) -> Self {
        self.sorts.extend(sorts);
        self
    }

    pub fn with_sort_builder<F>(mut self, build: F) -> Self
    where F: FnOnce(SortBuilder) -> SortBuilder,
    {
        self.sorts.extend(build(SortBuilder::new()).build());
        self
    }

    pub fn group_by(mut self, groups: GroupDefinition) -> Self {
        self.groups.extend(groups);
        self
    }

    pub fn with_group_builder<F>(mut self, build: F) -> Self
    where F: FnOnce(GroupBuilder) -> GroupBuilder,
    {
        self.groups.extend(build(GroupBuilder::new()).build());
        self
    }

    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }
}

#[async_trait]
pub trait PreparedFindQuery: Send + Sync {
    async fn execute(&self) -> DbResult<Box<dyn DbCursor>>;
}