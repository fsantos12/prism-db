use smallvec::SmallVec;
use smol_str::SmolStr;

/// A single expression in a `SELECT` column list.
///
/// Simple fields and aggregate functions are the primary variants. Any projection can be
/// given a SQL alias via [`as`](Projection::as).
#[derive(Debug, Clone)]
pub enum Projection {
    /// A bare column name: `field`.
    Field(SmolStr),
    /// Any projection with a SQL alias: `<projection> AS alias`.
    Aliased(Box<Projection>, SmolStr),
    /// `COUNT(*)`.
    CountAll,
    /// `COUNT(field)`.
    Count(SmolStr),
    /// `SUM(field)`.
    Sum(SmolStr),
    /// `AVG(field)`.
    Avg(SmolStr),
    /// `MIN(field)`.
    Min(SmolStr),
    /// `MAX(field)`.
    Max(SmolStr),
}

impl Projection {
    /// Wraps this projection in an `AS alias` clause.
    pub fn r#as<S: Into<SmolStr>>(self, alias: S) -> Self {
        Projection::Aliased(Box::new(self), alias.into())
    }
}

/// A stack-allocated list of [`Projection`]s with inline capacity for 10 entries.
pub type ProjectionDefinition = SmallVec<[Projection; 10]>;
