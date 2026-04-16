use simple_db_core::query::Query;
use simple_db_core::types::DbValue;
use simple_db_core::driver::{DbDriver, DbExecutor};
use simple_db_sqlite::SqliteDriver;
use sqlx::sqlite::SqlitePoolOptions;

/// Create an in-memory SQLite pool for testing
async fn create_test_pool() -> sqlx::SqlitePool {
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect("sqlite://:memory:")
        .await
        .expect("Failed to create in-memory database");

    // Create a test table
    sqlx::query(
        r#"
        CREATE TABLE users (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            email TEXT NOT NULL,
            age INTEGER,
            active BOOLEAN DEFAULT 1,
            balance REAL
        )
        "#,
    )
    .execute(&pool)
    .await
    .expect("Failed to create users table");

    pool
}

#[tokio::test]
async fn test_sqlite_driver_insert() {
    let pool = create_test_pool().await;
    let driver = SqliteDriver::new(pool);

    // Insert a user
    let insert_query = Query::insert("users")
        .insert(vec![
            ("name".to_string(), DbValue::from_string("Alice")),
            ("email".to_string(), DbValue::from_string("alice@example.com")),
            ("age".to_string(), DbValue::from_i32(30)),
            ("balance".to_string(), DbValue::from_f64(100.50)),
        ]);

    let rows_affected = driver.insert(insert_query).await.expect("Insert failed");
    assert_eq!(rows_affected, 1);

    // Verify the insertion
    let find_query = Query::find("users");
    let mut cursor = driver.find(find_query).await.expect("Find failed");
    let row = cursor.next().await.expect("Cursor next failed").expect("No row found");

    let email = row.get_by_name("email")
        .and_then(|v| v.as_string().map(|s| s.to_string()));
    assert_eq!(email, Some("alice@example.com".to_string()));
    assert_eq!(row.len(), 6); // 6 columns
}

#[tokio::test]
async fn test_sqlite_driver_find_all() {
    let pool = create_test_pool().await;
    let driver = SqliteDriver::new(pool.clone());

    // Insert test data
    for i in 1..=3 {
        let insert_query = Query::insert("users")
            .insert(vec![
                ("name".to_string(), DbValue::from_string(format!("User {}", i))),
                ("email".to_string(), DbValue::from_string(format!("user{}@example.com", i))),
                ("age".to_string(), DbValue::from_i32(20 + i)),
            ]);
        driver.insert(insert_query).await.expect("Insert failed");
    }

    // Find all users
    let find_query = Query::find("users");
    let mut cursor = driver.find(find_query).await.expect("Find failed");

    let mut count = 0;
    while let Ok(Some(row)) = cursor.next().await {
        count += 1;
        assert!(row.len() > 0);
    }

    assert_eq!(count, 3);
}

#[tokio::test]
async fn test_sqlite_driver_filter() {
    let pool = create_test_pool().await;
    let driver = SqliteDriver::new(pool.clone());

    // Insert test data
    for i in 1..=3 {
        let insert_query = Query::insert("users")
            .insert(vec![
                ("name".to_string(), DbValue::from_string(format!("User {}", i))),
                ("email".to_string(), DbValue::from_string(format!("user{}@example.com", i))),
                ("age".to_string(), DbValue::from_i32(20 + i * 5)),
            ]);
        driver.insert(insert_query).await.expect("Insert failed");
    }

    // Find users with age > 25
    let find_query = Query::find("users")
        .filter(|b| b.gt("age", DbValue::from_i32(25)));

    let mut cursor = driver.find(find_query).await.expect("Find failed");

    let mut count = 0;
    while let Ok(Some(_row)) = cursor.next().await {
        count += 1;
    }

    assert_eq!(count, 2); // Users with age 25 and 30
}

#[tokio::test]
async fn test_sqlite_driver_update() {
    let pool = create_test_pool().await;
    let driver = SqliteDriver::new(pool.clone());

    // Insert a user
    let insert_query = Query::insert("users")
        .insert(vec![
            ("name".to_string(), DbValue::from_string("Bob")),
            ("email".to_string(), DbValue::from_string("bob@example.com")),
            ("age".to_string(), DbValue::from_i32(25)),
        ]);
    driver.insert(insert_query).await.expect("Insert failed");

    // Update the user's age
    let update_query = Query::update("users")
        .set("age", DbValue::from_i32(26))
        .filter(|b| b.eq("name", DbValue::from_string("Bob")));

    let rows_affected = driver.update(update_query).await.expect("Update failed");
    assert_eq!(rows_affected, 1);

    // Verify the update
    let find_query = Query::find("users")
        .filter(|b| b.eq("name", DbValue::from_string("Bob")));
    let mut cursor = driver.find(find_query).await.expect("Find failed");
    let row = cursor.next().await.expect("Cursor next failed").expect("No row found");

    let age = row.get_by_name("age").and_then(|v| v.as_i64());
    assert_eq!(age, Some(26));
}

#[tokio::test]
async fn test_sqlite_driver_delete() {
    let pool = create_test_pool().await;
    let driver = SqliteDriver::new(pool.clone());

    // Insert users
    for name in &["Alice", "Bob", "Charlie"] {
        let insert_query = Query::insert("users")
            .insert(vec![
                ("name".to_string(), DbValue::from_string(*name)),
                ("email".to_string(), DbValue::from_string(format!("{}@example.com", name.to_lowercase()))),
            ]);
        driver.insert(insert_query).await.expect("Insert failed");
    }

    // Delete one user
    let delete_query = Query::delete("users")
        .filter(|b| b.eq("name", DbValue::from_string("Bob")));

    let rows_affected = driver.delete(delete_query).await.expect("Delete failed");
    assert_eq!(rows_affected, 1);

    // Verify deletion
    let find_query = Query::find("users");
    let mut cursor = driver.find(find_query).await.expect("Find failed");

    let mut count = 0;
    while let Ok(Some(_row)) = cursor.next().await {
        count += 1;
    }

    assert_eq!(count, 2); // Only Alice and Charlie remain
}

#[tokio::test]
async fn test_sqlite_driver_transaction_commit() {
    let pool = create_test_pool().await;
    let driver = SqliteDriver::new(pool.clone());

    // Start a transaction
    let tx = driver.begin().await.expect("Failed to begin transaction");

    // Insert within transaction
    let insert_query = Query::insert("users")
        .insert(vec![
            ("name".to_string(), DbValue::from_string("TxUser")),
            ("email".to_string(), DbValue::from_string("txuser@example.com")),
        ]);

    tx.insert(insert_query).await.expect("Insert in transaction failed");

    // Commit the transaction
    tx.commit().await.expect("Commit failed");

    // Verify the data persisted
    let find_query = Query::find("users");
    let mut cursor = driver.find(find_query).await.expect("Find failed");

    let mut found = false;
    while let Ok(Some(row)) = cursor.next().await {
        if let Some(name) = row.get_by_name("name")
            .and_then(|v| v.as_string().map(|s| s.to_string())) {
            if name == "TxUser" {
                found = true;
            }
        }
    }

    assert!(found, "Transaction data should be persisted after commit");
}

#[tokio::test]
async fn test_sqlite_driver_transaction_rollback() {
    let pool = create_test_pool().await;
    let driver = SqliteDriver::new(pool.clone());

    // Insert baseline data
    let insert_query = Query::insert("users")
        .insert(vec![
            ("name".to_string(), DbValue::from_string("Baseline")),
            ("email".to_string(), DbValue::from_string("baseline@example.com")),
        ]);
    driver.insert(insert_query).await.expect("Insert failed");

    // Start a transaction
    let tx = driver.begin().await.expect("Failed to begin transaction");

    // Insert within transaction
    let insert_query = Query::insert("users")
        .insert(vec![
            ("name".to_string(), DbValue::from_string("RollbackUser")),
            ("email".to_string(), DbValue::from_string("rollback@example.com")),
        ]);

    tx.insert(insert_query).await.expect("Insert in transaction failed");

    // Rollback the transaction
    tx.rollback().await.expect("Rollback failed");

    // Verify the data was not persisted
    let find_query = Query::find("users");
    let mut cursor = driver.find(find_query).await.expect("Find failed");

    let mut count = 0;
    while let Ok(Some(_row)) = cursor.next().await {
        count += 1;
    }

    assert_eq!(count, 1, "Only baseline data should exist after rollback");
}

#[tokio::test]
async fn test_sqlite_driver_ping() {
    let pool = create_test_pool().await;
    let driver = SqliteDriver::new(pool);

    let ping_result = driver.ping().await;
    assert!(ping_result.is_ok(), "Ping should succeed");
}

#[tokio::test]
async fn test_sqlite_driver_data_types() {
    let pool = create_test_pool().await;

    // Create a table with various data types
    sqlx::query(
        r#"
        CREATE TABLE test_types (
            id INTEGER PRIMARY KEY,
            bool_val BOOLEAN,
            int_val INTEGER,
            float_val REAL,
            text_val TEXT
        )
        "#,
    )
    .execute(&pool)
    .await
    .expect("Failed to create test_types table");

    let driver = SqliteDriver::new(pool);

    // Insert with various types
    let insert_query = Query::insert("test_types")
        .insert(vec![
            ("bool_val".to_string(), DbValue::from_bool(true)),
            ("int_val".to_string(), DbValue::from_i32(42)),
            ("float_val".to_string(), DbValue::from_f64(3.14159)),
            ("text_val".to_string(), DbValue::from_string("Hello, World!")),
        ]);

    driver.insert(insert_query).await.expect("Insert failed");

    // Find and verify types
    let find_query = Query::find("test_types");
    let mut cursor = driver.find(find_query).await.expect("Find failed");

    let row = cursor.next().await.expect("Cursor next failed").expect("No row found");

    // Note: SQLite stores booleans as 0/1 integers
    let bool_val = row.get_by_name("bool_val").and_then(|v| v.as_i64());
    assert_eq!(bool_val, Some(1));

    let int_val = row.get_by_name("int_val").and_then(|v| v.as_i64());
    assert_eq!(int_val, Some(42));

    let float_val = row.get_by_name("float_val").and_then(|v| v.as_f64());
    assert!(float_val.is_some() && (float_val.unwrap() - 3.14159).abs() < 0.00001);

    let text_val = row.get_by_name("text_val")
        .and_then(|v| v.as_string().map(|s| s.to_string()));
    assert_eq!(text_val, Some("Hello, World!".to_string()));
}
