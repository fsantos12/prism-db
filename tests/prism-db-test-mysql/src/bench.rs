//! MySQL benchmark tests — skipped by default.
//!
//! Defaults to `mysql://root@localhost/prism-db-tests`.
//! Override with the `MYSQL_URL` environment variable.
//!
//! Run with:
//! ```sh
//! cargo test -p prism-db-test-mysql --test bench -- --include-ignored --test-threads=1
//! ```

use prism_db_test::{bench_crud, bench_orm};
use prism_db_test_mysql::{MySqlTestContext, db_url};

macro_rules! mysql_bench {
    ($test_fn:path) => {{
        $test_fn(&MySqlTestContext::new(&db_url()).await).await.unwrap();
    }};
}

#[tokio::test] #[ignore = "benchmark"] async fn b_crud_insert() { mysql_bench!(bench_crud::bench_crud_insert); }
#[tokio::test] #[ignore = "benchmark"] async fn b_crud_find()   { mysql_bench!(bench_crud::bench_crud_find); }
#[tokio::test] #[ignore = "benchmark"] async fn b_crud_update() { mysql_bench!(bench_crud::bench_crud_update); }
#[tokio::test] #[ignore = "benchmark"] async fn b_crud_delete() { mysql_bench!(bench_crud::bench_crud_delete); }

#[tokio::test] #[ignore = "benchmark"] async fn b_orm_insert()  { mysql_bench!(bench_orm::bench_orm_insert); }
#[tokio::test] #[ignore = "benchmark"] async fn b_orm_find()    { mysql_bench!(bench_orm::bench_orm_find); }
#[tokio::test] #[ignore = "benchmark"] async fn b_orm_update()  { mysql_bench!(bench_orm::bench_orm_update); }
#[tokio::test] #[ignore = "benchmark"] async fn b_orm_delete()  { mysql_bench!(bench_orm::bench_orm_delete); }
