use prism_db_core::query::{Projection, ProjectionDefinition};

/// Compiles a [`ProjectionDefinition`] into a PostgreSQL `SELECT` column list.
///
/// Returns `"*"` if the list is empty (meaning `SELECT *`).
pub(crate) fn compile_projections(projections: &ProjectionDefinition) -> String {
    if projections.is_empty() { return "*".to_string() }

    projections.iter()
        .map(|proj| compile_projection(proj))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Compiles a single [`Projection`] variant into its SQL fragment.
fn compile_projection(projection: &Projection) -> String {
    match projection {
        Projection::Field(smol_str) => smol_str.to_string(),
        Projection::Aliased(projection, smol_str) => format!("{} AS {}", compile_projection(projection), smol_str),
        Projection::CountAll => "COUNT(*)".to_string(),
        Projection::Count(smol_str) => format!("COUNT({})", smol_str),
        Projection::Sum(smol_str) => format!("SUM({})", smol_str),
        Projection::Avg(smol_str) => format!("AVG({})", smol_str),
        Projection::Min(smol_str) => format!("MIN({})", smol_str),
        Projection::Max(smol_str) => format!("MAX({})", smol_str),
    }
}

#[cfg(test)]
mod tests {
    use prism_db_core::query::ProjectionBuilder;
    use super::*;

    fn build(f: impl FnOnce(ProjectionBuilder) -> ProjectionBuilder) -> String {
        let def = f(ProjectionBuilder::new()).build();
        compile_projections(&def)
    }

    #[test]
    fn empty_returns_star() {
        assert_eq!(compile_projections(&Default::default()), "*");
    }

    #[test]
    fn single_field() {
        assert_eq!(build(|b| b.field("id")), "id");
    }

    #[test]
    fn count_all() {
        assert_eq!(build(|b| b.count_all()), "COUNT(*)");
    }

    #[test]
    fn aggregates() {
        assert_eq!(build(|b| b.sum("amount")), "SUM(amount)");
        assert_eq!(build(|b| b.avg("score")), "AVG(score)");
        assert_eq!(build(|b| b.min("price")), "MIN(price)");
        assert_eq!(build(|b| b.max("price")), "MAX(price)");
    }

    #[test]
    fn aliased_projection() {
        assert_eq!(build(|b| b.r#as(Projection::CountAll, "total")), "COUNT(*) AS total");
    }

    #[test]
    fn multiple_projections_joined_with_comma() {
        assert_eq!(build(|b| b.field("id").field("name").count_all()), "id, name, COUNT(*)");
    }
}
