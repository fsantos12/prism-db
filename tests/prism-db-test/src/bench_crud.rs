use std::time::Instant;

use prism_db::{driver::executor::DbExecutor, filter, query::{DeleteQuery, FindQuery, InsertQuery, UpdateQuery}, types::{DbResult, DbValue}};

use crate::context::PrismTestContext;

const N: i64 = 10_000;
// SQLite limits bind parameters per statement; chunk bulk inserts to stay under.
const BATCH: usize = 200;

fn user_row(i: i64) -> Vec<(&'static str, DbValue)> {
    vec![
        ("id",    DbValue::from(i)),
        ("name",  DbValue::from(format!("User {i}"))),
        ("email", DbValue::from(format!("user{i}@example.com"))),
        ("age",   DbValue::from((20 + i % 60) as i32)),
    ]
}

async fn populate(ctx: &impl PrismTestContext) -> DbResult<()> {
    let db = ctx.get_context();
    let rows: Vec<Vec<(&str, DbValue)>> = (1..=N).map(user_row).collect();
    for chunk in rows.chunks(BATCH) {
        db.insert(InsertQuery::new("test_users").bulk_insert(chunk.to_vec())).await?;
    }
    Ok(())
}

/// Benchmarks individual INSERT performance.
///
/// Performs `N` (10 000) separate INSERT statements, one row at a time.
/// Prints elapsed time and ops/s to stdout.
pub async fn bench_crud_insert(ctx: &impl PrismTestContext) -> DbResult<()> {
    ctx.prepare_database().await?;
    let db = ctx.get_context();

    let start = Instant::now();
    for i in 1..=N {
        db.insert(InsertQuery::new("test_users").insert(user_row(i))).await?;
    }
    let elapsed = start.elapsed();

    println!(
        "[bench_crud_insert] {N} rows inserted in {elapsed:?}  ({:.0} ops/s)",
        N as f64 / elapsed.as_secs_f64()
    );
    Ok(())
}

/// Benchmarks full-table SELECT + cursor iteration.
///
/// Pre-populates `N` (10 000) rows via bulk insert, then scans all of them
/// through a single query cursor — `N` row reads in total.
/// Prints elapsed time and rows/s to stdout.
pub async fn bench_crud_find(ctx: &impl PrismTestContext) -> DbResult<()> {
    ctx.prepare_database().await?;
    populate(ctx).await?;
    let db = ctx.get_context();

    let start = Instant::now();
    let mut cursor = db.find(FindQuery::new("test_users")).await?;
    let mut count = 0i64;
    while cursor.next().await?.is_some() {
        count += 1;
    }
    let elapsed = start.elapsed();

    assert_eq!(count, N, "cursor should yield exactly N rows");
    println!(
        "[bench_crud_find] {N} rows read in {elapsed:?}  ({:.0} rows/s)",
        N as f64 / elapsed.as_secs_f64()
    );
    Ok(())
}

/// Benchmarks individual UPDATE performance.
///
/// Pre-populates `N` (10 000) rows, then updates the `name` column of each
/// row individually — `N` UPDATE statements in total.
/// Prints elapsed time and ops/s to stdout.
pub async fn bench_crud_update(ctx: &impl PrismTestContext) -> DbResult<()> {
    ctx.prepare_database().await?;
    populate(ctx).await?;
    let db = ctx.get_context();

    let start = Instant::now();
    for i in 1..=N {
        db.update(
            UpdateQuery::new("test_users")
                .set("name", format!("Updated {i}"))
                .filter(filter!(eq("id", i)))
        ).await?;
    }
    let elapsed = start.elapsed();

    println!(
        "[bench_crud_update] {N} rows updated in {elapsed:?}  ({:.0} ops/s)",
        N as f64 / elapsed.as_secs_f64()
    );
    Ok(())
}

/// Benchmarks individual DELETE performance.
///
/// Pre-populates `N` (10 000) rows, then deletes each row individually —
/// `N` DELETE statements in total.
/// Prints elapsed time and ops/s to stdout.
pub async fn bench_crud_delete(ctx: &impl PrismTestContext) -> DbResult<()> {
    ctx.prepare_database().await?;
    populate(ctx).await?;
    let db = ctx.get_context();

    let start = Instant::now();
    for i in 1..=N {
        db.delete(
            DeleteQuery::new("test_users").filter(filter!(eq("id", i)))
        ).await?;
    }
    let elapsed = start.elapsed();

    println!(
        "[bench_crud_delete] {N} rows deleted in {elapsed:?}  ({:.0} ops/s)",
        N as f64 / elapsed.as_secs_f64()
    );
    Ok(())
}
