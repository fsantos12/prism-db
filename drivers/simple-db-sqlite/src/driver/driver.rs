use std::sync::Arc;

use async_trait::async_trait;
use simple_db_core::{driver::{DbDriver, DbExecutor, DbTransaction}, query::{DeleteQuery, FindQuery, InsertQuery, UpdateQuery}, types::{DbCursor, DbError, DbResult}};
use sqlx::SqlitePool;

pub struct SqliteDriver {
    pub pool: SqlitePool,
}

// ==========================================
// 1. Implement the generic execution trait
// ==========================================
#[async_trait]
impl DbExecutor for SqliteDriver {
    async fn find(&self, query: FindQuery) -> DbResult<Box<dyn DbCursor>> {
        // Your logic using sqlx here...
        todo!()
    }

    async fn insert(&self, query: InsertQuery) -> DbResult<u64> {
        todo!()
    }

    async fn update(&self, query: UpdateQuery) -> DbResult<u64> {
        todo!()
    }

    async fn delete(&self, query: DeleteQuery) -> DbResult<u64> {
        todo!()
    }
}

// ==========================================
// 2. Implement the Driver-specific trait
// ==========================================
#[async_trait]
impl DbDriver for SqliteDriver {
    
    async fn begin(&self) -> DbResult<Arc<dyn DbTransaction>> {
        // Here we will ask sqlx to begin a transaction
        todo!()
    }

    async fn ping(&self) -> DbResult<()> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Driver(Box::new(e)))?;
        Ok(())
    }
}