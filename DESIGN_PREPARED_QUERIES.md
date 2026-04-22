# Prepared Queries Design Proposal

This document outlines a design for implementing cached/prepared queries in `simple-db` in a way that is highly performant and completely database-agnostic. 

## The Problem
Currently, when a user executes a `FindQuery` (or any other query), the driver must parse the AST and compile it into the database's native format (e.g., a SQL string) every single time. For applications that execute the exact same query repeatedly, this adds unnecessary CPU overhead.

Additionally, we want the abstraction to support SQL databases (MySQL, Postgres, SQLite) as well as NoSQL or key-value stores (MongoDB, Redis), without coupling `simple-db-core` to `String` SQL queries.

## The Solution: `PreparedQuery` Traits

By returning a Boxed trait from the core `DbExecutor`, we allow each driver to cache the compiled query in its own **native format**. The core library remains unaware of whether that format is a SQL string, a MongoDB BSON document, or a Redis command array.

### 1. Core Abstractions (`simple-db-core/src/driver/executor.rs`)

We introduce new traits for prepared queries. These represent a query that has already been compiled by the driver and is ready to execute.

```rust
use async_trait::async_trait;
use crate::types::{DbCursor, DbResult};

/// A pre-compiled SELECT query that can be executed repeatedly.
#[async_trait]
pub trait PreparedFindQuery: Send + Sync {
    /// Executes the prepared query using its cached internal state.
    async fn execute(&self) -> DbResult<Box<dyn DbCursor>>;
    
    // Future expansion: 
    // async fn execute_with(&self, params: Vec<DbValue>) -> DbResult<Box<dyn DbCursor>>;
}

#[async_trait]
pub trait DbExecutor: Send + Sync {
    // ... existing methods (find, insert, update, delete) ...

    /// Prepares a find query in the driver's native format for repeated execution.
    async fn prepare_find(&self, query: crate::query::FindQuery) -> DbResult<Box<dyn PreparedFindQuery>>;
}
```

### 2. SQL Implementation Example (MySQL / Postgres)

For SQL databases, the driver caches the generated SQL string and the extracted values.

```rust
pub struct SqlPreparedFind {
    // Caches the compiled string (e.g., "SELECT * FROM users WHERE age = ?")
    sql: String,
    // Caches the bound values extracted from the AST
    values: Vec<DbValue>,
    pool: sqlx::MySqlPool, 
}

#[async_trait]
impl PreparedFindQuery for SqlPreparedFind {
    async fn execute(&self) -> DbResult<Box<dyn DbCursor>> {
        // Just bind the saved values to the saved SQL. 
        // No AST traversal or string building overhead!
        let mut query = sqlx::query(&self.sql);
        for value in &self.values {
            query = bind_value(query, value);
        }
        let rows = query.fetch_all(&self.pool).await?;
        // Return DbCursor...
    }
}
```

### 3. MongoDB Implementation Example

For MongoDB, the driver compiles the AST into a BSON document (or aggregation pipeline) and caches that document. No strings involved!

```rust
use bson::Document;

pub struct MongoPreparedFind {
    // Caches the compiled BSON document instead of a SQL string!
    pipeline: Document, 
    collection: mongodb::Collection<Document>,
}

#[async_trait]
impl PreparedFindQuery for MongoPreparedFind {
    async fn execute(&self) -> DbResult<Box<dyn DbCursor>> {
        // Run the cached BSON document directly against Mongo
        let cursor = self.collection.aggregate(self.pipeline.clone(), None).await?;
        // Return DbCursor...
    }
}
```

### 4. Redis Implementation Example

For Redis, the driver compiles the AST into a raw Redis command vector.

```rust
pub struct RedisPreparedFind {
    // Caches a pre-built Redis command struct
    command: redis::cmd::Cmd,
    con: redis::aio::Connection, // simplified for example
}

#[async_trait]
impl PreparedFindQuery for RedisPreparedFind {
    async fn execute(&self) -> DbResult<Box<dyn DbCursor>> {
        // Executes the cached Redis command directly
        let result: redis::Value = self.command.query_async(&mut self.con).await?;
        // Return DbCursor...
    }
}
```

## Summary
The `PreparedFindQuery` abstraction provides zero-cost abstractions for the user while letting every driver heavily optimize its hot path by caching exactly the native structures it needs.
