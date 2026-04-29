use async_trait::async_trait;
use simple_db_core::types::{DbCursor, DbResult};

use crate::entity::DbEntityTrait;

#[async_trait]
pub trait DbCursorEntityExt {
    async fn next_entity<T: DbEntityTrait>(&mut self) -> DbResult<Option<T>>;
}

#[async_trait]
impl<C: DbCursor + ?Sized> DbCursorEntityExt for C {
    async fn next_entity<T: DbEntityTrait>(&mut self) -> DbResult<Option<T>> {
        if let Some(row) = self.next().await? {
            Ok(Some(T::from_db(row.as_ref())))
        } else {
            Ok(None)
        }
    }
}
