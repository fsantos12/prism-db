//! Benchmark tests — skipped by default.
//!
//! Run with:
//! ```sh
//! cargo test -p prism-db-test-sqlite --test bench -- --include-ignored
//! ```

use prism_db_test::{bench_crud, bench_orm};
use prism_db_test_sqlite::SqliteTestContext;

async fn ctx() -> SqliteTestContext {
    SqliteTestContext::new().await
}

#[tokio::test] #[ignore = "benchmark"] async fn b_crud_insert() { bench_crud::bench_crud_insert(&ctx().await).await.unwrap(); }
#[tokio::test] #[ignore = "benchmark"] async fn b_crud_find()   { bench_crud::bench_crud_find(&ctx().await).await.unwrap(); }
#[tokio::test] #[ignore = "benchmark"] async fn b_crud_update() { bench_crud::bench_crud_update(&ctx().await).await.unwrap(); }
#[tokio::test] #[ignore = "benchmark"] async fn b_crud_delete() { bench_crud::bench_crud_delete(&ctx().await).await.unwrap(); }

#[tokio::test] #[ignore = "benchmark"] async fn b_orm_insert()  { bench_orm::bench_orm_insert(&ctx().await).await.unwrap(); }
#[tokio::test] #[ignore = "benchmark"] async fn b_orm_find()    { bench_orm::bench_orm_find(&ctx().await).await.unwrap(); }
#[tokio::test] #[ignore = "benchmark"] async fn b_orm_update()  { bench_orm::bench_orm_update(&ctx().await).await.unwrap(); }
#[tokio::test] #[ignore = "benchmark"] async fn b_orm_delete()  { bench_orm::bench_orm_delete(&ctx().await).await.unwrap(); }
