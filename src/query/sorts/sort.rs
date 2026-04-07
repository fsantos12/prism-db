pub type SortDefinition = Vec<Sort>;

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