use crate::{DbContext, query::{Query, filters::{FilterBuilder, FilterDefinition}}, types::{DbError, DbRow, DbValue, FromDbRow}};

pub enum DbEntityState {
    /// Entity is new and not yet in the DB.
    Added,
    /// Entity is known to the DB and tracked for changes.
    Tracked,
    /// Entity has been marked for removal or deleted.
    Deleted,
    /// Entity is no longer managed by the context.
    Detached,
}

pub type DbEntityKey = Vec<(String, DbValue)>;

pub trait DbEntityModel: FromDbRow + Into<DbRow> + Send + Sync + Clone + 'static {
    fn collection_name() -> &'static str;
    fn key(&self) -> DbEntityKey;

    /// Generates a database filter safely based on the key fields. Throws an error if the key is empty to prevent mass operations.
    fn key_filter(&self) -> Result<FilterDefinition, DbError> {
        let key_pairs = self.key();
        if key_pairs.is_empty() {
            return Err(DbError::MappingError(format!(
                "Entity '{}' provided an empty key. Operations aborted to prevent accidental mass-deletion/update.",
                Self::collection_name()
            )));
        }

        let mut filter = FilterBuilder::new();
        for (field, value) in key_pairs {
            filter = filter.eq(field, value);
        }

        Ok(filter.build())
    }
}

pub struct DbEntity<T: DbEntityModel> {
    pub entity: T,
    snapshot: Option<DbRow>,
    state: DbEntityState
}

impl<T: DbEntityModel> DbEntity<T> {
    pub fn new(entity: T) -> Self {
        Self {
            entity,
            snapshot: None,
            state: DbEntityState::Added
        }
    }

    /// Internal: Wraps data loaded from the database.
    pub(crate) fn from_db(entity: T, row: DbRow) -> Self {
        Self {
            entity,
            snapshot: Some(row),
            state: DbEntityState::Tracked,
        }
    }

    fn dirty_fields(&self) -> DbRow {
        let current: DbRow = self.entity.clone().into();
        let mut updates = DbRow::new();

        if let Some(ref original) = self.snapshot {
            for (field, val) in &current.0 {
                if original.get(field)!= Some(val) {
                    updates.insert(field.clone(), val.clone());
                }
            }
        }

        updates
    }

    /// Persists changes to the database.
    pub async fn save(&mut self, ctx: &DbContext) -> Result<(), DbError> {
        match self.state {
            DbEntityState::Added => {
                let row: DbRow = self.entity.clone().into();
                let q = Query::insert(T::collection_name()).insert(row.clone());
                ctx.insert(q).await?;
                self.snapshot = Some(row);
                self.state = DbEntityState::Tracked;
            }
            DbEntityState::Tracked => {
                let updates = self.dirty_fields();
                if!updates.0.is_empty() {
                    let q = Query::update(T::collection_name())
                       .set_row(updates)
                       .with_filters(self.entity.key_filter()?);
                    ctx.update(q).await?;
                    self.snapshot = Some(self.entity.clone().into());
                }
            }
            DbEntityState::Detached => return Err(DbError::MappingError("Cannot save detached entity".into())),
            _ => {}
        }
        Ok(())
    }

    /// Removes the record from the database.
    pub async fn delete(mut self, ctx: &DbContext) -> Result<(), DbError> {
        let q = Query::delete(T::collection_name())
           .with_filters(self.entity.key_filter()?);
        ctx.delete(q).await?;
        self.state = DbEntityState::Deleted;
        Ok(())
    }

    pub fn state(&self) -> &DbEntityState { &self.state }
    pub fn detach(&mut self) { self.state = DbEntityState::Detached; }
}