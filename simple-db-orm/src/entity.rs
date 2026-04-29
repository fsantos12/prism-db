use async_trait::async_trait;
use simple_db_core::{driver::executor::DbExecutor, query::{DeleteQuery, FilterDefinition, InsertQuery, UpdateQuery}, types::{DbResult, DbRow, DbValue}};

#[async_trait]
pub trait DbEntityTrait: Clone {
    fn table_name() -> &'static str;
    fn primary_key(&self) -> Vec<(&'static str, DbValue)>;

    fn to_db(&self) -> Vec<(&'static str, DbValue)>;
    fn from_db(row: &dyn DbRow) -> Self;

    fn primary_key_filter(&self) -> FilterDefinition {
        use simple_db_core::query::FilterBuilder;
        let pk = self.primary_key();
        pk.into_iter()
            .fold(FilterBuilder::new(), |builder, (key, val)| builder.eq(key, val))
            .build()
    }
}

#[derive(Debug, Clone)]
pub enum TrackingState<T> {
    Untracked,
    Tracked(T),
    Detached,
}

impl<T> TrackingState<T> {
    pub fn is_tracked(&self) -> bool {
        matches!(self, TrackingState::Tracked(_))
    }

    pub fn is_untracked(&self) -> bool {
        matches!(self, TrackingState::Untracked)
    }

    pub fn is_detached(&self) -> bool {
        matches!(self, TrackingState::Detached)
    }

    pub fn original(&self) -> Option<&T> {
        match self {
            TrackingState::Tracked(original) => Some(original),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DbEntity<T: DbEntityTrait> {
    value: T,
    state: TrackingState<T>,
}

impl<T: DbEntityTrait> DbEntity<T> {
    // =========================================================================
    // CONSTRUCTORS
    // =========================================================================
    pub fn new(entity: T) -> Self {
        Self {
            value: entity,
            state: TrackingState::Untracked,
        }
    }

    pub fn from_db(row: &dyn DbRow) -> Self {
        let entity = T::from_db(row);
        Self {
            value: entity.clone(),
            state: TrackingState::Tracked(entity),
        }
    }

    pub fn from_db_readonly(row: &dyn DbRow) -> Self {
        let entity = T::from_db(row);
        Self {
            value: entity,
            state: TrackingState::Detached,
        }
    }

    // =========================================================================
    // GETTERS
    // =========================================================================
    pub fn get(&self) -> &T {
        &self.value
    }

    pub fn get_mut(&mut self) -> &mut T {
        &mut self.value
    }

    pub fn into_inner(self) -> T {
        self.value
    }

    // =========================================================================
    // STATE
    // =========================================================================
    pub fn get_state(&self) -> &TrackingState<T> {
        &self.state
    }

    pub fn is_tracked(&self) -> bool {
        self.state.is_tracked()
    }

    pub fn is_untracked(&self) -> bool {
        self.state.is_untracked()
    }

    pub fn is_detached(&self) -> bool {
        self.state.is_detached()
    }

    // =========================================================================
    // CUD - CREATE, UPDATE, DELETE
    // =========================================================================
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