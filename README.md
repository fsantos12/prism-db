# Prism-DB

**A modular, extensible database abstraction framework for Rust supporting SQL, NoSQL, and beyond.**

Prism-DB is a **backend-agnostic database layer** designed to unify access across heterogeneous data storesâ€”SQL databases (PostgreSQL, MySQL, SQLite), NoSQL systems (MongoDB, DynamoDB), in-memory caches (Redis, Memcached), and custom backends. It provides a pluggable driver architecture, type-safe abstractions, and common patterns (change tracking, transactions, entity mapping) that work identically regardless of the underlying store. Write once, run anywhere.

---

## Key Features

- **Backend-Agnostic API**: Write database code once; run against SQL, NoSQL, in-memory, or custom stores
- **Pluggable Drivers**: Drop-in driver system for PostgreSQL, MySQL, SQLite, MongoDB, Redis, DynamoDB, and beyond
- **Unified Query Interface**: `FindQuery`, `InsertQuery`, `UpdateQuery`, `DeleteQuery` work across backends (with backend-specific optimizations)
- **Change Tracking**: Automatic INSERT/UPDATE/DELETE detection via `DbEntity<T>` state machinesâ€”works for any datastore
- **Async/Await**: Full async support built on `tokio`â€”ideal for high-concurrency workloads
- **Transaction Support**: ACID-like semantics where supported, graceful degradation for eventual-consistency stores
- **Type-Safe Values**: `DbValue` tagged-pointer system ensures type safety at zero runtime cost
- **Connection Pooling**: Integrated pooling and resource management for pooled backends
- **Modular Architecture**: Clean separationâ€”swap drivers without touching application code

---

## Tech Stack

| Layer | Technology |
|-------|-----------|
| **Language** | Rust (2024 edition) |
| **Async Runtime** | Tokio |
| **Core Abstractions** | Async Traits (async-trait) |
| **Type System** | Serde, serde_json |
| **Date/Time** | Chrono |
| **Numeric** | Rust Decimal |
| **UUID** | UUID crate |
| **Error Handling** | thiserror |
| **Macros** | Procedural macros (syn, quote) |
| **SQL Drivers** | SQLx (Postgres, MySQL, SQLite) |
| **NoSQL Drivers** | MongoDB driver, Redis client, DynamoDB SDK (via botocore) |
| **Custom Drivers** | Implement `DbDriver` trait to add support for any backend |

---

## Architecture Overview

Prism-DB follows a **layered, plugin-based architecture** designed to support any backend data store:

```
â”Œâ”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”
â”‚           Application Layer                  â”‚
â”‚      (Your Entity Types & Queries)           â”‚
â”‚       (Database-agnostic code)               â”‚
â””â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”¬â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”˜
                 â”‚
â”Œâ”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â–¼â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”
â”‚        Prism-DB (Public Facade)             â”‚
â”‚      Re-exports core, orm, and macros        â”‚
â””â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”¬â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”˜
                 â”‚
    â”Œâ”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”¼â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”
    â”‚            â”‚            â”‚
â”Œâ”€â”€â”€â–¼â”€â”€â”    â”Œâ”€â”€â”€â”€â–¼â”€â”€â”€â”   â”Œâ”€â”€â”€â–¼â”€â”€â”€â”€â”
â”‚Core  â”‚    â”‚  ORM   â”‚   â”‚ Macros â”‚
â”‚      â”‚    â”‚        â”‚   â”‚        â”‚
â”‚- Traits   â”‚- Entity â”‚   â”‚- Deriveâ”‚
â”‚- Builders â”‚- Cursor â”‚   â”‚- Attrs â”‚
â”‚- Types    â”‚- Change â”‚   â”‚        â”‚
â””â”€â”€â”€â”¬â”€â”€â”˜    â”‚ Trackingâ”‚   â””â”€â”€â”€â”€â”€â”€â”€â”€â”˜
    â”‚       â””â”€â”€â”€â”€â”¬â”€â”€â”€â”˜
    â””â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”¬â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”
                â”‚                   â”‚
        â”Œâ”€â”€â”€â”€â”€â”€â”€â–¼â”€â”€â”€â”€â”€â”€â”€â”€â”  â”Œâ”€â”€â”€â”€â”€â”€â–¼â”€â”€â”€â”€â”€â”€â”
        â”‚  Driver Traits â”‚  â”‚  Transactionâ”‚
        â”‚                â”‚  â”‚   Helpers   â”‚
        â”‚- DbDriver      â”‚  â”‚             â”‚
        â”‚- DbExecutor    â”‚  â”‚- begin()    â”‚
        â”‚- DbTransaction â”‚  â”‚- commit()   â”‚
        â””â”€â”€â”€â”€â”€â”€â”€â”€â”¬â”€â”€â”€â”€â”€â”€â”€â”˜  â”‚- rollback() â”‚
                 â”‚          â””â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”˜
    â”Œâ”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”¼â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”
    â”‚            â”‚                    â”‚
    â–¼            â–¼                    â–¼
 â”Œâ”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â” â”Œâ”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â” â”Œâ”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”
 â”‚ SQL      â”‚ â”‚ NoSQL       â”‚ â”‚ Cache &    â”‚
 â”‚ Drivers  â”‚ â”‚ Drivers     â”‚ â”‚ Message Q  â”‚
 â”‚          â”‚ â”‚             â”‚ â”‚            â”‚
 â”‚- Postgresâ”‚ â”‚- MongoDB    â”‚ â”‚- Redis     â”‚
 â”‚- MySQL   â”‚ â”‚- DynamoDB   â”‚ â”‚- Memcached â”‚
 â”‚- SQLite  â”‚ â”‚- Cassandra  â”‚ â”‚- RabbitMQ  â”‚
 â””â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”˜ â””â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”˜ â””â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”˜
    â”‚            â”‚                    â”‚
    â””â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”¼â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”˜
                 â”‚
    â”Œâ”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â–¼â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”
    â”‚   Heterogeneous Backends      â”‚
    â”‚ (Any database, cache, or      â”‚
    â”‚  messaging system)            â”‚
    â””â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”˜
```

**Design Principles:**

- **Backend Agnostic**: Query builders and types don't depend on specific database implementations.
- **Trait-Based Driver Model**: `DbDriver`, `DbExecutor`, and `DbTransaction` define a pluggable interface that any backend can implement.
- **Unified API**: Identical usage patterns across SQL, NoSQL, in-memory, and custom stores.
- **Modular Crates**: Each crate has a single responsibilityâ€”easy to understand, test, and extend.
- **Zero-Cost Abstractions**: Leverages Rust's type system to eliminate runtime overhead.

**Supported Backend Categories:**

| Category | Examples | Query Support |
|----------|----------|---------------|
| **SQL** | PostgreSQL, MySQL, SQLite | `FindQuery`, `InsertQuery`, `UpdateQuery`, `DeleteQuery` (full SQL) |
| **NoSQL** | MongoDB, DynamoDB, Cassandra | `FindQuery` (filter/project), `InsertQuery`, `UpdateQuery` (document-oriented) |
| **Cache/KV** | Redis, Memcached | `FindQuery` (key-based), `InsertQuery` (set), `DeleteQuery` (delete key) |
| **Custom** | Your backend here | Implement `DbDriver` trait; adapt queries as needed |

---

## Quick Start

### 1. Install Rust

Ensure you have Rust 1.70+ installed. If not, visit [rustup.rs](https://rustup.rs).

### 2. Clone the Repository

```bash
git clone https://github.com/your-org/Prism-DB.git
cd Prism-DB
```

### 3. Set Up a Test Database (Optional)

For local SQL testing, spin up a PostgreSQL database:

```bash
# Using Docker
docker run --name postgres-test -e POSTGRES_PASSWORD=test -p 5432:5432 -d postgres:15
```

For Redis cache testing:

```bash
docker run --name redis-test -p 6379:6379 -d redis:7
```

### 4. Run Tests

```bash
# Unit tests (no database required)
cargo test --lib

# Integration tests (requires TEST_POSTGRES_URL, TEST_MYSQL_URL, TEST_SQLITE_URL, etc.)
export TEST_SQLITE_URL=sqlite://:memory:
cargo test --test sqlite_integration -- --nocapture
```

### 5. Add to Your Project

```toml
[dependencies]
Prism-DB = "0.1"

# Choose your driver(s)
Prism-DB-postgres = "0.1"   # for SQL: PostgreSQL
Prism-DB-mysql = "0.1"      # for SQL: MySQL
Prism-DB-sqlite = "0.1"     # for SQL: SQLite
# Prism-DB-redis = "0.1"    # (coming soon) for cache
# Prism-DB-mongodb = "0.1"  # (coming soon) for NoSQL

tokio = { version = "1.0", features = ["full"] }
```

---

## Configuration

### Environment Variables (for Integration Tests)

Set these to enable integration tests for each backend:

```bash
# SQL Backends
export TEST_POSTGRES_URL=postgres://user:password@localhost:5432/testdb
export TEST_MYSQL_URL=mysql://user:password@localhost:3306/testdb
export TEST_SQLITE_URL=sqlite:///path/to/test.db
# or for in-memory: sqlite://:memory:

# NoSQL / Cache Backends (future drivers)
export TEST_REDIS_URL=redis://localhost:6379/0
export TEST_MONGODB_URL=mongodb://localhost:27017/testdb
```

If not set, integration tests are skipped with a message.

### Backend-Specific Configuration

Each driver handles configuration via connection strings or builder APIs. Examples:

**PostgreSQL (via `sqlx` URI):**
```rust
let driver = PostgresDriver::connect("postgres://user:pass@localhost/db").await?;
```

**MySQL (via `sqlx` URI):**
```rust
let driver = MySqlDriver::connect("mysql://user:pass@localhost/db").await?;
```

**SQLite (local file or in-memory):**
```rust
let driver = SqliteDriver::connect("sqlite:///path/to/db.sqlite").await?;
let driver = SqliteDriver::connect("sqlite://:memory:").await?;  // in-memory
```

### Connection Pooling

SQL drivers (Postgres, MySQL, SQLite) use `sqlx` pooling with sensible defaults:

- **Max connections**: 5 per pool
- **Connection timeout**: sqlx default (30 seconds)
- **Idle timeout**: sqlx default

Override via backend-specific `PoolOptions`:

```rust
use sqlx::postgres::PgPoolOptions;
use std::time::Duration;

let pool = PgPoolOptions::new()
    .max_connections(10)
    .acquire_timeout(Duration::from_secs(10))
    .connect("postgres://...").await?;
```

For NoSQL and cache backends, pooling and connection management are driver-specific.

---

## Usage Examples

All examples below show SQL patterns, but the same API works for NoSQL, Redis, and custom backends.

### Basic Query: Find All Users

```rust
use prism_db_postgres::PostgresDriver;
use prism_db_core::query::FindQuery;
use prism_db_core::types::DbResult;

#[tokio::main]
async fn main() -> DbResult<()> {
    let driver = PostgresDriver::connect("postgres://localhost/mydb").await?;
    
    let query = FindQuery::new("users")
        .limit(10);
    
    let cursor = driver.prepare_find(query)?
        .execute()
        .await?;
    
    while let Some(row) = cursor.next().await? {
        println!("ID: {}, Name: {}", 
            row.get::<i64>("id")?, 
            row.get::<String>("name")?);
    }
    
    Ok(())
}
```

To use **MongoDB** instead, simply swap the driver:

```rust
// use prism_db_mongodb::MongoDbDriver;  // (when available)
// let driver = MongoDbDriver::connect("mongodb://localhost/mydb").await?;
```

The query and result-handling code remains **identical**.

### Insert Rows

```rust
use prism_db_core::query::InsertQuery;
use prism_db_core::types::DbValue;

let query = InsertQuery::new("users")
    .insert("id", DbValue::from(1i64))
    .insert("name", DbValue::from("Alice"))
    .insert("email", DbValue::from("alice@example.com"));

driver.prepare_insert(query)?
    .execute()
    .await?;
```

### Filtered Find (With Filters)

```rust
use prism_db_core::query::FindQuery;
use prism_db_core::filter;

let query = FindQuery::new("users")
    .filter(filter!()
        .eq("status", "active")
        .gt("age", 18)
    )
    .order_by("created_at DESC")
    .limit(100);

let cursor = driver.prepare_find(query)?
    .execute()
    .await?;
```

### Transaction Example

```rust
use prism_db_core::driver::transaction::DbTransactionExt;
use std::sync::Arc;

let result = driver.transaction(|tx| async move {
    // All queries within this block are in a single transaction
    driver.prepare_insert(insert_query_1)?.execute().await?;
    driver.prepare_insert(insert_query_2)?.execute().await?;
    
    Ok::<_, DbError>(42) // Automatically commits on Ok
}).await?; // Automatically rolls back on Err

assert_eq!(result, 42);
```

### Using Entities with Change Tracking

```rust
use prism_db_orm::DbEntity;

// Create a new, untracked entity
let mut user = DbEntity::new(User { id: 1, name: "Bob".to_string() });

// Modify it
user.data_mut().name = "Robert".to_string();

// Track and persist changes (INSERT or UPDATE based on state)
driver.persist(&user).await?;
```

### Implementing a Custom Driver

To add support for a new backend (e.g., DuckDB, Firestore, DynamoDB), implement the `DbDriver` trait:

```rust
use prism_db_core::driver::driver::DbDriver;
use prism_db_core::driver::executor::DbExecutor;
use prism_db_core::types::DbResult;
use async_trait::async_trait;

pub struct MyCustomDriver {
    // Your backend connection/pool
}

#[async_trait]
impl DbDriver for MyCustomDriver {
    async fn begin_transaction(&self) -> DbResult<Arc<dyn DbTransaction>> {
        // Implement transaction start for your backend
        todo!()
    }

    async fn ping(&self) -> DbResult<()> {
        // Implement health check
        todo!()
    }
}

#[async_trait]
impl DbExecutor for MyCustomDriver {
    fn prepare_find(&self, query: FindQuery) -> DbResult<Box<dyn PreparedFindQuery + '_>> {
        // Adapt FindQuery to your backend's query language
        todo!()
    }

    fn prepare_insert(&self, query: InsertQuery) -> DbResult<Box<dyn PreparedInsertQuery + '_>> {
        // Implement insert for your backend
        todo!()
    }

    // ... implement other methods ...
}
```

Once implemented, your custom driver integrates seamlessly with the entire ecosystem.

---

## Testing

### Run All Tests

```bash
cargo test
```

### Run Unit Tests Only

```bash
cargo test --lib
```

### Run Integration Tests (with environment variables set)

```bash
export TEST_SQLITE_URL=sqlite://:memory:
cargo test --test sqlite_integration -- --nocapture

export TEST_POSTGRES_URL=postgres://localhost/testdb
cargo test --test postgres_integration

export TEST_MYSQL_URL=mysql://localhost/testdb
cargo test --test mysql_integration
```

### Test Coverage (via `tarpaulin`, if installed)

```bash
cargo tarpaulin --out Html --output-dir coverage
```

---

## Contributing

We welcome contributions! Please follow these guidelines:

### 1. Fork and Clone

```bash
git clone https://github.com/your-fork/Prism-DB.git
cd Prism-DB
```

### 2. Create a Feature Branch

```bash
git checkout -b feat/my-feature
```

### 3. Make Your Changes

- Follow Rust naming conventions and idioms
- Add doc comments to public APIs
- Include unit tests for logic; add integration tests for driver-specific features
- Run `cargo fmt` and `cargo clippy` before committing

```bash
cargo fmt
cargo clippy --all --all-targets
cargo test
```

### 4. Commit and Push

```bash
git add .
git commit -m "feat: add new feature"
git push origin feat/my-feature
```

### 5. Open a Pull Request

- Provide a clear description of changes
- Link any related issues
- Ensure CI passes (unit tests, formatting, clippy)

### Code Style

- **Formatting**: `rustfmt` (run via `cargo fmt`)
- **Linting**: `clippy` (run via `cargo clippy --all --all-targets`)
- **Documentation**: All public items require doc comments with examples
- **Error Handling**: Use `Result<T, DbError>` for fallible operations

---

## Roadmap

**SQL Drivers** (Core)
- [x] PostgreSQL driver (sqlx-based)
- [x] MySQL driver (sqlx-based)
- [x] SQLite driver (sqlx-based)

**NoSQL Drivers** (Coming soon)
- [ ] MongoDB driver (official driver)
- [ ] DynamoDB driver (AWS SDK)
- [ ] Cassandra driver (datastax-rust-driver)
- [ ] Firestore driver (google-cloud-firestore)

**Cache & Message Queue Drivers** (Future)
- [ ] Redis driver (redis-rs)
- [ ] Memcached driver (memcache crate)
- [ ] RabbitMQ driver (lapin)

**Core Enhancements**
- [ ] Async stream cursors with backpressure
- [ ] Query result caching layer
- [ ] Migration framework (schema versioning)
- [ ] Relationship eager loading (JOINs DSL)
- [ ] Performance benchmarks suite
- [ ] Custom driver starter template / scaffold tool

---

## License

This project is licensed under the **MIT License**â€”see [LICENSE](./LICENSE) for details.

---

## Support

- **Documentation**: Check the [docs](./docs/) directory for detailed guides.
- **Issues**: Report bugs or feature requests on [GitHub Issues](https://github.com/your-org/Prism-DB/issues).
- **Discussions**: Join our community on [GitHub Discussions](https://github.com/your-org/Prism-DB/discussions).

---

## Acknowledgments

Built with â¤ï¸ using [Rust](https://www.rust-lang.org)
