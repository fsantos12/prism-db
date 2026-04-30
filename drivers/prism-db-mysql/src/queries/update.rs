use async_trait::async_trait;
use prism_db_core::{query::{UpdateQuery, PreparedUpdateQuery}, types::{DbError, DbResult, DbValue}};
use crate::{builders::filters::compile_filters, driver::executor::MySqlExecutor, queries::binders::bind_values};

pub(crate) struct MySqlPreparedUpdateQuery<'a> {
    executor: &'a MySqlExecutor,
    sql: String,
    parameters: Vec<DbValue>
}

impl<'a> MySqlPreparedUpdateQuery<'a> {
    pub(crate) fn new(executor: &'a MySqlExecutor, query: UpdateQuery) -> Self {
        let (filter_sql, mut filter_params) = compile_filters(&query.filters);

        let mut sql = String::with_capacity(128);
        let mut parameters = Vec::with_capacity(query.updates.len() + filter_params.len());

        sql.push_str("UPDATE ");
        sql.push_str(&query.table);
        sql.push_str(" SET ");

        let mut set_clauses = Vec::with_capacity(query.updates.len());
        for (field, value) in query.updates {
            set_clauses.push(format!("{} = ?", field));
            parameters.push(value);
        }
        sql.push_str(&set_clauses.join(", "));

        if !filter_sql.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&filter_sql);
            parameters.append(&mut filter_params);
        }

        Self { executor, sql, parameters }
    }
}

#[async_trait]
impl PreparedUpdateQuery for MySqlPreparedUpdateQuery<'_> {
    async fn execute(&self) -> DbResult<u64> {
        let mut query = sqlx::query(&self.sql);
        query = bind_values(query, &self.parameters);
        let result = self.executor.execute(query)
            .await
            .map_err(DbError::driver)?;
        Ok(result.rows_affected())
    }
}
