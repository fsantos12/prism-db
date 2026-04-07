# simple-db

A type-safe, async-first query builder and ORM for Rust with in-memory and pluggable database drivers.

## Features

- **Type-Safe Queries**: Fluent API with builder pattern for constructing queries with compile-time safety
- **Async/Await**: Native Tokio integration for async database operations and non-blocking execution
- **Change Tracking**: Automatic entity state management (Added, Tracked, Deleted, Detached) for ORM operations
- **Flexible Filtering**: Rich filter API supporting null checks, comparisons, pattern matching, ranges, set membership, and logical operators
- **Memory Driver**: In-memory database implementation for testing and prototyping without external dependencies
- **Driver Abstraction**: Pluggable architecture for adding new database backends (PostgreSQL, MongoDB, etc.)
- **Comprehensive Type Support**: Primitives, temporal types (Date, Time, Timestamp), and specialized types (UUID, Decimal, JSON)
- **Aggregation Functions**: Built-in support for COUNT, SUM, AVG, MIN, MAX operations
- **Transactions**: Full ACID transaction support with proper rollback handling
- **Pagination**: Efficient result set pagination with offset and limit support
- **Sorting**: Advanced sorting with null placement control and random ordering

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
simple-db = "0.1.0"
tokio = { version = "1.51", features = ["full"] }
```

## Quick Start

### Basic Setup

```rust
use simple_db::{DbContext, driver::memory::MemoryDriver, query::Query};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create an in-memory driver
    let driver = Arc::new(MemoryDriver::new());
    let ctx = DbContext::new(driver);

    // Insert some data
    let insert_query = Query::insert("users")
        .insert([("id", 1), ("name", "Alice"), ("age", 30)])
        .insert([("id", 2), ("name", "Bob"), ("age", 25)])
        .insert([("id", 3), ("name", "Charlie"), ("age", 35)]);

    ctx.insert(insert_query).await?;

    // Find users
    let find_query = Query::find("users")
        .filter(|fb| fb.gte("age", 30))
        .order_by(|sb| sb.asc("name"));

    let results = ctx.find(find_query).await?;
    println!("Found {} users", results.len());

    Ok(())
}
```

## Core Concepts

### Queries

The library provides four main query types:

#### Find Query

Select and filter records from a collection:

```rust
let query = Query::find("users")
    .project(|pb| pb.field("name").field("email"))
    .filter(|fb| fb
        .is_not_null("email")
        .and(|b| b
            .gte("age", 18)
            .lt("age", 65)
        )
    )
    .order_by(|sb| sb.asc("name"))
    .limit(10)
    .offset(0);

let results = ctx.find(query).await?;
```

#### Insert Query

Add new records to a collection:

```rust
let query = Query::insert("users")
    .insert([("id", 1), ("name", "Alice"), ("email", "alice@example.com")])
    .insert([("id", 2), ("name", "Bob"), ("email", "bob@example.com")]);

let count = ctx.insert(query).await?;
println!("Inserted {} records", count);
```

#### Update Query

Modify existing records:

```rust
let query = Query::update("users")
    .set("age", 31)
    .filter(|fb| fb.eq("id", 1));

let count = ctx.update(query).await?;
println!("Updated {} records", count);
```

#### Delete Query

Remove records from a collection:

```rust
let query = Query::delete("users")
    .filter(|fb| fb.lt("age", 18));

let count = ctx.delete(query).await?;
println!("Deleted {} records", count);
```

### Filtering

The filter API supports comprehensive conditional logic:

#### Null Checks

```rust
.filter(|fb| fb
    .is_null("phone")        // Phone is NULL
    .is_not_null("email")    // Email is NOT NULL
)
```

#### Basic Comparisons

```rust
.filter(|fb| fb
    .eq("status", "active")     // Equals
    .neq("role", "admin")       // Not equals
    .lt("score", 100)           // Less than
    .lte("score", 100)          // Less than or equal
    .gt("price", 50.0)          // Greater than
    .gte("price", 50.0)         // Greater than or equal
)
```

#### Pattern Matching

```rust
.filter(|fb| fb
    .starts_with("email", "admin")
    .not_starts_with("name", "Test")
    .ends_with("domain", ".com")
    .not_ends_with("domain", ".ru")
    .contains("description", "important")
    .not_contains("tags", "deprecated")
)
```

#### Range Checks

```rust
.filter(|fb| fb
    .between("age", 18, 65)
    .not_between("price", 100.0, 500.0)
)
```

#### Set Membership

```rust
.filter(|fb| fb
    .is_in("status", vec!["active", "pending"])
    .not_in("country", vec!["US", "CA"])
)
```

#### Logical Operators

```rust
.filter(|fb| fb
    .and(|b| b
        .eq("status", "active")
        .gte("age", 18)
    )
    .or(|b| b
        .eq("role", "admin")
        .eq("role", "moderator")
    )
)
```

### Projections

Select specific fields and apply aggregations:

```rust
let query = Query::find("orders")
    .project(|pb| pb
        .field("customer_id")
        .count("id")        // Count all orders
        .sum("amount")      // Sum of amounts
        .avg("amount")      // Average amount
        .min("created_at")  // Earliest date
        .max("created_at")  // Latest date
    )
    .group_by(|gb| gb.field("customer_id"));

let results = ctx.find(query).await?;
```

#### Aliasing

```rust
let query = Query::find("products")
    .project(|pb| pb
        .field_as("product_name", "name")
        .count_all()        // Count all rows
    );
```

### Sorting

Control result ordering with flexible null placement:

```rust
let query = Query::find("users")
    .order_by(|sb| sb
        .asc("name")                    // Ascending
        .desc("created_at")             // Descending
        .asc_nulls_first("phone")       // Ascending, nulls first
        .asc_nulls_last("phone")        // Ascending, nulls last
        .desc_nulls_first("deleted_at") // Descending, nulls first
        .random()                       // Random order
    );
```

### Pagination

Implement efficient pagination:

```rust
let page_size = 20;
let page_num = 2;
let offset = (page_num - 1) * page_size;

let query = Query::find("users")
    .order_by(|sb| sb.asc("id"))
    .limit(page_size)
    .offset(offset);

let results = ctx.find(query).await?;
```

### Transactions

Execute multiple operations atomically:

```rust
let result = ctx.transaction(|driver| async {
    // Insert operation
    let insert_q = Query::insert("audit_log")
        .insert([("action", "user_created"), ("timestamp", "2024-01-01")]);
    driver.insert(insert_q).await?;

    // Update operation
    let update_q = Query::update("users")
        .set("last_login", "2024-01-01")
        .filter(|fb| fb.eq("id", 1));
    driver.update(update_q).await?;

    Ok::<_, DbError>(())
}).await?;
```

If any operation fails, the entire transaction rolls back automatically.

### Entity Models (ORM)

Map database rows to Rust types for type-safe operations:

```rust
use simple_db::{DbEntityModel, types::{DbRow, DbValue}};

#[derive(Clone)]
struct User {
    id: i32,
    name: String,
    email: String,
    age: i32,
}

impl DbEntityModel for User {
    fn collection_name() -> &'static str {
        "users"
    }

    fn key(&self) -> Vec<(String, DbValue)> {
        vec![("id".into(), DbValue::I32(Some(self.id)))]
    }
}

// Load and track entities
let users = ctx.find_entities::<User>(
    Query::find("users")
        .filter(|fb| fb.gte("age", 18))
).await?;

// Entities are automatically tracked for changes
for mut user in users {
    user.entity.name = "Updated Name".to_string();
    user.save(&ctx).await?;  // Only changed fields are updated
}
```

## Database Types

The library supports a comprehensive set of database types:

### Primitive Types

- **Integers**: `i8`, `i16`, `i32`, `i64`, `i128`, `u8`, `u16`, `u32`, `u64`, `u128`
- **Floats**: `f32`, `f64`
- **Boolean**: `bool`
- **Character**: `char`

### Temporal Types

- **Date**: `NaiveDate` - Date without time
- **Time**: `NaiveTime` - Time without date
- **Timestamp**: `NaiveDateTime` - Date and time (no timezone)
- **TimestampTZ**: `DateTime<Utc>` - Date and time with UTC timezone

### Specialized Types

- **Decimal**: `Decimal` - Arbitrary precision decimal numbers
- **String**: `String` - UTF-8 text (boxed for memory efficiency)
- **Bytes**: `Vec<u8>` - Binary data (boxed)
- **UUID**: `Uuid` - Universally unique identifiers (boxed)
- **JSON**: `JsonValue` - Arbitrary JSON data (boxed)

All types support nullable values via `Option<T>`.

## Architecture

### Driver Interface

The `Driver` trait provides the abstraction for database backends:

```rust
pub trait Driver: Send + Sync {
    async fn find(&self, query: FindQuery) -> Result<Vec<DbRow>, DbError>;
    async fn insert(&self, query: InsertQuery) -> Result<u64, DbError>;
    async fn update(&self, query: UpdateQuery) -> Result<u64, DbError>;
    async fn delete(&self, query: DeleteQuery) -> Result<u64, DbError>;

    async fn transaction_begin(&self) -> Result<(), DbError>;
    async fn transaction_commit(&self) -> Result<(), DbError>;
    async fn transaction_rollback(&self) -> Result<(), DbError>;

    async fn ping(&self) -> Result<(), DbError>;
}
```

### Memory Driver

The built-in `MemoryDriver` is thread-safe and provides:

- **Collections**: Stored as `HashMap<String, Vec<DbRow>>`
- **Thread Safety**: `Arc<RwLock<HashMap>>` for concurrent access
- **Filtering**: AST-based evaluation of complex predicates
- **Sorting**: Multi-key sorting with null handling
- **Pagination**: Offset and limit support

Perfect for:
- Unit testing
- Integration testing
- Prototyping
- Development without database setup

### Query Builders

Each query type uses a builder pattern for fluent API construction:

- **FindQuery**: Filter, project, sort, group, paginate
- **InsertQuery**: Single and bulk inserts
- **UpdateQuery**: Set fields and apply filters
- **DeleteQuery**: Apply filters before deletion

Builders compile to immutable query objects safe for async execution.

## Error Handling

The library provides comprehensive error types via `DbError`:

```rust
pub enum DbError {
    ConnectionError(String),    // Connection failures
    QueryError(String),         // Invalid queries
    NotFound,                   // Record not found
    TypeError { expected, found },  // Type mismatches
    DriverError(Box<dyn Error>), // Driver-specific errors
    ConcurrencyError(String),   // Lock poisoning
    MappingError(String),       // Entity conversion failures
}
```

All `DbError` implements `Error + Display` for proper error handling:

```rust
match ctx.find(query).await {
    Ok(rows) => println!("Found {} rows", rows.len()),
    Err(DbError::NotFound) => println!("No rows found"),
    Err(DbError::ConnectionError(msg)) => eprintln!("Connection failed: {}", msg),
    Err(e) => eprintln!("Error: {}", e),
}
```

## Performance Considerations

### Memory Efficiency

- **Large types are boxed** (`String`, `Bytes`, `JSON`, `UUID`, `Decimal`) to reduce stack overhead
- **Type-safe comparisons** prevent unnecessary conversions

### Concurrency

- **RwLock for MemoryDriver** allows multiple concurrent readers
- **Async/await** enables non-blocking database operations
- **Clone-on-write friendly** for efficient snapshot comparison

### Query Optimization

- **Early filtering** reduces in-memory dataset size
- **Lazy projections** select only needed fields
- **Pagination** limits result set size

## Testing

Example unit test using MemoryDriver:

```rust
#[tokio::test]
async fn test_user_filtering() {
    let driver = Arc::new(MemoryDriver::new());
    let ctx = DbContext::new(driver);

    // Setup test data
    let insert = Query::insert("users")
        .insert([("id", 1), ("name", "Alice"), ("age", 30)])
        .insert([("id", 2), ("name", "Bob"), ("age", 25)])
        .insert([("id", 3), ("name", "Charlie"), ("age", 35)]);
    ctx.insert(insert).await.unwrap();

    // Test filtering
    let query = Query::find("users")
        .filter(|fb| fb.gte("age", 30));
    let results = ctx.find(query).await.unwrap();

    assert_eq!(results.len(), 2);
}
```

## Examples

Full examples are available in the `examples/` directory:

- `basic_crud.rs` - Complete CRUD operations
- `filtering.rs` - Comprehensive filtering examples
- `transactions.rs` - Transaction handling

Run examples with:

```bash
cargo run --example basic_crud
cargo run --example filtering
cargo run --example transactions
```

## Roadmap

Future enhancements:

- [ ] PostgreSQL driver implementation
- [ ] MongoDB driver implementation
- [ ] Query result caching layer
- [ ] Lazy loading for relationships
- [ ] Schema validation and migrations
- [ ] Computed/virtual columns
- [ ] Index support
- [ ] Query profiling and analysis

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- Built with [Tokio](https://tokio.rs/) for async runtime
- Uses [Serde](https://serde.rs/) for serialization
- Leverages [Rust Decimal](https://github.com/paholg/rust_decimal) for arbitrary precision
- Inspired by Entity Framework and SQLAlchemy design patterns
