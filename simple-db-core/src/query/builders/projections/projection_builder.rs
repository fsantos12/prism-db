use smallvec::SmallVec;
use smol_str::SmolStr;

use crate::query::builders::{Projection, ProjectionDefinition};

/// Fluent builder for constructing a [`ProjectionDefinition`].
///
/// Each method appends one [`Projection`]. Call [`build`](ProjectionBuilder::build) to obtain
/// the final list. An empty list is interpreted by drivers as `SELECT *`.
pub struct ProjectionBuilder(ProjectionDefinition);

impl ProjectionBuilder {
    /// Creates a new, empty builder.
    pub fn new() -> Self {
        Self(SmallVec::new())
    }

    fn add(mut self, projection: Projection) -> Self {
        self.0.push(projection);
        self
    }

    /// Appends all projections from an iterator.
    pub fn extend<I>(mut self, projections: I) -> Self
    where I: IntoIterator<Item = Projection>,
    {
        self.0.extend(projections);
        self
    }

    /// Appends a bare column name: `field`.
    pub fn field<F: Into<SmolStr>>(self, field: F) -> Self {
        self.add(Projection::Field(field.into()))
    }

    /// Appends `COUNT(*)`.
    pub fn count_all(self) -> Self {
        self.add(Projection::CountAll)
    }

    /// Appends `COUNT(field)`.
    pub fn count<F: Into<SmolStr>>(self, field: F) -> Self {
        self.add(Projection::Count(field.into()))
    }

    /// Appends `SUM(field)`.
    pub fn sum<F: Into<SmolStr>>(self, field: F) -> Self {
        self.add(Projection::Sum(field.into()))
    }

    /// Appends `AVG(field)`.
    pub fn avg<F: Into<SmolStr>>(self, field: F) -> Self {
        self.add(Projection::Avg(field.into()))
    }

    /// Appends `MIN(field)`.
    pub fn min<F: Into<SmolStr>>(self, field: F) -> Self {
        self.add(Projection::Min(field.into()))
    }

    /// Appends `MAX(field)`.
    pub fn max<F: Into<SmolStr>>(self, field: F) -> Self {
        self.add(Projection::Max(field.into()))
    }

    /// Appends `proj AS alias`.
    pub fn r#as<A: Into<SmolStr>>(self, proj: Projection, alias: A) -> Self {
        self.add(proj.r#as(alias))
    }

    /// Consumes the builder and returns the collected [`ProjectionDefinition`].
    pub fn build(self) -> SmallVec<[Projection; 10]> {
        self.0
    }
}

impl Default for ProjectionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Shorthand macro for constructing a [`ProjectionDefinition`] without holding a builder variable.
///
/// # Examples
/// ```ignore
/// let p = project!(); // empty => SELECT *
/// let p = project!(field("id"), count_all());
/// ```
#[macro_export]
macro_rules! project {
    () => {
        $crate::query::ProjectionBuilder::new().build()
    };

    ( $( $method:ident ( $( $arg:expr ),* ) ),+ $(,)? ) => {
        {
            let builder = $crate::query::ProjectionBuilder::new();
            $( let builder = builder.$method( $( $arg ),* ); )+
            builder.build()
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_build_produces_empty_list() {
        assert!(ProjectionBuilder::new().build().is_empty());
    }

    #[test]
    fn default_is_empty() {
        assert!(ProjectionBuilder::default().build().is_empty());
    }

    #[test]
    fn field_appends_correct_variant() {
        let p = ProjectionBuilder::new().field("id").build();
        assert_eq!(p.len(), 1);
        assert!(matches!(&p[0], Projection::Field(f) if f.as_str() == "id"));
    }

    #[test]
    fn aggregate_variants() {
        let p = ProjectionBuilder::new()
            .count_all()
            .count("id")
            .sum("amount")
            .avg("score")
            .min("price")
            .max("price")
            .build();
        assert_eq!(p.len(), 6);
        assert!(matches!(&p[0], Projection::CountAll));
        assert!(matches!(&p[1], Projection::Count(f) if f.as_str() == "id"));
        assert!(matches!(&p[2], Projection::Sum(f) if f.as_str() == "amount"));
        assert!(matches!(&p[3], Projection::Avg(f) if f.as_str() == "score"));
        assert!(matches!(&p[4], Projection::Min(f) if f.as_str() == "price"));
        assert!(matches!(&p[5], Projection::Max(f) if f.as_str() == "price"));
    }

    #[test]
    fn as_wraps_in_aliased() {
        let p = ProjectionBuilder::new()
            .r#as(Projection::CountAll, "total")
            .build();
        assert!(matches!(&p[0], Projection::Aliased(inner, alias)
            if matches!(inner.as_ref(), Projection::CountAll) && alias.as_str() == "total"
        ));
    }

    #[test]
    fn projection_as_method_wraps_correctly() {
        let proj = Projection::Field("name".into()).r#as("n");
        assert!(matches!(proj, Projection::Aliased(inner, alias)
            if matches!(inner.as_ref(), Projection::Field(f) if f.as_str() == "name")
            && alias.as_str() == "n"
        ));
    }

    #[test]
    fn chaining_produces_multiple_projections() {
        let p = ProjectionBuilder::new().field("a").field("b").count_all().build();
        assert_eq!(p.len(), 3);
    }

    #[test]
    fn extend_appends_all_projections() {
        let extra = vec![Projection::Field("x".into()), Projection::CountAll];
        let p = ProjectionBuilder::new().extend(extra).build();
        assert_eq!(p.len(), 2);
    }

    #[test]
    fn project_macro_empty() {
        let p = crate::project!();
        assert!(p.is_empty());
    }

    #[test]
    fn project_macro_with_methods() {
        let p = crate::project!(field("id"), count_all());
        assert_eq!(p.len(), 2);
    }
}
