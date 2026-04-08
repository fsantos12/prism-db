//! Sort instructions for result ordering and null value placement.
//!
//! The `Sort` enum defines result ordering with support for ascending/descending
//! directions, null placement control, and random ordering for sampling.

use smallvec::SmallVec;

/// Type alias for a list of sort specifications.
/// Stack-allocated for up to 4 sorts; larger queries spill to heap automatically.
pub type SortDefinition = SmallVec<[Sort; 4]>;

#[derive(Debug, Clone)]
pub enum Sort {
    // --- Basic ---
    Asc(Box<String>),
    Desc(Box<String>),

    // --- Null Handling ---
    AscNullsFirst(Box<String>),
    AscNullsLast(Box<String>),
    DescNullsFirst(Box<String>),
    DescNullsLast(Box<String>),

    // --- Special Cases ---
    Random,
}