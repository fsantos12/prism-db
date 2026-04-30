use prism_db::{DbEntity, DbCursorEntityExt, driver::executor::DbExecutor, filter, query::FindQuery, types::DbResult};

use crate::{context::PrismTestContext, entity::TestUser};

fn user(id: i64, name: &str, email: &str, age: i32) -> TestUser {
    TestUser { id, name: name.to_string(), email: email.to_string(), age }
}

/// Tests the ORM INSERT path: `DbEntity::new` + `save()` issues an INSERT.
///
/// Asserts state transitions: Untracked → Tracked after first save.
pub async fn test_orm_insert(ctx: &impl PrismTestContext) -> DbResult<()> {
    ctx.prepare_database().await?;
    let db = ctx.get_context();

    let mut entity = DbEntity::new(user(1, "Alice", "alice@example.com", 30));
    assert!(entity.is_untracked(), "new entity should be Untracked");

    entity.save(db).await?;
    assert!(entity.is_tracked(), "entity should be Tracked after save");

    let mut cursor = db.find(FindQuery::new("test_users")).await?;
    let r = cursor.next().await?.expect("entity should exist in the database");
    assert_eq!(r.get_by_name("name").unwrap().cast::<String>().unwrap(), "Alice");
    assert!(cursor.next().await?.is_none());

    Ok(())
}

/// Tests the ORM UPDATE path: mutate via `get_mut()`, then `save()` issues an UPDATE.
///
/// Asserts only changed fields are persisted and the row is correct after re-read.
pub async fn test_orm_update(ctx: &impl PrismTestContext) -> DbResult<()> {
    ctx.prepare_database().await?;
    let db = ctx.get_context();

    let mut entity = DbEntity::new(user(1, "Alice", "alice@example.com", 30));
    entity.save(db).await?;

    entity.get_mut().name = "Alicia".to_string();
    entity.get_mut().age = 31;
    entity.save(db).await?;

    let mut cursor = db.find(
        FindQuery::new("test_users").filter(filter!(eq("id", 1i64)))
    ).await?;
    let r = cursor.next().await?.expect("updated entity should exist");
    assert_eq!(r.get_by_name("name").unwrap().cast::<String>().unwrap(), "Alicia");
    assert_eq!(r.get_by_name("age").unwrap().cast::<i32>().unwrap(), 31);

    Ok(())
}

/// Tests the ORM DELETE path: `delete()` removes the row and transitions to Detached.
///
/// Asserts the entity is Detached after deletion and the table is empty.
pub async fn test_orm_delete(ctx: &impl PrismTestContext) -> DbResult<()> {
    ctx.prepare_database().await?;
    let db = ctx.get_context();

    let mut entity = DbEntity::new(user(1, "Alice", "alice@example.com", 30));
    entity.save(db).await?;

    entity.delete(db).await?;
    assert!(entity.is_detached(), "entity should be Detached after delete");

    let mut cursor = db.find(FindQuery::new("test_users")).await?;
    assert!(cursor.next().await?.is_none(), "table should be empty after delete");

    Ok(())
}

/// Tests `DbCursorEntityExt::next_entity` for typed entity deserialization from a cursor.
///
/// Saves 3 entities then reads them back via the cursor extension trait.
pub async fn test_orm_cursor_entity(ctx: &impl PrismTestContext) -> DbResult<()> {
    ctx.prepare_database().await?;
    let db = ctx.get_context();

    for i in 1i64..=3 {
        let mut e = DbEntity::new(user(i, &format!("User {i}"), &format!("u{i}@test.com"), (20 + i) as i32));
        e.save(db).await?;
    }

    let mut cursor = db.find(FindQuery::new("test_users")).await?;
    let mut users: Vec<TestUser> = Vec::new();
    while let Some(u) = cursor.next_entity::<TestUser>().await? {
        users.push(u);
    }

    assert_eq!(users.len(), 3);
    assert!(users.iter().any(|u| u.name == "User 1"));
    assert!(users.iter().any(|u| u.name == "User 3"));

    Ok(())
}

/// Tests that calling `save()` on an unchanged tracked entity is a no-op.
///
/// A second save on an unmodified entity should produce no UPDATE and leave the row intact.
pub async fn test_orm_no_update_if_unchanged(ctx: &impl PrismTestContext) -> DbResult<()> {
    ctx.prepare_database().await?;
    let db = ctx.get_context();

    let mut entity = DbEntity::new(user(1, "Alice", "alice@example.com", 30));
    entity.save(db).await?;
    entity.save(db).await?; // second save: tracked, unchanged → no UPDATE

    let mut cursor = db.find(FindQuery::new("test_users")).await?;
    let r = cursor.next().await?.expect("row must still exist");
    assert_eq!(r.get_by_name("name").unwrap().cast::<String>().unwrap(), "Alice");
    assert!(cursor.next().await?.is_none());

    Ok(())
}

/// Tests a full entity lifecycle: insert → update → delete in sequence.
pub async fn test_orm_full_lifecycle(ctx: &impl PrismTestContext) -> DbResult<()> {
    ctx.prepare_database().await?;
    let db = ctx.get_context();

    let mut entity = DbEntity::new(user(1, "Bob", "bob@example.com", 25));
    assert!(entity.is_untracked());

    entity.save(db).await?;
    assert!(entity.is_tracked());

    entity.get_mut().email = "bob@updated.com".to_string();
    entity.save(db).await?;

    let mut cursor = db.find(
        FindQuery::new("test_users").filter(filter!(eq("id", 1i64)))
    ).await?;
    let r = cursor.next().await?.expect("entity must exist after update");
    assert_eq!(r.get_by_name("email").unwrap().cast::<String>().unwrap(), "bob@updated.com");

    entity.delete(db).await?;
    assert!(entity.is_detached());

    let mut cursor = db.find(FindQuery::new("test_users")).await?;
    assert!(cursor.next().await?.is_none(), "table must be empty after final delete");

    Ok(())
}
