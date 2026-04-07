use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::{
    types::DbError,
    driver::driver::Driver,
    entity::entity::{DbEntityState, DbEntity, DbEntityModel, DbTrackedEntity},
    entity::entity_query_builder::EntityQueryBuilder
};

pub struct DbContext {
    pub driver: Arc<dyn Driver>,
    // The tracker hides the specific types behind Box<dyn DbTrackedEntity>
    changes: RwLock<HashMap<String, Box<dyn DbTrackedEntity>>>,
}

impl DbContext {
    /// Initializes a new Database Context with the provided driver
    pub fn new(driver: Arc<dyn Driver>) -> Self {
        Self {
            driver,
            changes: RwLock::new(HashMap::new()),
        }
    }

    /// Internal method to store the boxed entity
    pub(crate) fn register_change(&self, hash_key: String, entity: Box<dyn DbTrackedEntity>) {
        self.changes.write().unwrap().insert(hash_key, entity);
    }

    // ==========================================
    // THE NEW, DIRECT API (Replaces DbSet)
    // ==========================================
    pub fn query<T: DbEntityModel>(&self) -> EntityQueryBuilder<'_, T> {
        EntityQueryBuilder::new(self)
    }

    /// Creates a brand new tracked entity
    pub fn create_entity<T: DbEntityModel>(&self, model: T) -> DbEntity<T> {
        DbEntity::new(model)
    }

    /// Explicitly mark an entity for INSERT
    pub fn add<T: DbEntityModel>(&self, mut entity: DbEntity<T>) {
        // We use the trait method to update the state
        
        DbTrackedEntity::set_state(&mut entity, DbEntityState::Added);
        let hash = entity.inner.key_hash();
        self.register_change(hash, Box::new(entity));
    }

    /// Explicitly mark an entity for UPDATE
    pub fn update<T: DbEntityModel>(&self, mut entity: DbEntity<T>) {
        DbTrackedEntity::set_state(&mut entity, DbEntityState::Modified);
        let hash = entity.inner.key_hash();
        self.register_change(hash, Box::new(entity));
    }

    /// Explicitly mark an entity for DELETE
    pub fn remove<T: DbEntityModel>(&self, mut entity: DbEntity<T>) {
        DbTrackedEntity::set_state(&mut entity, DbEntityState::Deleted);
        let hash = entity.inner.key_hash();
        self.register_change(hash, Box::new(entity));
    }

    // ==========================================
    // SAVING
    // ==========================================

    /// Executes all pending database operations in the tracker
    pub async fn save_changes(&self) -> Result<usize, DbError> {
        let mut tracker = self.changes.write().unwrap();
        let saved_count = tracker.len();

        for (_, entity) in tracker.iter_mut() {
            entity.save_to_db(self.driver.as_ref()).await?;
            entity.set_state(DbEntityState::Unchanged);
        }

        tracker.clear();
        Ok(saved_count)
    }

    /// Wipes out any unsaved changes from the memory tracker
    pub fn discard_changes(&self) {
        self.changes.write().unwrap().clear();
    }
}