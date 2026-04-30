use async_trait::async_trait;
use simple_db_core::{query::{DeleteQuery, PreparedDeleteQuery}, types::{DbError, DbResult, DbValue}};
use crate::{builders::filters::compile_filters, driver::executor::PostgresExecutor, queries::binders::bind_values};

pub(crate) struct PostgresPreparedDeleteQuery<'a> {
    executor: &'a PostgresExecutor,
    sql: String,
    parameters: Vec<DbValue>,
}

impl<'a> PostgresPreparedDeleteQuery<'a> {
    pub(crate) fn new(executor: &'a PostgresExecutor, query: DeleteQuery) -> Self {
        let mut counter = 1usize;
        let (filter_sql, parameters) = compile_filters(&query.filters, &mut counter);

        let mut sql = String::with_capacity(19 + query.table.len() + filter_sql.len());
        sql.push_str("DELETE FROM ");
        sql.push_str(&query.table);
        if !filter_sql.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&filter_sql);
        }

        Self { executor, sql, parameters }
    }
}

#[async_trait]
impl PreparedDeleteQuery for PostgresPreparedDeleteQuery<'_> {
    async fn execute(&self) -> DbResult<u64> {
        let mut query = sqlx::query(&self.sql);
        query = bind_values(query, &self.parameters);
        let result = self.executor.execute(query)
            .await
            .map_err(DbError::driver)?;
        Ok(result.rows_affected())
    }
}
