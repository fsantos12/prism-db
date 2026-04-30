use prism_db::DbEntity;

/// Test entity mapped to the `test_users` table.
///
/// Used by all CRUD and ORM test/bench functions in this crate.
/// Requires the table created by [`PrismTestContext::prepare_database`].
///
/// [`PrismTestContext::prepare_database`]: crate::context::PrismTestContext::prepare_database
#[derive(DbEntity, Clone, Debug, PartialEq)]
#[db(table = "test_users")]
pub struct TestUser {
    #[db(primary_key)]
    pub id: i64,
    pub name: String,
    pub email: String,
    pub age: i32,
}
