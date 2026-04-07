use crate::types::DbValue;

pub type FilterDefinition = Vec<Filter>;

#[derive(Debug, Clone)]
pub enum Filter {
    // --- Null Checks ---
    IsNull(Box<String>),
    IsNotNull(Box<String>),

    // --- Basic Comparisons ---
    Eq(Box<String>, DbValue),
    Neq(Box<String>, DbValue),
    Lt(Box<String>, DbValue),
    Lte(Box<String>, DbValue),
    Gt(Box<String>, DbValue),
    Gte(Box<String>, DbValue),

    // --- Pattern Matching ---
    StartsWith(Box<String>, DbValue),
    NotStartsWith(Box<String>, DbValue),
    EndsWith(Box<String>, DbValue),
    NotEndsWith(Box<String>, DbValue),
    Contains(Box<String>, DbValue),
    NotContains(Box<String>, DbValue),

    // --- Regex Matching ---
    Regex(Box<String>, Box<String>),

    // --- Range Checks ---
    Between(Box<String>, Box<(DbValue, DbValue)>),
    NotBetween(Box<String>, Box<(DbValue, DbValue)>),

    // --- Set Membership ---
    In(Box<String>, Box<Vec<DbValue>>),
    NotIn(Box<String>, Box<Vec<DbValue>>),

    // --- Logical Operators ---
    And(Box<FilterDefinition>),
    Or(Box<FilterDefinition>),
    Not(Box<Filter>),
}