use prism_db::{filter, group, project, query::{DeleteQuery, FindQuery, InsertQuery, UpdateQuery}, sort, types::{DbResult, DbValue}, driver::executor::DbExecutor};

use crate::context::PrismTestContext;

fn row(id: i64, name: &str, email: &str, age: i32) -> Vec<(&'static str, DbValue)> {
    vec![
        ("id",    DbValue::from(id)),
        ("name",  DbValue::from(name.to_string())),
        ("email", DbValue::from(email.to_string())),
        ("age",   DbValue::from(age)),
    ]
}

/// Tests inserting a single row and verifying it was stored correctly.
///
/// Asserts: 1 row affected, row is readable and has the expected field values.
pub async fn test_crud_insert(ctx: &impl PrismTestContext) -> DbResult<()> {
    ctx.prepare_database().await?;
    let db = ctx.get_context();

    let affected = db.insert(
        InsertQuery::new("test_users").insert(row(1, "Alice", "alice@example.com", 30))
    ).await?;
    assert_eq!(affected, 1, "insert should report 1 affected row");

    let mut cursor = db.find(FindQuery::new("test_users")).await?;
    let r = cursor.next().await?.expect("expected one row after insert");
    assert_eq!(r.get_by_name("name").unwrap().cast::<String>().unwrap(), "Alice");
    assert_eq!(r.get_by_name("age").unwrap().cast::<i32>().unwrap(), 30);
    assert!(cursor.next().await?.is_none(), "expected exactly one row");

    Ok(())
}

/// Tests finding a specific row using an equality filter on the primary key.
///
/// Inserts 5 rows, queries for id=3, asserts only that row is returned.
pub async fn test_crud_find(ctx: &impl PrismTestContext) -> DbResult<()> {
    ctx.prepare_database().await?;
    let db = ctx.get_context();

    for i in 1i64..=5 {
        db.insert(InsertQuery::new("test_users").insert(row(
            i,
            &format!("User {i}"),
            &format!("user{i}@example.com"),
            (20 + i) as i32,
        ))).await?;
    }

    let mut cursor = db.find(
        FindQuery::new("test_users").filter(filter!(eq("id", 3i64)))
    ).await?;

    let r = cursor.next().await?.expect("expected row with id=3");
    assert_eq!(r.get_by_name("id").unwrap().cast::<i64>().unwrap(), 3);
    assert_eq!(r.get_by_name("name").unwrap().cast::<String>().unwrap(), "User 3");
    assert!(cursor.next().await?.is_none(), "filter should match exactly one row");

    Ok(())
}

/// Tests updating a row and verifying the changed values persisted.
///
/// Inserts a row, updates name and age, re-fetches and asserts new values.
pub async fn test_crud_update(ctx: &impl PrismTestContext) -> DbResult<()> {
    ctx.prepare_database().await?;
    let db = ctx.get_context();

    db.insert(InsertQuery::new("test_users").insert(row(1, "Alice", "alice@example.com", 30))).await?;

    let affected = db.update(
        UpdateQuery::new("test_users")
            .set("name", "Alicia".to_string())
            .set("age", DbValue::from(31i32))
            .filter(filter!(eq("id", 1i64)))
    ).await?;
    assert_eq!(affected, 1, "update should report 1 affected row");

    let mut cursor = db.find(
        FindQuery::new("test_users").filter(filter!(eq("id", 1i64)))
    ).await?;
    let r = cursor.next().await?.expect("expected updated row");
    assert_eq!(r.get_by_name("name").unwrap().cast::<String>().unwrap(), "Alicia");
    assert_eq!(r.get_by_name("age").unwrap().cast::<i32>().unwrap(), 31);

    Ok(())
}

/// Tests deleting a row and verifying it no longer exists.
///
/// Inserts a row, deletes it, then asserts the table is empty.
pub async fn test_crud_delete(ctx: &impl PrismTestContext) -> DbResult<()> {
    ctx.prepare_database().await?;
    let db = ctx.get_context();

    db.insert(InsertQuery::new("test_users").insert(row(1, "Alice", "alice@example.com", 30))).await?;

    let affected = db.delete(
        DeleteQuery::new("test_users").filter(filter!(eq("id", 1i64)))
    ).await?;
    assert_eq!(affected, 1, "delete should report 1 affected row");

    let mut cursor = db.find(FindQuery::new("test_users")).await?;
    assert!(cursor.next().await?.is_none(), "table should be empty after delete");

    Ok(())
}

/// Tests bulk insertion and range-filtered queries.
///
/// Inserts 10 rows via `bulk_insert`, then queries with a `gte` filter,
/// asserting the correct subset is returned.
pub async fn test_crud_bulk(ctx: &impl PrismTestContext) -> DbResult<()> {
    ctx.prepare_database().await?;
    let db = ctx.get_context();

    // ages: 21 (i=1) … 30 (i=10)
    let rows: Vec<Vec<(&str, DbValue)>> = (1i64..=10)
        .map(|i| row(i, &format!("User {i}"), &format!("u{i}@test.com"), (20 + i) as i32))
        .collect();
    let affected = db.insert(InsertQuery::new("test_users").bulk_insert(rows)).await?;
    assert_eq!(affected, 10);

    // age >= 26 → ids 6..=10 → 5 rows
    let mut cursor = db.find(
        FindQuery::new("test_users").filter(filter!(gte("age", 26i32)))
    ).await?;
    let mut count = 0usize;
    while cursor.next().await?.is_some() {
        count += 1;
    }
    assert_eq!(count, 5, "gte filter should return 5 rows");

    Ok(())
}

/// Tests column projection: only the specified fields are included in each result row.
///
/// Projects `id` and `name` only; asserts `email` is absent from the result.
pub async fn test_crud_projection(ctx: &impl PrismTestContext) -> DbResult<()> {
    ctx.prepare_database().await?;
    let db = ctx.get_context();

    db.insert(InsertQuery::new("test_users").insert(row(1, "Alice", "alice@example.com", 30))).await?;

    let mut cursor = db.find(
        FindQuery::new("test_users").project(project!(field("id"), field("name")))
    ).await?;

    let r = cursor.next().await?.expect("expected one projected row");
    assert_eq!(r.get_by_name("id").unwrap().cast::<i64>().unwrap(), 1);
    assert_eq!(r.get_by_name("name").unwrap().cast::<String>().unwrap(), "Alice");
    assert!(r.get_by_name("email").is_none(), "email should not be in projected result");
    assert!(cursor.next().await?.is_none());

    Ok(())
}

/// Tests ORDER BY: rows are returned in the sort order specified by `sort!`.
///
/// Inserts rows with ages [30, 25, 35] and queries with `sort!(desc("age"))`,
/// asserting the result sequence is [35, 30, 25].
pub async fn test_crud_sort(ctx: &impl PrismTestContext) -> DbResult<()> {
    ctx.prepare_database().await?;
    let db = ctx.get_context();

    for (id, age) in [(1i64, 30i32), (2, 25), (3, 35)] {
        db.insert(InsertQuery::new("test_users").insert(
            row(id, &format!("User {id}"), &format!("u{id}@test.com"), age)
        )).await?;
    }

    let mut cursor = db.find(
        FindQuery::new("test_users").order_by(sort!(desc("age")))
    ).await?;

    let mut ages: Vec<i32> = Vec::new();
    while let Some(r) = cursor.next().await? {
        ages.push(r.get_by_name("age").unwrap().cast::<i32>().unwrap());
    }
    assert_eq!(ages, [35, 30, 25], "rows should be in descending age order");

    Ok(())
}

/// Tests GROUP BY: collapses duplicate group values into one row per distinct value.
///
/// Inserts 4 rows across 3 distinct ages and asserts the grouped query returns 3 rows.
pub async fn test_crud_group(ctx: &impl PrismTestContext) -> DbResult<()> {
    ctx.prepare_database().await?;
    let db = ctx.get_context();

    // ages: 30 (×2), 25 (×1), 35 (×1) → 3 distinct groups
    for (id, age) in [(1i64, 30i32), (2, 30), (3, 25), (4, 35)] {
        db.insert(InsertQuery::new("test_users").insert(
            row(id, &format!("User {id}"), &format!("u{id}@test.com"), age)
        )).await?;
    }

    let mut cursor = db.find(
        FindQuery::new("test_users")
            .project(project!(field("age")))
            .group_by(group!("age"))
            .order_by(sort!(asc("age")))
    ).await?;

    let mut groups: Vec<i32> = Vec::new();
    while let Some(r) = cursor.next().await? {
        groups.push(r.get_by_name("age").unwrap().cast::<i32>().unwrap());
    }
    assert_eq!(groups, [25, 30, 35], "GROUP BY should yield one row per distinct age");

    Ok(())
}

/// Tests LIMIT and OFFSET for paginated queries.
///
/// Inserts 5 rows ordered by id, then queries with `LIMIT 2 OFFSET 1` using
/// `sort!(asc("id"))`, asserting the second page rows (id=2, id=3) are returned.
pub async fn test_crud_limit_offset(ctx: &impl PrismTestContext) -> DbResult<()> {
    ctx.prepare_database().await?;
    let db = ctx.get_context();

    for i in 1i64..=5 {
        db.insert(InsertQuery::new("test_users").insert(
            row(i, &format!("User {i}"), &format!("u{i}@test.com"), (20 + i) as i32)
        )).await?;
    }

    let mut cursor = db.find(
        FindQuery::new("test_users")
            .order_by(sort!(asc("id")))
            .limit(2)
            .offset(1)
    ).await?;

    let first = cursor.next().await?.expect("expected first page row");
    assert_eq!(first.get_by_name("id").unwrap().cast::<i64>().unwrap(), 2);
    let second = cursor.next().await?.expect("expected second page row");
    assert_eq!(second.get_by_name("id").unwrap().cast::<i64>().unwrap(), 3);
    assert!(cursor.next().await?.is_none(), "LIMIT 2 should return exactly 2 rows");

    Ok(())
}
