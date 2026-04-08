//! Field selection and aggregation definitions for result shaping.
//!
//! The `Projection` enum supports selecting individual fields, applying aggregate
//! functions (Count, Sum, Avg, Min, Max), and aliasing results for custom output.

use smallvec::SmallVec;

/// Type alias for a list of projections (SELECT clauses).
/// Stack-allocated for up to 10 projections; larger queries spill to heap automatically.
pub type ProjectionDefinition = SmallVec<[Projection; 10]>;

#[derive(Debug, Clone)]
pub enum Projection {
    /// A single column: 'price'
    Field(Box<String>),

    /// A recursive alias: 'inner_projection AS alias_name'
    /// Allows complex structures like 'SUM(price) AS total'. 
    Aliased(Box<Projection>, Box<String>),

    // --- Aggregations ---
    CountAll,
    Count(Box<String>),
    Sum(Box<String>),
    Avg(Box<String>),
    Min(Box<String>),
    Max(Box<String>),
}

impl Projection {
    /// Helper to wrap any projection with an alias.
    pub fn r#as<S: Into<String>>(self, alias: S) -> Self {
        Projection::Aliased(Box::new(self), Box::new(alias.into()))
    }
}