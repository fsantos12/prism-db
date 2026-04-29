use smallvec::SmallVec;
use smol_str::SmolStr;

#[derive(Debug, Clone)]
pub enum Sort {
    Asc(SmolStr),
    Desc(SmolStr),
    AscNullsFirst(SmolStr),
    AscNullsLast(SmolStr),
    DescNullsFirst(SmolStr),
    DescNullsLast(SmolStr),
    Random,
}

pub type SortDefinition = SmallVec<[Sort; 4]>;