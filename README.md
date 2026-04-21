# simple-db — A Lightweight, Type-Safe Database Abstraction for Rust

A modular, async-first database abstraction layer with an ORM, supporting SQLite, MySQL, and PostgreSQL.

## Table of Contents

- [Core Architecture](#core-architecture)
- [Quick Start](#quick-start)
- [Driver Usage](#driver-usage)
- [ORM Layer](#orm-layer)
- [Future Roadmap](#future-roadmap)

---

## Core Architecture

simple-db is built in layers:

### 1. **Drivers** (`simple-db-{sqlite,mysql,postgres}`)
Concrete database implementations. Each driver translates abstract queries into SQL and executes them.

- **DbDriver** trait: High-level interface for connection pools
- **DbExecutor** trait: Query execution (used inside transactions)
- **DbTransaction** trait: Transactional guarantees

### 2. **Query Builders** (`simple-db-core/query`)
Type-safe query builders that prevent invalid queries at compile time.

- **FindQuery** — SELECT with filters, projections, sorting
- **InsertQuery** — INSERT with row values
- **UpdateQuery** — UPDATE with field changes and filters
- **DeleteQuery** — DELETE with filters
- **FilterBuilder** — Type-safe WHERE clauses with AND/OR logic

### 3. **Values & Rows** (`simple-db-core/types`)
Unified value representation across all databases.

- **DbValue** — 64-bit tagged union for SQL values (int, string, date, JSON, etc.)
- **DbRow** trait — Uniform row access by index or column name
- **DbCursor** trait — Async streaming cursor for large result sets

### 4. **ORM Layer** (`simple-db-orm`)
Entity tracking with change detection.

- **DbEntity<T>** — Wrapper managing entity state (untracked, tracked, detached)
- **TrackingState** — Enum for untracked/tracked/detached states
- **DbEntityTrait** — Your entities implement this to define schema mapping
- **find()** / **find_readonly()** — Load entities with filters
- **save()** — Insert or partial update (only changed fields)
- **delete()** — Remove from database

---

## Quick Start

### 1. Define an Entity

```rust
use simple_db_orm::{DbEntity, DbEntityTrait};
use simple_db_core::types::{DbValue, DbRow};

#[derive(Clone, Debug)]
struct User {
    id: i32,
    name: String,
    email: String,
}

impl DbEntityTrait for User {
    fn collection_name() -> &'static str {
        "users"
    }

    fn primary_key(&self) -> Vec<(&'static str, DbValue)> {
        vec![("id", DbValue::from_i32(self.id))]
    }

    fn to_db(&self) -> Vec<(&'static str, DbValue)> {
        vec![
            ("id", DbValue::from_i32(self.id)),
            ("name", DbValue::from_string(self.name.clone())),
            ("email", DbValue::from_string(self.email.clone())),
        ]
    }

    fn from_db(row: &dyn DbRow) -> Self {
        User {
            id: row.get_by_name("id").and_then(|v| v.as_i32()).unwrap_or(0),
            name: row.get_by_name("name")
                .and_then(|v| v.as_string().map(|s| s.to_string()))
                .unwrap_or_default(),
            email: row.get_by_name("email")
                .and_then(|v| v.as_string().map(|s| s.to_string()))
                .unwrap_or_default(),
        }
    }
}
```

### 2. Create an Entity

```rust
let user = User {
    id: 1,
    name: "Alice".to_string(),
    email: "alice@example.com".to_string(),
};

// Create an untracked entity (new, not yet in DB)
let entity = DbEntity::new(user);
assert!(entity.is_untracked());
```

### 3. Mutate & Save

```rust
let mut entity = DbEntity::new(user);
entity.get_mut().email = "alice.new@example.com".to_string();

// Save: inserts new, or updates if tracked
entity.save(executor).await?;
```

### 4. Query & Load

```rust
// Load tracked entities (can be updated)
let users = User::find(executor, |f| f.eq("active", true)).await?;

// Load read-only (for display, no update capability)
let users_readonly = User::find_readonly(executor, |f| f.eq("active", true)).await?;
```

### 5. Delete

```rust
let mut entity = /* load from database */;
entity.delete(executor).await?;
assert!(entity.is_detached());
```

---

## Driver Usage

### SQLite

```rust
use simple_db_sqlite::SqliteDriver;

let driver = SqliteDriver::new("database.db").await?;

// Use with executor trait
let executor = driver.as_ref();
let cursor = executor.find(FindQuery::new("users")).await?;
```

### MySQL

```rust
use simple_db_mysql::MysqlDriver;

let driver = MysqlDriver::new(
    "mysql://user:password@localhost:3306/mydb"
).await?;

let executor = driver.as_ref();
```

### PostgreSQL

```rust
use simple_db_postgres::PostgresDriver;

let driver = PostgresDriver::new(
    "postgresql://user:password@localhost:5432/mydb"
).await?;

let executor = driver.as_ref();
```

### Transactions

```rust
driver.transaction(|tx| async move {
    // All queries inside use tx.as_ref()
    tx.insert(insert_query).await?;
    tx.update(update_query).await?;
    Ok(())
}).await?;
```

---

## ORM Layer

### Entity States

Every `DbEntity<T>` is in one of three states:

| State | Has Original | Can Save | Use Case |
|-------|--------------|----------|----------|
| **Untracked** | No | Yes (insert) | New entities, read-only display data |
| **Tracked** | Yes | Yes (update) | Loaded from DB, monitoring changes |
| **Detached** | No | No | Deleted entities, explicitly detached data |

### Creating Entities

```rust
// New, untracked
let user = DbEntity::new(user_struct);

// From database, tracked
let user = DbEntity::from_db(row);

// From database, detached (read-only)
let user = DbEntity::from_db_readonly(row);
```

### Accessing Values

```rust
let entity = DbEntity::new(user);

// Read
let name = entity.get().name;

// Modify
entity.get_mut().name = "Bob".to_string();

// Consume
let user = entity.into_inner();
```

### Checking State

```rust
if entity.is_untracked() {
    // New, not yet saved
}

if entity.is_tracked() {
    // From DB, can detect changes
}

if entity.is_detached() {
    // Read-only or deleted
}
```

### Save & Delete

```rust
// Insert if untracked, update only changed fields if tracked
entity.save(executor).await?;

// Delete (only works on tracked entities)
entity.delete(executor).await?;
```

### Partial Updates

When an entity is tracked, `save()` only sends changed fields to the database:

```rust
let mut entity = User::find(executor, |f| f.eq("id", 1)).await?
    .into_iter()
    .next()
    .unwrap();

// Change just the email
entity.get_mut().email = "newemail@example.com".to_string();

// Save: only updates the email column, not the entire row
entity.save(executor).await?;
```

---

## Future Roadmap

### Phase 1 — Macro Support (High Priority)
- `#[derive(DbModel)]` — Auto-generate `DbEntityTrait` for structs
- Reduce boilerplate in entity definitions
- Auto-map struct fields to database columns

### Phase 2 — Relationships (High Priority)
- One-to-many associations
- Foreign key constraints
- Lazy and eager loading options

### Phase 3 — Migrations (Medium Priority)
- Schema versioning and migration files
- Up/down migration runners
- Rollback support

### Phase 4 — Advanced Querying (Medium Priority)
- JOIN support (INNER, LEFT, RIGHT)
- Subqueries
- Aggregate functions (COUNT, SUM, AVG, etc.)
- GROUP BY / HAVING

### Phase 5 — Performance & DX (Medium Priority)
- Query logging and profiling
- Connection pooling integration
- Batch operations (bulk insert/update)
- Index management

### Phase 6 — Advanced Features (Lower Priority)
- Compile-time query validation against schema
- Auto-generated migration files from schema changes
- Soft deletes (logical delete with timestamp)
- Audit trails (track who/when for changes)

---

## License

MIT
