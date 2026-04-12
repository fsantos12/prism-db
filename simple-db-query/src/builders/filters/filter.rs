use smol_str::SmolStr;

use crate::types::DbValue;

#[derive(Debug, Clone)]
pub enum Filter {
    // --- Null Checks ---
    IsNull(SmolStr),
    IsNotNull(SmolStr),

    // --- Basic Comparisons ---
    Eq(SmolStr, DbValue),
    Neq(SmolStr, DbValue),
    Lt(SmolStr, DbValue),
    Lte(SmolStr, DbValue),
    Gt(SmolStr, DbValue),
    Gte(SmolStr, DbValue),

    // --- Pattern Matching ---
    StartsWith(SmolStr, DbValue),
    NotStartsWith(SmolStr, DbValue),
    EndsWith(SmolStr, DbValue),
    NotEndsWith(SmolStr, DbValue),
    Contains(SmolStr, DbValue),
    NotContains(SmolStr, DbValue),

    // --- Regex Matching ---
    Regex(SmolStr, SmolStr),

    // --- Range Checks ---
    Between(SmolStr, (DbValue, DbValue)),
    NotBetween(SmolStr, (DbValue, DbValue)),

    // --- Set Membership ---
    In(SmolStr, Vec<DbValue>),
    NotIn(SmolStr, Vec<DbValue>),

    // --- Logical Operators ---
    And(Vec<Filter>),
    Or(Vec<Filter>),
    Not(Box<Filter>),
}