use std::sync::Arc;

use prism_db::{DbContext, SqliteDriver};
use prism_db::types::DbResult;
use prism_db_test::PrismTestContext;

/// Test context backed by an in-memory SQLite database.
///
/// Each call to [`new`](SqliteTestContext::new) creates an independent in-memory
/// database, so tests are fully isolated without any cleanup between runs.
pub struct SqliteTestContext {
    ctx: DbContext,
    driver: Arc<SqliteDriver>,
}

impl SqliteTestContext {
    /// Creates a fresh in-memory SQLite context.
    pub async fn new() -> Self {
        let driver = Arc::new(
            SqliteDriver::connect("sqlite::memory:")
                .await
                .expect("failed to open in-memory SQLite database"),
        );
        let ctx = DbContext::new(driver.clone());
        Self { ctx, driver }
    }
}

impl PrismTestContext for SqliteTestContext {
    fn get_context(&self) -> &DbContext {
        &self.ctx
    }

    /// Drops and recreates `test_users` with the SQLite schema.
    ///
    /// ```sql
    /// DROP TABLE IF EXISTS test_users;
    /// CREATE TABLE test_users (
    ///     id    INTEGER PRIMARY KEY,
    ///     name  TEXT    NOT NULL,
    ///     email TEXT    NOT NULL,
    ///     age   INTEGER NOT NULL
    /// );
    /// ```
    async fn prepare_database(&self) -> DbResult<()> {
        self.driver.execute_raw(
            "CREATE TABLE IF NOT EXISTS test_users (
                id    INTEGER PRIMARY KEY,
                name  TEXT    NOT NULL,
                email TEXT    NOT NULL,
                age   INTEGER NOT NULL
            )",
        ).await
    }
}
