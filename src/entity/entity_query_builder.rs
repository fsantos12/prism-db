use std::marker::PhantomData;

use crate::{entity::{DbContext, DbEntity, DbEntityModel}, query::{Query, filters::FilterDefinition, groups::GroupDefinition, projections::ProjectionDefinition, sorts::SortDefinition}, types::DbError};

/// A strongly-typed query builder that returns tracked DbEntities
pub struct EntityQueryBuilder<'a, T: DbEntityModel> {
    context: &'a DbContext,
    projections: ProjectionDefinition,
    filters: FilterDefinition,
    sorts: SortDefinition,
    groups: GroupDefinition,
    limit: Option<usize>,
    offset: Option<usize>,
    _marker: PhantomData<T>,
}

impl<'a, T: DbEntityModel> EntityQueryBuilder<'a, T> {
    pub fn new(context: &'a DbContext) -> Self {
        Self {
            context,
            projections: ProjectionDefinition::empty(),
            filters: FilterDefinition::empty(),
            sorts: SortDefinition::empty(),
            groups: GroupDefinition::empty(),
            limit: None,
            offset: None,
            _marker: PhantomData,
        }
    }

    pub fn project(mut self, projections: ProjectionDefinition) -> Self {
        self.projections = projections;
        self
    }

    pub fn filter(mut self, filters: FilterDefinition) -> Self {
        self.filters = filters;
        self
    }

    pub fn order_by(mut self, sorts: SortDefinition) -> Self {
        self.sorts = sorts;
        self
    }

    pub fn group_by(mut self, groups: GroupDefinition) -> Self {
        self.groups = groups;
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

    /// Executes the query and returns a list of tracked DbEntities
    pub async fn execute(self) -> Result<Vec<DbEntity<T>>, DbError> {
        let mut query = Query::find(T::collection_name()).
            project(self.projections).
            filter(self.filters).
            order_by(self.sorts).
            group_by(self.groups);

        if let Some(l) = self.limit { query = query.limit(l); }
        if let Some(o) = self.offset { query = query.offset(o); }

        let rows = self.context.driver.find(query).await?;

        let mut entities = Vec::new();
        for row in rows {
            let snapshot = row.clone();
            let model = T::from_db_row(row)?;
            entities.push(DbEntity::from_snapshot(model, snapshot));
        }

        Ok(entities)
    }
}