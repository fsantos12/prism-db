use smallvec::SmallVec;
use smol_str::SmolStr;

use crate::types::DbValue;

#[derive(Debug, Clone)]
pub enum Filter {
    IsNull(SmolStr),
    IsNotNull(SmolStr),
    Eq(SmolStr, DbValue),
    Neq(SmolStr, DbValue),
    Lt(SmolStr, DbValue),
    Lte(SmolStr, DbValue),
    Gt(SmolStr, DbValue),
    Gte(SmolStr, DbValue),
    StartsWith(SmolStr, DbValue),
    NotStartsWith(SmolStr, DbValue),
    EndsWith(SmolStr, DbValue),
    NotEndsWith(SmolStr, DbValue),
    Contains(SmolStr, DbValue),
    NotContains(SmolStr, DbValue),
    Regex(SmolStr, SmolStr),
    Between(SmolStr, (DbValue, DbValue)),
    NotBetween(SmolStr, (DbValue, DbValue)),
    In(SmolStr, Vec<DbValue>),
    NotIn(SmolStr, Vec<DbValue>),
    And(Vec<Filter>),
    Or(Vec<Filter>),
    Not(Box<Filter>),
}

pub type FilterDefinition = SmallVec<[Filter; 8]>;