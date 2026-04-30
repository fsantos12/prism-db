//! # prism-db-test
//!
//! Reusable integration-test helpers for prism-db drivers.
//!
//! ## Usage
//!
//! 1. Implement [`PrismTestContext`] for your test setup type.
//! 2. Call the exported functions from your own `#[tokio::test]` blocks.
//!
//! ## Modules
//!
//! | Module | Contents |
//! |---|---|
//! | [`crud`] | Basic INSERT / SELECT / UPDATE / DELETE tests |
//! | [`orm`] | `DbEntity` lifecycle and cursor entity tests |
//! | [`bench_crud`] | CRUD benchmarks (10 000+ ops each) |
//! | [`bench_orm`] | ORM benchmarks (10 000+ ops each) |
//!
//! ## Example
//!
//! ```ignore
//! use prism_db_test::{PrismTestContext, crud, orm, bench_crud, bench_orm};
//!
//! #[tokio::test]
//! async fn run_crud_suite() {
//!     let ctx = MyTestContext::new().await;
//!     crud::test_crud_insert(&ctx).await.unwrap();
//!     crud::test_crud_find(&ctx).await.unwrap();
//!     crud::test_crud_update(&ctx).await.unwrap();
//!     crud::test_crud_delete(&ctx).await.unwrap();
//!     crud::test_crud_bulk(&ctx).await.unwrap();
//! }
//!
//! #[tokio::test]
//! async fn run_orm_suite() {
//!     let ctx = MyTestContext::new().await;
//!     orm::test_orm_insert(&ctx).await.unwrap();
//!     orm::test_orm_update(&ctx).await.unwrap();
//!     orm::test_orm_delete(&ctx).await.unwrap();
//!     orm::test_orm_cursor_entity(&ctx).await.unwrap();
//!     orm::test_orm_no_update_if_unchanged(&ctx).await.unwrap();
//!     orm::test_orm_full_lifecycle(&ctx).await.unwrap();
//! }
//! ```

pub mod context;
pub mod entity;
pub mod crud;
pub mod orm;
pub mod bench_crud;
pub mod bench_orm;

pub use context::PrismTestContext;
pub use entity::TestUser;
pub use prism_db::DbEntity;
