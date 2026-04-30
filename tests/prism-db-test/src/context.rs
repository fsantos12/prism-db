use prism_db::{DbContext, types::DbResult};

/// Test context that provides database access for prism-db integration tests.
///
/// Implementors supply a live [`DbContext`] and the logic to reset the schema
/// before each test run.
///
/// # Table Structure
///
/// Implementations must manage a `test_users` table with the following schema:
///
/// ```sql
/// DROP TABLE IF EXISTS test_users;
/// CREATE TABLE test_users (
///     id    INTEGER PRIMARY KEY,  -- unique row identifier
///     name  TEXT    NOT NULL,     -- display name
///     email TEXT    NOT NULL,     -- contact email address
///     age   INTEGER NOT NULL      -- age in years
/// );
/// ```
///
/// # Example (SQLite)
///
/// ```ignore
/// use std::sync::Arc;
/// use prism_db_core::DbContext;
/// use prism_db_core::types::DbResult;
/// use prism_db_sqlite::SqliteDriver;
/// use prism_db_test::PrismTestContext;
///
/// struct SqliteTestCtx { ctx: DbContext, driver: SqliteDriver }
///
/// impl PrismTestContext for SqliteTestCtx {
///     fn get_context(&self) -> &DbContext { &self.ctx }
///
///     async fn prepare_database(&self) -> DbResult<()> {
///         self.driver.execute_raw("DROP TABLE IF EXISTS test_users").await?;
///         self.driver.execute_raw(
///             "CREATE TABLE test_users (
///                  id    INTEGER PRIMARY KEY,
///                  name  TEXT    NOT NULL,
///                  email TEXT    NOT NULL,
///                  age   INTEGER NOT NULL
///             )"
///         ).await
///     }
/// }
/// ```
pub trait PrismTestContext: Send + Sync {
    /// Returns the shared database context used to execute queries.
    fn get_context(&self) -> &DbContext;

    /// Drops and recreates the `test_users` table for a clean test environment.
    ///
    /// RECOMMENDED: Call this at the beginning of **every** test to drop and
    /// recreate the table, preventing leftover data from causing false failures.
    ///
    /// # Recommended SQL (run in order)
    ///
    /// ```sql
    /// DROP TABLE IF EXISTS test_users;
    /// CREATE TABLE test_users (
    ///     id    INTEGER PRIMARY KEY,  -- unique row identifier
    ///     name  TEXT    NOT NULL,     -- display name
    ///     email TEXT    NOT NULL,     -- contact email address
    ///     age   INTEGER NOT NULL      -- age in years
    /// );
    /// ```
    fn prepare_database(&self) -> impl std::future::Future<Output = DbResult<()>> + Send;
}
