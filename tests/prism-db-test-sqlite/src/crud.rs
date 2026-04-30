use prism_db_test::crud;
use prism_db_test_sqlite::SqliteTestContext;

async fn ctx() -> SqliteTestContext {
    SqliteTestContext::new().await
}

#[tokio::test] async fn crud_insert()       { crud::test_crud_insert(&ctx().await).await.unwrap(); }
#[tokio::test] async fn crud_find()         { crud::test_crud_find(&ctx().await).await.unwrap(); }
#[tokio::test] async fn crud_update()       { crud::test_crud_update(&ctx().await).await.unwrap(); }
#[tokio::test] async fn crud_delete()       { crud::test_crud_delete(&ctx().await).await.unwrap(); }
#[tokio::test] async fn crud_bulk()         { crud::test_crud_bulk(&ctx().await).await.unwrap(); }
#[tokio::test] async fn crud_projection()   { crud::test_crud_projection(&ctx().await).await.unwrap(); }
#[tokio::test] async fn crud_sort()         { crud::test_crud_sort(&ctx().await).await.unwrap(); }
#[tokio::test] async fn crud_group()        { crud::test_crud_group(&ctx().await).await.unwrap(); }
#[tokio::test] async fn crud_limit_offset() { crud::test_crud_limit_offset(&ctx().await).await.unwrap(); }
