# simple-db — Roadmap

## Code Improvements

### 1. `DbError` is too opaque — `simple-db-core/src/error.rs`
`Box<dyn DbError>` can't be matched or downcast without `Any`. Users can't distinguish a connection error from a type error. Consider an enum at the core level:
```rust
pub enum DbError {
    Type(TypeError),
    Driver(Box<dyn std::error::Error + Send + Sync>),
    // ...
}
```

### 2. `DbCursor::next` allocates on every row — `simple-db-core/src/cursor.rs`
`Box<dyn DbRow>` is returned per row. Consider a `GenericRow` concrete type or `SmallVec<DbValue>` that avoids the vtable dispatch for the common case.

### 3. `DbTransactionalDriver` passes `Arc<dyn DbDriver>` to the closure — `simple-db-driver/src/lib.rs`
The `Arc` inside a transaction is the same as outside — there's no isolation. A `DbTransaction` type (wrapping a dedicated connection) would make the semantics correct and prevent accidental use of the outer driver mid-transaction.

### 4. `FilterBuilder` implicit AND only — `simple-db-query/src/filter.rs`
OR-groups at the top level aren't expressible today. `FilterCondition::Or` exists but the builder only exposes `.and()/.or()` as nested groups. A top-level `FilterBuilder::or_group(|b| ...)` would complete this.

### 5. `InsertQuery` clones column names per row — `simple-db-query/src/insert.rs`
Rows are stored as `Vec<Vec<(String, DbValue)>>`, cloning column names each time. A column-oriented layout `(columns: Vec<String>, rows: Vec<Vec<DbValue>>)` is more memory-efficient for bulk inserts.

### 6. `regex` in `FilterCondition` — `simple-db-query/src/filter.rs`
`regex` is a heavy dependency but SQLite doesn't natively support `REGEXP` without a loaded extension. This feature should be behind a cargo feature flag.

---

## Roadmap

### Phase 1 — Solidify the core
- [ ] Fix `DbError`: make it a concrete enum, not `Box<dyn trait>`
- [ ] Add `DbTransaction` type (separate from `DbDriver`)
- [ ] Add query-to-SQL round-trip tests per driver

### Phase 2 — ORM layer
- [ ] `#[derive(DbModel)]` → maps struct fields to columns
- [ ] `FromRow` trait → convert `DbRow` → struct automatically
- [ ] `DbModel::find()/insert()/update()/delete()` convenience methods

### Phase 3 — Driver completeness
- [ ] PostgreSQL driver (re-use SQLite builder, swap sqlx backend)
- [ ] MySQL driver
- [ ] Connection pool support (`sqlx::Pool` integration)

### Phase 4 — Query power
- [ ] JOIN builder (INNER, LEFT, RIGHT)
- [ ] Subquery support
- [ ] Raw SQL escape hatch: `driver.raw("SELECT ...", params)`

### Phase 5 — Developer experience
- [ ] Query logging middleware (hook before/after execute)
- [ ] Schema migrations (simple up/down file runner)
- [ ] Compile-time query validation (macro that checks against a schema snapshot)
