use async_trait::async_trait;

use crate::{query::{DeleteQuery, FindQuery, InsertQuery, PreparedDeleteQuery, PreparedFindQuery, PreparedInsertQuery, PreparedUpdateQuery, UpdateQuery}, types::{DbCursor, DbResult}};

#[async_trait]
pub trait DbExecutor: Send + Sync {
    // --- Query Builders ---
    fn prepare_find(&self, query: FindQuery) -> DbResult<Box<dyn PreparedFindQuery + '_>>;
    async fn find(&self, query: FindQuery) -> DbResult<Box<dyn DbCursor>>{
        self.prepare_find(query)?.execute().await
    }

    fn prepare_insert(&self, query: InsertQuery) -> DbResult<Box<dyn PreparedInsertQuery + '_>>;
    async fn insert(&self, query: InsertQuery) -> DbResult<u64> {
        self.prepare_insert(query)?.execute().await
    }

    fn prepare_update(&self, query: UpdateQuery) -> DbResult<Box<dyn PreparedUpdateQuery + '_>>;
    async fn update(&self, query: UpdateQuery) -> DbResult<u64> {
        self.prepare_update(query)?.execute().await
    }

    fn prepare_delete(&self, query: DeleteQuery) -> DbResult<Box<dyn PreparedDeleteQuery + '_>>;
    async fn delete(&self, query: DeleteQuery) -> DbResult<u64> {
        self.prepare_delete(query)?.execute().await
    }
}