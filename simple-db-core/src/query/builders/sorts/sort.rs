use smallvec::SmallVec;
use smol_str::SmolStr;

/// Ordering direction for a single column in an `ORDER BY` clause.
#[derive(Debug, Clone)]
pub enum Sort {
    /// `field ASC`
    Asc(SmolStr),
    /// `field DESC`
    Desc(SmolStr),
    /// `field ASC NULLS FIRST`
    AscNullsFirst(SmolStr),
    /// `field ASC NULLS LAST`
    AscNullsLast(SmolStr),
    /// `field DESC NULLS FIRST`
    DescNullsFirst(SmolStr),
    /// `field DESC NULLS LAST`
    DescNullsLast(SmolStr),
    /// `RANDOM()` — randomises the result order.
    Random,
}

/// A stack-allocated list of [`Sort`]s with inline capacity for 4 entries.
pub type SortDefinition = SmallVec<[Sort; 4]>;
