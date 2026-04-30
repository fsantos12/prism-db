use prism_db_test::orm;
use prism_db_test_sqlite::SqliteTestContext;

async fn ctx() -> SqliteTestContext {
    SqliteTestContext::new().await
}

#[tokio::test] async fn orm_insert()                { orm::test_orm_insert(&ctx().await).await.unwrap(); }
#[tokio::test] async fn orm_update()                { orm::test_orm_update(&ctx().await).await.unwrap(); }
#[tokio::test] async fn orm_delete()                { orm::test_orm_delete(&ctx().await).await.unwrap(); }
#[tokio::test] async fn orm_cursor_entity()         { orm::test_orm_cursor_entity(&ctx().await).await.unwrap(); }
#[tokio::test] async fn orm_no_update_if_unchanged(){ orm::test_orm_no_update_if_unchanged(&ctx().await).await.unwrap(); }
#[tokio::test] async fn orm_full_lifecycle()        { orm::test_orm_full_lifecycle(&ctx().await).await.unwrap(); }
