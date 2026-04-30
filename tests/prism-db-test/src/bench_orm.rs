use std::time::Instant;

use prism_db::{DbEntity, DbCursorEntityExt, driver::executor::DbExecutor, query::{FindQuery, InsertQuery}, types::{DbResult, DbValue}};

use crate::{context::PrismTestContext, entity::TestUser};

const N: i64 = 10_000;
// SQLite limits bind parameters per statement; chunk bulk inserts to stay under.
const BATCH: usize = 200;

fn test_user(i: i64) -> TestUser {
    TestUser {
        id: i,
        name: format!("User {i}"),
        email: format!("user{i}@example.com"),
        age: (20 + i % 60) as i32,
    }
}

async fn populate(ctx: &impl PrismTestContext) -> DbResult<()> {
    let db = ctx.get_context();
    let rows: Vec<Vec<(&str, DbValue)>> = (1..=N)
        .map(|i| vec![
            ("id",    DbValue::from(i)),
            ("name",  DbValue::from(format!("User {i}"))),
            ("email", DbValue::from(format!("user{i}@example.com"))),
            ("age",   DbValue::from((20 + i % 60) as i32)),
        ])
        .collect();
    for chunk in rows.chunks(BATCH) {
        db.insert(InsertQuery::new("test_users").bulk_insert(chunk.to_vec())).await?;
    }
    Ok(())
}

/// Benchmarks ORM INSERT performance via change tracking.
///
/// Creates `N` (10 000) `DbEntity` instances and calls `save()` on each
/// individually, exercising the full Untracked → Tracked state transition path.
/// Prints elapsed time and ops/s to stdout.
pub async fn bench_orm_insert(ctx: &impl PrismTestContext) -> DbResult<()> {
    ctx.prepare_database().await?;
    let db = ctx.get_context();

    let start = Instant::now();
    for i in 1..=N {
        let mut entity = DbEntity::new(test_user(i));
        entity.save(db).await?;
    }
    let elapsed = start.elapsed();

    println!(
        "[bench_orm_insert] {N} entities saved in {elapsed:?}  ({:.0} ops/s)",
        N as f64 / elapsed.as_secs_f64()
    );
    Ok(())
}

/// Benchmarks ORM cursor deserialization performance.
///
/// Pre-populates `N` (10 000) rows, then reads all of them as typed `TestUser`
/// entities using `DbCursorEntityExt::next_entity` — `N` row deserializations.
/// Prints elapsed time and rows/s to stdout.
pub async fn bench_orm_find(ctx: &impl PrismTestContext) -> DbResult<()> {
    ctx.prepare_database().await?;
    populate(ctx).await?;
    let db = ctx.get_context();

    let start = Instant::now();
    let mut cursor = db.find(FindQuery::new("test_users")).await?;
    let mut count = 0i64;
    while cursor.next_entity::<TestUser>().await?.is_some() {
        count += 1;
    }
    let elapsed = start.elapsed();

    assert_eq!(count, N, "cursor should yield exactly N entities");
    println!(
        "[bench_orm_find] {N} entities deserialized in {elapsed:?}  ({:.0} rows/s)",
        N as f64 / elapsed.as_secs_f64()
    );
    Ok(())
}

/// Benchmarks ORM UPDATE performance via change tracking.
///
/// Pre-populates `N` (10 000) rows, loads them all as tracked `DbEntity`
/// instances, modifies the `name` field of each, then calls `save()` on each —
/// `N` UPDATE statements driven by change detection.
/// Prints elapsed time and ops/s to stdout.
pub async fn bench_orm_update(ctx: &impl PrismTestContext) -> DbResult<()> {
    ctx.prepare_database().await?;
    populate(ctx).await?;
    let db = ctx.get_context();

    // Load all N entities as tracked instances.
    let mut entities: Vec<DbEntity<TestUser>> = Vec::with_capacity(N as usize);
    let mut cursor = db.find(FindQuery::new("test_users")).await?;
    while let Some(row) = cursor.next().await? {
        entities.push(DbEntity::from_db(row.as_ref()));
    }
    assert_eq!(entities.len(), N as usize);

    let start = Instant::now();
    for entity in &mut entities {
        let id = entity.get().id;
        entity.get_mut().name = format!("Updated {id}");
        entity.save(db).await?;
    }
    let elapsed = start.elapsed();

    println!(
        "[bench_orm_update] {N} entities updated in {elapsed:?}  ({:.0} ops/s)",
        N as f64 / elapsed.as_secs_f64()
    );
    Ok(())
}

/// Benchmarks ORM DELETE performance via change tracking.
///
/// Pre-populates `N` (10 000) rows, loads them all as tracked `DbEntity`
/// instances, then calls `delete()` on each — `N` DELETE statements in total.
/// Prints elapsed time and ops/s to stdout.
pub async fn bench_orm_delete(ctx: &impl PrismTestContext) -> DbResult<()> {
    ctx.prepare_database().await?;
    populate(ctx).await?;
    let db = ctx.get_context();

    // Load all N entities as tracked instances.
    let mut entities: Vec<DbEntity<TestUser>> = Vec::with_capacity(N as usize);
    let mut cursor = db.find(FindQuery::new("test_users")).await?;
    while let Some(row) = cursor.next().await? {
        entities.push(DbEntity::from_db(row.as_ref()));
    }
    assert_eq!(entities.len(), N as usize);

    let start = Instant::now();
    for entity in &mut entities {
        entity.delete(db).await?;
    }
    let elapsed = start.elapsed();

    println!(
        "[bench_orm_delete] {N} entities deleted in {elapsed:?}  ({:.0} ops/s)",
        N as f64 / elapsed.as_secs_f64()
    );
    Ok(())
}
