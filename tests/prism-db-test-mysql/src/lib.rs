use std::sync::Arc;

use prism_db::{DbContext, MySqlDriver};
use prism_db::types::DbResult;
use prism_db_test::PrismTestContext;

const DEFAULT_URL: &str = "mysql://root:root@localhost:3306/prism-db-tests";

/// Test context backed by a MySQL database.
///
/// Falls back to `mysql://root@localhost/prism-db-tests` when
/// `MYSQL_URL` is not set.
///
/// # Warning
/// All tests share the same `test_users` table. Run with `-- --test-threads=1`
/// to prevent parallel tests from interfering with each other.
pub struct MySqlTestContext {
    ctx: DbContext,
    driver: Arc<MySqlDriver>,
}

impl MySqlTestContext {
    /// Connects to the MySQL database at `url`.
    pub async fn new(url: &str) -> Self {
        let driver = Arc::new(
            MySqlDriver::connect(url)
                .await
                .expect("failed to connect to MySQL"),
        );
        let ctx = DbContext::new(driver.clone());
        Self { ctx, driver }
    }
}

impl PrismTestContext for MySqlTestContext {
    fn get_context(&self) -> &DbContext {
        &self.ctx
    }

    /// Drops and recreates `test_users` with the MySQL schema.
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

/// Returns the MySQL connection URL from `MYSQL_URL`, falling back to
/// `mysql://root@localhost/prism-db-tests`.
pub fn db_url() -> String {
    std::env::var("MYSQL_URL").unwrap_or_else(|_| DEFAULT_URL.to_string())
}
