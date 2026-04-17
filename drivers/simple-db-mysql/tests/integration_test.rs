use simple_db_core::query::Query;
use simple_db_core::types::DbValue;
use simple_db_core::driver::{DbDriver, DbExecutor};
use simple_db_mysql::MysqlDriver;
use sqlx::mysql::MySqlPoolOptions;
use std::sync::OnceLock;

static TEST_MUTEX: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();

async fn mysql_test<F, T>(future: F) -> T
where
    F: std::future::Future<Output = T>,
{
    let mutex = TEST_MUTEX.get_or_init(|| tokio::sync::Mutex::new(()));
    let _guard = mutex.lock().await;
    future.await
}

async fn create_test_pool() -> sqlx::MySqlPool {
    let database_url = "mysql://root:root@localhost:3306/simple-db-test";
    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await
        .expect("Failed to connect to MySQL test database");

    sqlx::query("DROP TABLE IF EXISTS users")
        .execute(&pool)
        .await
        .expect("Failed to drop users table");

    sqlx::query(
        r#"
        CREATE TABLE users (
            id INT AUTO_INCREMENT PRIMARY KEY,
            name VARCHAR(255) NOT NULL,
            email VARCHAR(255) NOT NULL,
            age INT,
            active BOOLEAN DEFAULT TRUE,
            balance DOUBLE
        )
        "#,
    )
    .execute(&pool)
    .await
    .expect("Failed to create users table");

    pool
}

#[tokio::test]
async fn test_mysql_driver_insert() {
    mysql_test(async {
        let pool = create_test_pool().await;
        let driver = MysqlDriver::new(pool);

    let insert_query = Query::insert("users")
        .insert(vec![
            ("name".to_string(), DbValue::from_string("Alice")),
            ("email".to_string(), DbValue::from_string("alice@example.com")),
            ("age".to_string(), DbValue::from_i32(30)),
            ("balance".to_string(), DbValue::from_f64(100.50)),
        ]);

    let rows_affected = driver.insert(insert_query).await.expect("Insert failed");
    assert_eq!(rows_affected, 1);

    let find_query = Query::find("users");
    let mut cursor = driver.find(find_query).await.expect("Find failed");
    let row = cursor.next().await.expect("Cursor next failed").expect("No row found");

    let email = row.get_by_name("email")
        .and_then(|v| v.as_string().map(|s| s.to_string()));
    assert_eq!(email, Some("alice@example.com".to_string()));
    assert_eq!(row.len(), 6);
    }).await;
}

#[tokio::test]
async fn test_mysql_driver_find_all() {
    mysql_test(async {
        let pool = create_test_pool().await;
        let driver = MysqlDriver::new(pool.clone());

    for i in 1..=3 {
        let insert_query = Query::insert("users")
            .insert(vec![
                ("name".to_string(), DbValue::from_string(format!("User {}", i))),
                ("email".to_string(), DbValue::from_string(format!("user{}@example.com", i))),
                ("age".to_string(), DbValue::from_i32(20 + i)),
            ]);
        driver.insert(insert_query).await.expect("Insert failed");
    }

    let find_query = Query::find("users");
    let mut cursor = driver.find(find_query).await.expect("Find failed");

    let mut count = 0;
    while let Ok(Some(row)) = cursor.next().await {
        count += 1;
        assert!(row.len() > 0);
    }

    assert_eq!(count, 3);
    }).await;
}

#[tokio::test]
async fn test_mysql_driver_filter() {
    mysql_test(async {
        let pool = create_test_pool().await;
        let driver = MysqlDriver::new(pool.clone());

    for i in 1..=3 {
        let insert_query = Query::insert("users")
            .insert(vec![
                ("name".to_string(), DbValue::from_string(format!("User {}", i))),
                ("email".to_string(), DbValue::from_string(format!("user{}@example.com", i))),
                ("age".to_string(), DbValue::from_i32(20 + i * 5)),
            ]);
        driver.insert(insert_query).await.expect("Insert failed");
    }

    let find_query = Query::find("users")
        .filter(|b| b.gt("age", DbValue::from_i32(25)));

    let mut cursor = driver.find(find_query).await.expect("Find failed");

    let mut count = 0;
    while let Ok(Some(_row)) = cursor.next().await {
        count += 1;
    }

    assert_eq!(count, 2);
    }).await;
}

#[tokio::test]
async fn test_mysql_driver_update() {
    mysql_test(async {
        let pool = create_test_pool().await;
        let driver = MysqlDriver::new(pool.clone());

    let insert_query = Query::insert("users")
        .insert(vec![
            ("name".to_string(), DbValue::from_string("Bob")),
            ("email".to_string(), DbValue::from_string("bob@example.com")),
            ("age".to_string(), DbValue::from_i32(25)),
        ]);
    driver.insert(insert_query).await.expect("Insert failed");

    let update_query = Query::update("users")
        .set("age", DbValue::from_i32(26))
        .filter(|b| b.eq("name", DbValue::from_string("Bob")));

    let rows_affected = driver.update(update_query).await.expect("Update failed");
    assert_eq!(rows_affected, 1);

    let find_query = Query::find("users")
        .filter(|b| b.eq("name", DbValue::from_string("Bob")));
    let mut cursor = driver.find(find_query).await.expect("Find failed");
    let row = cursor.next().await.expect("Cursor next failed").expect("No row found");

    let age = row.get_by_name("age").and_then(|v| v.as_i64());
    assert_eq!(age, Some(26));
    }).await;
}

#[tokio::test]
async fn test_mysql_driver_delete() {
    mysql_test(async {
        let pool = create_test_pool().await;
        let driver = MysqlDriver::new(pool.clone());

    for name in &["Alice", "Bob", "Charlie"] {
        let insert_query = Query::insert("users")
            .insert(vec![
                ("name".to_string(), DbValue::from_string(*name)),
                ("email".to_string(), DbValue::from_string(format!("{}@example.com", name.to_lowercase()))),
            ]);
        driver.insert(insert_query).await.expect("Insert failed");
    }

    let delete_query = Query::delete("users")
        .filter(|b| b.eq("name", DbValue::from_string("Bob")));

    let rows_affected = driver.delete(delete_query).await.expect("Delete failed");
    assert_eq!(rows_affected, 1);

    let find_query = Query::find("users");
    let mut cursor = driver.find(find_query).await.expect("Find failed");

    let mut count = 0;
    while let Ok(Some(_row)) = cursor.next().await {
        count += 1;
    }

    assert_eq!(count, 2);
    }).await;
}

#[tokio::test]
async fn test_mysql_driver_transaction_commit() {
    mysql_test(async {
        let pool = create_test_pool().await;
        let driver = MysqlDriver::new(pool.clone());

    let tx = driver.begin().await.expect("Failed to begin transaction");

    let insert_query = Query::insert("users")
        .insert(vec![
            ("name".to_string(), DbValue::from_string("TxUser")),
            ("email".to_string(), DbValue::from_string("txuser@example.com")),
        ]);

    tx.insert(insert_query).await.expect("Insert in transaction failed");
    tx.commit().await.expect("Commit failed");

    let find_query = Query::find("users");
    let mut cursor = driver.find(find_query).await.expect("Find failed");

    let mut found = false;
    while let Ok(Some(row)) = cursor.next().await {
        if let Some(name) = row.get_by_name("name").and_then(|v| v.as_string().map(|s| s.to_string())) {
            if name == "TxUser" {
                found = true;
            }
        }
    }

    assert!(found, "Transaction data should be persisted after commit");
    }).await;
}

#[tokio::test]
async fn test_mysql_driver_transaction_rollback() {
    mysql_test(async {
        let pool = create_test_pool().await;
        let driver = MysqlDriver::new(pool.clone());

    let insert_query = Query::insert("users")
        .insert(vec![
            ("name".to_string(), DbValue::from_string("Baseline")),
            ("email".to_string(), DbValue::from_string("baseline@example.com")),
        ]);
    driver.insert(insert_query).await.expect("Insert failed");

    let tx = driver.begin().await.expect("Failed to begin transaction");
    let insert_query = Query::insert("users")
        .insert(vec![
            ("name".to_string(), DbValue::from_string("RollbackUser")),
            ("email".to_string(), DbValue::from_string("rollback@example.com")),
        ]);

    tx.insert(insert_query).await.expect("Insert in transaction failed");
    tx.rollback().await.expect("Rollback failed");

    let find_query = Query::find("users");
    let mut cursor = driver.find(find_query).await.expect("Find failed");

    let mut count = 0;
    while let Ok(Some(_row)) = cursor.next().await {
        count += 1;
    }

    assert_eq!(count, 1, "Only baseline data should exist after rollback");
    }).await;
}

#[tokio::test]
async fn test_mysql_driver_ping() {
    mysql_test(async {
        let pool = create_test_pool().await;
        let driver = MysqlDriver::new(pool);

        assert!(driver.ping().await.is_ok(), "Ping should succeed");
    }).await;
}

#[tokio::test]
async fn test_mysql_driver_data_types() {
    mysql_test(async {
        let pool = create_test_pool().await;

        sqlx::query("DROP TABLE IF EXISTS test_types")
            .execute(&pool)
            .await
            .expect("Failed to drop test_types table");

        sqlx::query(
            r#"
            CREATE TABLE test_types (
                id INT AUTO_INCREMENT PRIMARY KEY,
                bool_val BOOLEAN,
                int_val INT,
                float_val DOUBLE,
                text_val TEXT
            )
            "#,
        )
        .execute(&pool)
        .await
        .expect("Failed to create test_types table");

    let driver = MysqlDriver::new(pool);

    let insert_query = Query::insert("test_types")
        .insert(vec![
            ("bool_val".to_string(), DbValue::from_bool(true)),
            ("int_val".to_string(), DbValue::from_i32(42)),
            ("float_val".to_string(), DbValue::from_f64(3.14159)),
            ("text_val".to_string(), DbValue::from_string("Hello, World!")),
        ]);

    driver.insert(insert_query).await.expect("Insert failed");

    let find_query = Query::find("test_types");
    let mut cursor = driver.find(find_query).await.expect("Find failed");
    let row = cursor.next().await.expect("Cursor next failed").expect("No row found");

    let bool_val = row.get_by_name("bool_val").and_then(|v| v.as_bool());
    assert_eq!(bool_val, Some(true));

    let int_val = row.get_by_name("int_val").and_then(|v| v.as_i64());
    assert_eq!(int_val, Some(42));

    let float_val = row.get_by_name("float_val").and_then(|v| v.as_f64());
    assert!(float_val.is_some() && (float_val.unwrap() - 3.14159).abs() < 0.00001);

    let text_val = row.get_by_name("text_val")
        .and_then(|v| v.as_string().map(|s| s.to_string()));
    assert_eq!(text_val, Some("Hello, World!".to_string()));
    }).await;
}
