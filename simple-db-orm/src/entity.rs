use async_trait::async_trait;
use simple_db_core::{driver::executor::DbExecutor, query::{DeleteQuery, FilterDefinition, InsertQuery, UpdateQuery}, types::{DbResult, DbRow, DbValue}};

/// Core ORM trait for mapping between Rust types and database rows.
///
/// Implementors define:
/// - The database table name via `table_name()`
/// - The primary key composition via `primary_key()`
/// - Field serialization via `to_db()`
/// - Row deserialization via `from_db()`
///
/// The primary key is used for change tracking and automatic WHERE clause generation.
#[async_trait]
pub trait DbEntityTrait: Clone {
    /// Returns the database table name for this entity type.
    fn table_name() -> &'static str;

    /// Returns the primary key field names and current values as a vector of tuples.
    ///
    /// Used for change tracking and generating WHERE clauses on UPDATE/DELETE.
    fn primary_key(&self) -> Vec<(&'static str, DbValue)>;

    /// Serializes the entity into database field names and values.
    ///
    /// This includes the primary key fields. Fields in this list matching the primary
    /// key names are excluded from UPDATE queries by the change tracking logic.
    fn to_db(&self) -> Vec<(&'static str, DbValue)>;

    /// Deserializes a database row into an entity instance.
    fn from_db(row: &dyn DbRow) -> Self;

    /// Generates a filter for this entity's primary key.
    ///
    /// Used by `save()` and `delete()` to target the correct row in the database.
    /// Overridable for complex primary key scenarios.
    fn primary_key_filter(&self) -> FilterDefinition {
        use simple_db_core::query::FilterBuilder;
        let pk = self.primary_key();
        pk.into_iter()
            .fold(FilterBuilder::new(), |builder, (key, val)| builder.eq(key, val))
            .build()
    }
}

/// Entity change tracking state machine.
///
/// - **Untracked**: A new entity created in memory, not yet persisted.
/// - **Tracked**: An entity loaded from the database; change tracking compares against this original.
/// - **Detached**: A read-only entity (loaded via `from_db_readonly()`) or a deleted entity.
#[derive(Debug, Clone)]
pub enum TrackingState<T> {
    /// Entity is new and has never been saved.
    Untracked,
    /// Entity was loaded from the database; stores the original values for change detection.
    Tracked(T),
    /// Entity is read-only or has been deleted.
    Detached,
}

impl<T> TrackingState<T> {
    /// Returns `true` if the entity is tracked.
    pub fn is_tracked(&self) -> bool {
        matches!(self, TrackingState::Tracked(_))
    }

    /// Returns `true` if the entity is untracked (newly created).
    pub fn is_untracked(&self) -> bool {
        matches!(self, TrackingState::Untracked)
    }

    /// Returns `true` if the entity is detached (read-only or deleted).
    pub fn is_detached(&self) -> bool {
        matches!(self, TrackingState::Detached)
    }

    /// Returns the original tracked values if the state is `Tracked`, else `None`.
    pub fn original(&self) -> Option<&T> {
        match self {
            TrackingState::Tracked(original) => Some(original),
            _ => None,
        }
    }
}

/// A wrapper around an entity with change tracking and persistence support.
///
/// `DbEntity<T>` tracks whether an entity is new, modified, or read-only,
/// and provides methods to `save()` and `delete()` it from the database.
///
/// # Example
///
/// ```ignore
/// let mut user = DbEntity::new(User { id: 1, name: "Alice".to_string() });
/// user.save(&driver).await?;  // INSERT
///
/// user.get_mut().name = "Bob".to_string();
/// user.save(&driver).await?;  // UPDATE
///
/// user.delete(&driver).await?;  // DELETE
/// ```
#[derive(Debug, Clone)]
pub struct DbEntity<T: DbEntityTrait> {
    value: T,
    state: TrackingState<T>,
}

impl<T: DbEntityTrait> DbEntity<T> {
    /// Creates a new untracked entity.
    pub fn new(entity: T) -> Self {
        Self {
            value: entity,
            state: TrackingState::Untracked,
        }
    }

    /// Creates an entity from a database row with tracking enabled.
    ///
    /// The returned entity tracks the loaded values for change detection.
    pub fn from_db(row: &dyn DbRow) -> Self {
        let entity = T::from_db(row);
        Self {
            value: entity.clone(),
            state: TrackingState::Tracked(entity),
        }
    }

    /// Creates a read-only entity from a database row.
    ///
    /// Read-only entities cannot be saved or deleted (operations return `Ok(())`).
    pub fn from_db_readonly(row: &dyn DbRow) -> Self {
        let entity = T::from_db(row);
        Self {
            value: entity,
            state: TrackingState::Detached,
        }
    }

    /// Returns a reference to the entity value.
    pub fn get(&self) -> &T {
        &self.value
    }

    /// Returns a mutable reference to the entity value.
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.value
    }

    /// Consumes the wrapper and returns the inner entity.
    pub fn into_inner(self) -> T {
        self.value
    }

    /// Returns the current tracking state.
    pub fn get_state(&self) -> &TrackingState<T> {
        &self.state
    }

    /// Returns `true` if the entity is tracked (loaded from database).
    pub fn is_tracked(&self) -> bool {
        self.state.is_tracked()
    }

    /// Returns `true` if the entity is untracked (newly created).
    pub fn is_untracked(&self) -> bool {
        self.state.is_untracked()
    }

    /// Returns `true` if the entity is detached (read-only or deleted).
    pub fn is_detached(&self) -> bool {
        self.state.is_detached()
    }

    /// Persists the entity to the database.
    ///
    /// Behavior depends on the current state:
    /// - **Untracked**: Executes an INSERT query.
    /// - **Tracked**: Compares current values against originals and executes an UPDATE for changed fields.
    /// - **Detached**: No-op (returns `Ok(())`).
    ///
    /// After successful save, untracked entities become tracked.
    pub async fn save(&mut self, executor: &dyn DbExecutor) -> DbResult<()> where T: PartialEq {
        match &self.state {
            TrackingState::Untracked => {
                let fields = self.value.to_db();
                let row: Vec<(String, DbValue)> = fields
                    .into_iter()
                    .map(|(field, value)| (field.to_string(), value))
                    .collect();
                let insert_query = InsertQuery::new(T::table_name()).insert(row);
                executor.insert(insert_query).await?;
                self.state = TrackingState::Tracked(self.value.clone());
                Ok(())
            },
            TrackingState::Tracked(original) => {
                let current_fields = self.value.to_db();
                let original_fields = original.to_db();
                let pk_names: Vec<&str> = self.value.primary_key()
                    .into_iter()
                    .map(|(name, _)| name)
                    .collect();

                let changed_fields: Vec<(String, DbValue)> = current_fields
                    .iter()
                    .zip(original_fields.iter())
                    .filter(|(current, orig)| !pk_names.contains(&current.0) && current.1 != orig.1)
                    .map(|(current, _)| (current.0.to_string(), current.1.clone()))
                    .collect();

                if !changed_fields.is_empty() {
                    let filter = self.value.primary_key_filter();
                    let mut update_query = UpdateQuery::new(T::table_name()).filter(filter);
                    for (field, value) in changed_fields {
                        update_query = update_query.set(field, value);
                    }
                    executor.update(update_query).await?;
                }

                self.state = TrackingState::Tracked(self.value.clone());
                Ok(())
            },
            TrackingState::Detached => Ok(()),
        }
    }

    /// Deletes the entity from the database.
    ///
    /// Only tracked entities can be deleted. Detached or untracked entities are no-ops.
    /// After successful deletion, the entity becomes detached.
    pub async fn delete(&mut self, executor: &dyn DbExecutor) -> DbResult<()> {
        if self.is_tracked() {
            let filter = self.value.primary_key_filter();
            let query = DeleteQuery::new(T::table_name()).filter(filter);
            executor.delete(query).await?;
            self.state = TrackingState::Detached;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracking_state_queries() {
        let tracked = TrackingState::Tracked("original".to_string());
        assert!(tracked.is_tracked());
        assert!(!tracked.is_untracked());
        assert!(!tracked.is_detached());
        assert_eq!(tracked.original(), Some(&"original".to_string()));

        let untracked = TrackingState::<String>::Untracked;
        assert!(!untracked.is_tracked());
        assert!(untracked.is_untracked());
        assert!(!untracked.is_detached());
        assert_eq!(untracked.original(), None);

        let detached = TrackingState::<String>::Detached;
        assert!(!detached.is_tracked());
        assert!(!detached.is_untracked());
        assert!(detached.is_detached());
        assert_eq!(detached.original(), None);
    }
}