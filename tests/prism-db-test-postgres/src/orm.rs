//! PostgreSQL ORM integration tests.
//!
//! Defaults to `postgres://postgres@localhost/prism-db-tests`.
//! Override with the `POSTGRES_URL` environment variable.
//!
//! Run with `-- --test-threads=1` to avoid table conflicts from parallel tests.

use prism_db_test::orm;
use prism_db_test_postgres::{PostgresTestContext, db_url};
use serial_test::serial;

macro_rules! pg_test {
    ($test_fn:path) => {{
        $test_fn(&PostgresTestContext::new(&db_url()).await).await.unwrap();
    }};
}

#[tokio::test] #[serial] async fn orm_insert()                { pg_test!(orm::test_orm_insert); }
#[tokio::test] #[serial] async fn orm_update()                { pg_test!(orm::test_orm_update); }
#[tokio::test] #[serial] async fn orm_delete()                { pg_test!(orm::test_orm_delete); }
#[tokio::test] #[serial] async fn orm_cursor_entity()         { pg_test!(orm::test_orm_cursor_entity); }
#[tokio::test] #[serial] async fn orm_no_update_if_unchanged(){ pg_test!(orm::test_orm_no_update_if_unchanged); }
#[tokio::test] #[serial] async fn orm_full_lifecycle()        { pg_test!(orm::test_orm_full_lifecycle); }
