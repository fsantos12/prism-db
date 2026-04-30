use std::sync::Arc;

use prism_db::{DbContext, PostgresDriver};
use prism_db::types::DbResult;
use prism_db_test::PrismTestContext;

const DEFAULT_URL: &str = "postgres://postgres:postgres@localhost:5432/prism-db-tests";

/// Test context backed by a PostgreSQL database.
///
/// Falls back to `postgres://postgres@localhost/prism-db-tests` when
/// `POSTGRES_URL` is not set.
///
/// # Warning
/// All tests share the same `test_users` table. Run with `-- --test-threads=1`
/// to prevent parallel tests from interfering with each other.
pub struct PostgresTestContext {
    ctx: DbContext,
    driver: Arc<PostgresDriver>,
}

impl PostgresTestContext {
    /// Connects to the PostgreSQL database at `url`.
    pub async fn new(url: &str) -> Self {
        let driver = Arc::new(
            PostgresDriver::connect(url)
                .await
                .expect("failed to connect to PostgreSQL"),
        );
        let ctx = DbContext::new(driver.clone());
        Self { ctx, driver }
    }
}

impl PrismTestContext for PostgresTestContext {
    fn get_context(&self) -> &DbContext {
        &self.ctx
    }

    /// Drops and recreates `test_users` with the PostgreSQL schema.
    ///
    /// ```sql
    /// DROP TABLE IF EXISTS test_users;
    /// CREATE TABLE test_users (
    ///     id    BIGINT  PRIMARY KEY,
    ///     name  TEXT    NOT NULL,
    ///     email TEXT    NOT NULL,
    ///     age   INTEGER NOT NULL
    /// );
    /// ```
    async fn prepare_database(&self) -> DbResult<()> {
        self.driver.execute_raw(
            "CREATE TABLE IF NOT EXISTS test_users (
                id    BIGINT  PRIMARY KEY,
                name  TEXT    NOT NULL,
                email TEXT    NOT NULL,
                age   INTEGER NOT NULL
            )",
        ).await?;
        self.driver.execute_raw("DELETE FROM test_users").await
    }
}

/// Returns the PostgreSQL connection URL from `POSTGRES_URL`, falling back to
/// `postgres://postgres@localhost/prism-db-tests`.
pub fn db_url() -> String {
    std::env::var("POSTGRES_URL").unwrap_or_else(|_| DEFAULT_URL.to_string())
}
