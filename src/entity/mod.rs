mod entity;
mod entity_query_builder;
mod context;

pub use entity::{DbEntity, DbEntityModel, DbEntityState};
pub use entity_query_builder::EntityQueryBuilder;
pub use context::DbContext;

// ==========================================
// TESTS
// ==========================================
#[cfg(test)]
mod tests {
    use crate::{driver::memory::MemoryDriver, query::filters::FilterDefinition, types::{DbError, DbRow, DbValue, FromDbRow}};

    use super::*;
    use std::{sync::Arc, time::Instant};

    // 1. Define a dummy model for testing
    #[derive(Clone, Debug, PartialEq)]
    pub struct User {
        pub id: i32,
        pub name: String,
    }

    impl Into<DbRow> for User {
        fn into(self) -> DbRow {
            let mut row = DbRow::new();
            row.insert("id".to_string(), DbValue::I32(Some(self.id)));
            row.insert("name".to_string(), DbValue::String(Some(self.name)));
            row
        }
    }

    impl FromDbRow for User {
        fn from_db_row(row: DbRow) -> Result<Self, DbError> {
            let id = match row.0.get("id") {
                Some(DbValue::I32(opt_i)) => {
                    opt_i.expect("Fatal DB Error: User ID column cannot be NULL")
                },
                _ => return Err(DbError::MappingError("Missing ID column completely".into())),
            };

            let name = match row.0.get("name") {
                Some(DbValue::String(opt_s)) => {
                    opt_s.clone().expect("Fatal DB Error: User Name column cannot be NULL")
                },
                _ => return Err(DbError::MappingError("Missing Name column completely".into())),
            };

            Ok(User { id, name })
        }
    }

    // 2. Implement your unified DbEntityModel trait
    impl DbEntityModel for User {
        fn collection_name() -> &'static str {
            "users"
        }

        fn key(&self) -> entity::DbEntityKey {
            vec![("id".to_string(), DbValue::I32(Some(self.id)))]
        }
    }

    #[tokio::test]
    async fn test_full_crud_workflow_with_metrics() {
        let driver = Arc::new(MemoryDriver::new()); 
        let context = DbContext::new(driver.clone());

        // ---------------------------------------------------------
        // 1. CREATE (Timed)
        // ---------------------------------------------------------
        let start = Instant::now();
        let new_user = context.create_entity(User { id: 1, name: "Alice".into() });
        context.add(new_user);
        let saved = context.save_changes().await.unwrap();
        let duration = start.elapsed();
        
        println!("🚀 CREATE: Saved {} entity in {:?}", saved, duration);
        assert_eq!(saved, 1);

        // ---------------------------------------------------------
        // 2. READ (Timed)
        // ---------------------------------------------------------
        let start = Instant::now();
        let fetched_users = context.query::<User>()
            .filter(FilterDefinition::empty().eq("id", 1))
            .execute()
            .await
            .unwrap();
        let duration = start.elapsed();

        println!("🔍 READ: Fetched {} entities in {:?}", fetched_users.len(), duration);
        assert_eq!(fetched_users.len(), 1);

        // ---------------------------------------------------------
        // 3. UPDATE (Timed)
        // ---------------------------------------------------------
        let mut user_to_update = fetched_users[0].clone();
        user_to_update.inner.name = "Alice Updated".into();
        
        let start = Instant::now();
        context.update(user_to_update);
        let updated = context.save_changes().await.unwrap();
        let duration = start.elapsed();

        println!("🔄 UPDATE: Updated {} entity in {:?}", updated, duration);
        assert_eq!(updated, 1);

        // ---------------------------------------------------------
        // 4. DELETE (Timed)
        // ---------------------------------------------------------
        let start = Instant::now();
        context.remove(fetched_users[0].clone());
        let deleted = context.save_changes().await.unwrap();
        let duration = start.elapsed();

        println!("🗑️ DELETE: Removed {} entity in {:?}", deleted, duration);
        assert_eq!(deleted, 1);
    }

    #[tokio::test]
    async fn test_performance_batch_load() {
        let driver = Arc::new(MemoryDriver::new()); 
        let context = DbContext::new(driver.clone());
        let count = 100_000;

        println!("🧪 Starting stress test: {} entities", count);

        // Measure Tracking Speed
        let start = Instant::now();
        for i in 0..count {
            let u = context.create_entity(User { id: i, name: format!("User-{}", i) });
            context.add(u);
        }
        println!("⏱️  Memory Tracking (Add to Context): {:?}", start.elapsed());

        // Measure Database Persistence Speed
        let start = Instant::now();
        let saved = context.save_changes().await.unwrap();
        let duration = start.elapsed();
        
        println!("💾 DB Persistence (Save Changes): {:?}", duration);
        println!("📈 Throughput: {:.2} inserts/sec", (saved as f64 / duration.as_secs_f64()));
        
        assert_eq!(saved, count as usize);
    }

}