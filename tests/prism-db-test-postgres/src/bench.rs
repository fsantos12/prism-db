//! PostgreSQL benchmark tests — skipped by default.
//!
//! Defaults to `postgres://postgres@localhost/prism-db-tests`.
//! Override with the `POSTGRES_URL` environment variable.
//!
//! Run with:
//! ```sh
//! cargo test -p prism-db-test-postgres --test bench -- --include-ignored --test-threads=1
//! ```

use prism_db_test::{bench_crud, bench_orm};
use prism_db_test_postgres::{PostgresTestContext, db_url};

macro_rules! pg_bench {
    ($test_fn:path) => {{
        $test_fn(&PostgresTestContext::new(&db_url()).await).await.unwrap();
    }};
}

#[tokio::test] #[ignore = "benchmark"] async fn b_crud_insert() { pg_bench!(bench_crud::bench_crud_insert); }
#[tokio::test] #[ignore = "benchmark"] async fn b_crud_find()   { pg_bench!(bench_crud::bench_crud_find); }
#[tokio::test] #[ignore = "benchmark"] async fn b_crud_update() { pg_bench!(bench_crud::bench_crud_update); }
#[tokio::test] #[ignore = "benchmark"] async fn b_crud_delete() { pg_bench!(bench_crud::bench_crud_delete); }

#[tokio::test] #[ignore = "benchmark"] async fn b_orm_insert()  { pg_bench!(bench_orm::bench_orm_insert); }
#[tokio::test] #[ignore = "benchmark"] async fn b_orm_find()    { pg_bench!(bench_orm::bench_orm_find); }
#[tokio::test] #[ignore = "benchmark"] async fn b_orm_update()  { pg_bench!(bench_orm::bench_orm_update); }
#[tokio::test] #[ignore = "benchmark"] async fn b_orm_delete()  { pg_bench!(bench_orm::bench_orm_delete); }
