//! MySQL ORM integration tests.
//!
//! Defaults to `mysql://root@localhost/prism-db-tests`.
//! Override with the `MYSQL_URL` environment variable.
//!
//! Run with `-- --test-threads=1` to avoid table conflicts from parallel tests.

use prism_db_test::orm;
use prism_db_test_mysql::{MySqlTestContext, db_url};
use serial_test::serial;

macro_rules! mysql_test {
    ($test_fn:path) => {{
        $test_fn(&MySqlTestContext::new(&db_url()).await).await.unwrap();
    }};
}

#[tokio::test] #[serial] async fn orm_insert()                { mysql_test!(orm::test_orm_insert); }
#[tokio::test] #[serial] async fn orm_update()                { mysql_test!(orm::test_orm_update); }
#[tokio::test] #[serial] async fn orm_delete()                { mysql_test!(orm::test_orm_delete); }
#[tokio::test] #[serial] async fn orm_cursor_entity()         { mysql_test!(orm::test_orm_cursor_entity); }
#[tokio::test] #[serial] async fn orm_no_update_if_unchanged(){ mysql_test!(orm::test_orm_no_update_if_unchanged); }
#[tokio::test] #[serial] async fn orm_full_lifecycle()        { mysql_test!(orm::test_orm_full_lifecycle); }
