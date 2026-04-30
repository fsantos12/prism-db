use async_trait::async_trait;
use prism_db_core::{query::{FindQuery, PreparedFindQuery}, types::{DbCursor, DbError, DbResult, DbValue}};
use crate::{builders::{filters::compile_filters, groups::compile_groups, projections::compile_projections, sorts::compile_sorts}, driver::executor::SqliteExecutor, queries::binders::bind_values, types::cursor::SqliteDbCursor};

pub(crate) struct SqlitePreparedFindQuery<'a> {
    executor: &'a SqliteExecutor,
    sql: String,
    parameters: Vec<DbValue>
}

impl<'a> SqlitePreparedFindQuery<'a> {
    pub(crate) fn new(executor: &'a SqliteExecutor, query: FindQuery) -> Self {
        let (filter_sql, parameters) = compile_filters(&query.filters);
        let proj_sql = compile_projections(&query.projections);
        let group_sql = compile_groups(&query.groups);
        let sort_sql = compile_sorts(&query.sorts);

        let capacity = 64 + query.table.len() + proj_sql.len() + filter_sql.len() + group_sql.len() + sort_sql.len();
        let mut sql = String::with_capacity(capacity);

        sql.push_str("SELECT ");
        if proj_sql.is_empty() {
            sql.push('*');
        } else {
            sql.push_str(&proj_sql);
        }

        sql.push_str(" FROM ");
        sql.push_str(&query.table);

        if !filter_sql.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&filter_sql);
        }

        if !group_sql.is_empty() {
            sql.push_str(" GROUP BY ");
            sql.push_str(&group_sql);
        }

        if !sort_sql.is_empty() {
            sql.push_str(" ORDER BY ");
            sql.push_str(&sort_sql);
        }

        if let Some(limit) = query.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }
        
        if let Some(offset) = query.offset {
            sql.push_str(&format!(" OFFSET {}", offset));
        }

        Self { executor, sql, parameters }
    }
}

#[async_trait]
impl PreparedFindQuery for SqlitePreparedFindQuery<'_> {
    async fn execute(&self) -> DbResult<Box<dyn DbCursor>> {
        let mut query = sqlx::query(&self.sql);
        query = bind_values(query, &self.parameters);
        let result = self.executor.fetch_all(query)
            .await
            .map_err(DbError::driver)?;
        let stream = Box::pin(futures::stream::iter(result.into_iter().map(Ok)));
        Ok(Box::new(SqliteDbCursor::new(stream)))
    }
}
