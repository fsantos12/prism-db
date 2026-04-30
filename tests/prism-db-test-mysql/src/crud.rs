//! MySQL CRUD integration tests.
//!
//! Defaults to `mysql://root@localhost/prism-db-tests`.
//! Override with the `MYSQL_URL` environment variable.
//!
//! Run with `-- --test-threads=1` to avoid table conflicts from parallel tests.

use prism_db_test::crud;
use prism_db_test_mysql::{MySqlTestContext, db_url};
use serial_test::serial;

macro_rules! mysql_test {
    ($test_fn:path) => {{
        $test_fn(&MySqlTestContext::new(&db_url()).await).await.unwrap();
    }};
}

#[tokio::test] #[serial] async fn crud_insert()       { mysql_test!(crud::test_crud_insert); }
#[tokio::test] #[serial] async fn crud_find()         { mysql_test!(crud::test_crud_find); }
#[tokio::test] #[serial] async fn crud_update()       { mysql_test!(crud::test_crud_update); }
#[tokio::test] #[serial] async fn crud_delete()       { mysql_test!(crud::test_crud_delete); }
#[tokio::test] #[serial] async fn crud_bulk()         { mysql_test!(crud::test_crud_bulk); }
#[tokio::test] #[serial] async fn crud_projection()   { mysql_test!(crud::test_crud_projection); }
#[tokio::test] #[serial] async fn crud_sort()         { mysql_test!(crud::test_crud_sort); }
#[tokio::test] #[serial] async fn crud_group()        { mysql_test!(crud::test_crud_group); }
#[tokio::test] #[serial] async fn crud_limit_offset() { mysql_test!(crud::test_crud_limit_offset); }
