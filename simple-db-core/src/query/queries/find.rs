use async_trait::async_trait;

use crate::{query::{FilterBuilder, FilterDefinition, GroupBuilder, GroupDefinition, ProjectionBuilder, ProjectionDefinition, SortBuilder, SortDefinition}, types::{DbCursor, DbResult}};

/// A `SELECT` query with optional projections, filters, sorting, grouping, limit, and offset.
///
/// Build one via the fluent methods or the `with_*_builder` closures, then hand it to a
/// [`DbExecutor`](crate::driver::DbExecutor) to run it.
#[derive(Debug, Clone)]
pub struct FindQuery {
    /// Target table name.
    pub table: String,
    /// Columns / aggregates to select. Empty means `SELECT *`.
    pub projections: ProjectionDefinition,
    /// `WHERE` predicates. Multiple entries are combined with `AND`.
    pub filters: FilterDefinition,
    /// `ORDER BY` directives.
    pub sorts: SortDefinition,
    /// `GROUP BY` columns.
    pub groups: GroupDefinition,
    /// `LIMIT` value.
    pub limit: Option<usize>,
    /// `OFFSET` value.
    pub offset: Option<usize>,
}

impl FindQuery {
    /// Creates a new query targeting `table` with all optional parts unset.
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

    /// Appends a pre-built [`ProjectionDefinition`] to the `SELECT` list.
    pub fn project(mut self, projections: ProjectionDefinition) -> Self {
        self.projections.extend(projections);
        self
    }

    /// Builds the projection list via a closure and appends it.
    pub fn with_projection_builder<F>(mut self, build: F) -> Self
    where F: FnOnce(ProjectionBuilder) -> ProjectionBuilder,
    {
        self.projections.extend(build(ProjectionBuilder::new()).build());
        self
    }

    /// Appends a pre-built [`FilterDefinition`] to the `WHERE` clause.
    pub fn filter(mut self, filters: FilterDefinition) -> Self {
        self.filters.extend(filters);
        self
    }

    /// Builds the filter list via a closure and appends it.
    pub fn with_filter_builder<F>(mut self, build: F) -> Self
    where F: FnOnce(FilterBuilder) -> FilterBuilder,
    {
        self.filters.extend(build(FilterBuilder::new()).build());
        self
    }

    /// Appends a pre-built [`SortDefinition`] to the `ORDER BY` clause.
    pub fn order_by(mut self, sorts: SortDefinition) -> Self {
        self.sorts.extend(sorts);
        self
    }

    /// Builds the sort list via a closure and appends it.
    pub fn with_sort_builder<F>(mut self, build: F) -> Self
    where F: FnOnce(SortBuilder) -> SortBuilder,
    {
        self.sorts.extend(build(SortBuilder::new()).build());
        self
    }

    /// Appends a pre-built [`GroupDefinition`] to the `GROUP BY` clause.
    pub fn group_by(mut self, groups: GroupDefinition) -> Self {
        self.groups.extend(groups);
        self
    }

    /// Builds the group list via a closure and appends it.
    pub fn with_group_builder<F>(mut self, build: F) -> Self
    where F: FnOnce(GroupBuilder) -> GroupBuilder,
    {
        self.groups.extend(build(GroupBuilder::new()).build());
        self
    }

    /// Sets the `LIMIT`.
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Sets the `OFFSET`.
    pub fn offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }
}

/// A compiled, ready-to-execute version of a [`FindQuery`].
///
/// Obtained from [`DbExecutor::prepare_find`](crate::driver::DbExecutor::prepare_find).
#[async_trait]
pub trait PreparedFindQuery: Send + Sync {
    /// Executes the prepared query and returns a cursor over the result rows.
    async fn execute(&self) -> DbResult<Box<dyn DbCursor>>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::builders::{Filter, Projection, Sort};

    #[test]
    fn new_creates_empty_query() {
        let q = FindQuery::new("users");
        assert_eq!(q.table, "users");
        assert!(q.projections.is_empty());
        assert!(q.filters.is_empty());
        assert!(q.sorts.is_empty());
        assert!(q.groups.is_empty());
        assert_eq!(q.limit, None);
        assert_eq!(q.offset, None);
    }

    #[test]
    fn project_appends_projections() {
        let projs = ProjectionDefinition::from_iter([Projection::Field("id".into()), Projection::CountAll]);
        let q = FindQuery::new("t").project(projs);
        assert_eq!(q.projections.len(), 2);
    }

    #[test]
    fn with_projection_builder_appends_projections() {
        let q = FindQuery::new("t")
            .with_projection_builder(|b| b.field("id").count_all());
        assert_eq!(q.projections.len(), 2);
    }

    #[test]
    fn filter_appends_filters() {
        let filters = FilterDefinition::from_iter([Filter::IsNull("col".into())]);
        let q = FindQuery::new("t").filter(filters);
        assert_eq!(q.filters.len(), 1);
    }

    #[test]
    fn with_filter_builder_appends_filters() {
        let q = FindQuery::new("t")
            .with_filter_builder(|b| b.eq("id", 1i32).gt("score", 0i32));
        assert_eq!(q.filters.len(), 2);
    }

    #[test]
    fn order_by_appends_sorts() {
        let sorts = SortDefinition::from_iter([Sort::Asc("name".into())]);
        let q = FindQuery::new("t").order_by(sorts);
        assert_eq!(q.sorts.len(), 1);
    }

    #[test]
    fn with_sort_builder_appends_sorts() {
        let q = FindQuery::new("t")
            .with_sort_builder(|b| b.asc("name").desc("age"));
        assert_eq!(q.sorts.len(), 2);
    }

    #[test]
    fn group_by_appends_groups() {
        let groups = GroupDefinition::from_iter(["dept".into()]);
        let q = FindQuery::new("t").group_by(groups);
        assert_eq!(q.groups.len(), 1);
    }

    #[test]
    fn with_group_builder_appends_groups() {
        let q = FindQuery::new("t")
            .with_group_builder(|b| b.field("dept").field("region"));
        assert_eq!(q.groups.len(), 2);
    }

    #[test]
    fn limit_and_offset_are_set() {
        let q = FindQuery::new("t").limit(10).offset(20);
        assert_eq!(q.limit, Some(10));
        assert_eq!(q.offset, Some(20));
    }

    #[test]
    fn multiple_calls_accumulate() {
        let q = FindQuery::new("t")
            .with_filter_builder(|b| b.eq("a", 1i32))
            .with_filter_builder(|b| b.eq("b", 2i32));
        assert_eq!(q.filters.len(), 2);
    }
}
