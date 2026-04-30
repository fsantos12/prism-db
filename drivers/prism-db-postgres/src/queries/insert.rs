use async_trait::async_trait;
use prism_db_core::{query::{InsertQuery, PreparedInsertQuery}, types::{DbError, DbResult, DbValue}};
use crate::{driver::executor::PostgresExecutor, queries::binders::bind_values};

pub(crate) struct PostgresPreparedInsertQuery<'a> {
    executor: &'a PostgresExecutor,
    sql: String,
    parameters: Vec<DbValue>,
}

impl<'a> PostgresPreparedInsertQuery<'a> {
    pub(crate) fn new(executor: &'a PostgresExecutor, query: InsertQuery) -> Self {
        let columns: Vec<String> = query.values[0].iter().map(|(col, _)| col.clone()).collect();
        let mut sql = String::with_capacity(128);

        sql.push_str("INSERT INTO ");
        sql.push_str(&query.table);
        sql.push_str(" (");
        sql.push_str(&columns.join(", "));
        sql.push_str(") VALUES ");

        let total_rows = query.values.len();
        let columns_per_row = columns.len();

        let mut parameters = Vec::with_capacity(total_rows * columns_per_row);
        let mut row_placeholders = Vec::with_capacity(total_rows);
        let mut counter = 1usize;

        for row in query.values {
            let placeholders: Vec<String> = (0..columns_per_row).map(|_| {
                let p = format!("${}", counter);
                counter += 1;
                p
            }).collect();
            row_placeholders.push(format!("({})", placeholders.join(", ")));
            for (_, value) in row {
                parameters.push(value);
            }
        }

        sql.push_str(&row_placeholders.join(", "));
        Self { executor, sql, parameters }
    }
}

#[async_trait]
impl PreparedInsertQuery for PostgresPreparedInsertQuery<'_> {
    async fn execute(&self) -> DbResult<u64> {
        let mut query = sqlx::query(&self.sql);
        query = bind_values(query, &self.parameters);
        let result = self.executor.execute(query)
            .await
            .map_err(DbError::driver)?;
        Ok(result.rows_affected())
    }
}
