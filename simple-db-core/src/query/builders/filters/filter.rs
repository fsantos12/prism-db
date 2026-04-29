use smallvec::SmallVec;
use smol_str::SmolStr;

use crate::types::DbValue;

/// A single predicate that can be applied to a database query's `WHERE` clause.
///
/// Filters are composable: [`And`](Filter::And), [`Or`](Filter::Or), and [`Not`](Filter::Not)
/// wrap a collection of inner filters to build arbitrarily complex boolean expressions.
/// All other variants compare a named column against a bound [`DbValue`].
#[derive(Debug, Clone)]
pub enum Filter {
    /// Column is `NULL`.
    IsNull(SmolStr),
    /// Column is not `NULL`.
    IsNotNull(SmolStr),
    /// Column equals value (`=`).
    Eq(SmolStr, DbValue),
    /// Column does not equal value (`!=`).
    Neq(SmolStr, DbValue),
    /// Column is less than value (`<`).
    Lt(SmolStr, DbValue),
    /// Column is less than or equal to value (`<=`).
    Lte(SmolStr, DbValue),
    /// Column is greater than value (`>`).
    Gt(SmolStr, DbValue),
    /// Column is greater than or equal to value (`>=`).
    Gte(SmolStr, DbValue),
    /// Column value starts with the given prefix (`LIKE ? || '%'`).
    StartsWith(SmolStr, DbValue),
    /// Column value does not start with the given prefix.
    NotStartsWith(SmolStr, DbValue),
    /// Column value ends with the given suffix (`LIKE '%' || ?`).
    EndsWith(SmolStr, DbValue),
    /// Column value does not end with the given suffix.
    NotEndsWith(SmolStr, DbValue),
    /// Column value contains the given substring (`LIKE '%' || ? || '%'`).
    Contains(SmolStr, DbValue),
    /// Column value does not contain the given substring.
    NotContains(SmolStr, DbValue),
    /// Column value matches the given regular expression pattern.
    Regex(SmolStr, SmolStr),
    /// Column value is between `low` and `high` (inclusive, `BETWEEN ? AND ?`).
    Between(SmolStr, (DbValue, DbValue)),
    /// Column value is not between `low` and `high`.
    NotBetween(SmolStr, (DbValue, DbValue)),
    /// Column value is one of the provided list (`IN (?,...)`).
    In(SmolStr, Vec<DbValue>),
    /// Column value is not in the provided list.
    NotIn(SmolStr, Vec<DbValue>),
    /// All inner filters must hold (`AND`).
    And(Vec<Filter>),
    /// At least one inner filter must hold (`OR`).
    Or(Vec<Filter>),
    /// The inner filter must not hold (`NOT`).
    Not(Box<Filter>),
}

/// A stack-allocated list of [`Filter`]s with inline capacity for 8 entries.
pub type FilterDefinition = SmallVec<[Filter; 8]>;
