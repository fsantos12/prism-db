//! PostgreSQL CRUD integration tests.
//!
//! Defaults to `postgres://postgres@localhost/prism-db-tests`.
//! Override with the `POSTGRES_URL` environment variable.
//!
//! Run with `-- --test-threads=1` to avoid table conflicts from parallel tests.

use prism_db_test::crud;
use prism_db_test_postgres::{PostgresTestContext, db_url};
use serial_test::serial;

macro_rules! pg_test {
    ($test_fn:path) => {{
        $test_fn(&PostgresTestContext::new(&db_url()).await).await.unwrap();
    }};
}

#[tokio::test] #[serial] async fn crud_insert()       { pg_test!(crud::test_crud_insert); }
#[tokio::test] #[serial] async fn crud_find()         { pg_test!(crud::test_crud_find); }
#[tokio::test] #[serial] async fn crud_update()       { pg_test!(crud::test_crud_update); }
#[tokio::test] #[serial] async fn crud_delete()       { pg_test!(crud::test_crud_delete); }
#[tokio::test] #[serial] async fn crud_bulk()         { pg_test!(crud::test_crud_bulk); }
#[tokio::test] #[serial] async fn crud_projection()   { pg_test!(crud::test_crud_projection); }
#[tokio::test] #[serial] async fn crud_sort()         { pg_test!(crud::test_crud_sort); }
#[tokio::test] #[serial] async fn crud_group()        { pg_test!(crud::test_crud_group); }
#[tokio::test] #[serial] async fn crud_limit_offset() { pg_test!(crud::test_crud_limit_offset); }
