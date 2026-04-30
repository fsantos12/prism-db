use prism_db_core::{query::{Filter, FilterDefinition}, types::DbValue};

/// Compiles a [`FilterDefinition`] into a SQLite `WHERE` clause fragment and bound values.
///
/// Returns an empty string and no values if `filters` is empty.
/// Multiple top-level filters are combined with `AND`.
/// SQLite uses positional `?` placeholders.
///
/// Note: SQLite does not support `REGEXP` natively. The [`Filter::Regex`] variant is compiled
/// to `GLOB`, which uses glob-style patterns rather than POSIX regex. Register a custom
/// `regexp` function on the connection if full regex support is required.
pub(crate) fn compile_filters(filters: &FilterDefinition) -> (String, Vec<DbValue>) {
    if filters.is_empty() { return ("".to_string(), vec![]) }

    let mut sql_parts = Vec::new();
    let mut values = Vec::new();

    for filter in filters {
        let (sql, mut params) = compile_filter(filter);
        sql_parts.push(sql);
        values.append(&mut params);
    }

    let final_sql = sql_parts.join(" AND ");
    (final_sql, values)
}

/// Compiles a slice of filters joined by `operator` and wrapped in parentheses.
fn compile_logical_filters(filters: &[Filter], operator: &str) -> (String, Vec<DbValue>) {
    if filters.is_empty() { return ("".to_string(), vec![]) }

    let mut sql_parts = Vec::new();
    let mut values = Vec::new();

    for filter in filters {
        let (sql, mut params) = compile_filter(filter);
        sql_parts.push(sql);
        values.append(&mut params);
    }

    let final_sql = format!("({})", sql_parts.join(operator));
    (final_sql, values)
}

/// Compiles a single [`Filter`] variant into a SQL fragment and its bound values.
fn compile_filter(filter: &Filter) -> (String, Vec<DbValue>) {
    match filter {
        Filter::IsNull(smol_str) => (format!("{} IS NULL", smol_str), vec![]),
        Filter::IsNotNull(smol_str) => (format!("{} IS NOT NULL", smol_str), vec![]),

        Filter::Eq(smol_str, db_value) => (format!("{} = ?", smol_str), vec![db_value.clone()]),
        Filter::Neq(smol_str, db_value) => (format!("{} != ?", smol_str), vec![db_value.clone()]),
        Filter::Lt(smol_str, db_value) => (format!("{} < ?", smol_str), vec![db_value.clone()]),
        Filter::Lte(smol_str, db_value) => (format!("{} <= ?", smol_str), vec![db_value.clone()]),
        Filter::Gt(smol_str, db_value) => (format!("{} > ?", smol_str), vec![db_value.clone()]),
        Filter::Gte(smol_str, db_value) => (format!("{} >= ?", smol_str), vec![db_value.clone()]),

        Filter::StartsWith(col, val) => (format!("{} LIKE ? || '%'", col), vec![val.clone()]),
        Filter::NotStartsWith(col, val) => (format!("{} NOT LIKE ? || '%'", col), vec![val.clone()]),
        Filter::EndsWith(col, val) => (format!("{} LIKE '%' || ?", col), vec![val.clone()]),
        Filter::NotEndsWith(col, val) => (format!("{} NOT LIKE '%' || ?", col), vec![val.clone()]),
        Filter::Contains(col, val) => (format!("{} LIKE '%' || ? || '%'", col), vec![val.clone()]),
        Filter::NotContains(col, val) => (format!("{} NOT LIKE '%' || ? || '%'", col), vec![val.clone()]),

        // SQLite uses GLOB for pattern matching instead of REGEXP
        Filter::Regex(smol_str, smol_str1) => (format!("{} GLOB ?", smol_str), vec![DbValue::from(smol_str1.clone())]),

        Filter::Between(smol_str, (low, high)) => (format!("{} BETWEEN ? AND ?", smol_str), vec![low.clone(), high.clone()]),
        Filter::NotBetween(smol_str, (low, high)) => (format!("{} NOT BETWEEN ? AND ?", smol_str), vec![low.clone(), high.clone()]),

        Filter::In(smol_str, vals) => {
            if vals.is_empty() {
                return ("1=0".to_string(), vec![]);
            }
            let placeholders = vec!["?"; vals.len()].join(", ");
            let sql = format!("{} IN ({})", smol_str, placeholders);
            (sql, vals.clone())
        }
        Filter::NotIn(smol_str, vals) => {
            if vals.is_empty() {
                return ("1=1".to_string(), vec![]);
            }
            let placeholders = vec!["?"; vals.len()].join(", ");
            let sql = format!("{} NOT IN ({})", smol_str, placeholders);
            (sql, vals.clone())
        }

        Filter::And(filters) => compile_logical_filters(filters, " AND "),
        Filter::Or(filters) => compile_logical_filters(filters, " OR "),
        Filter::Not(filter) => {
            let (sql, params) = compile_filter(filter);
            (format!("NOT ({})", sql), params)
        }
    }
}

#[cfg(test)]
mod tests {
    use prism_db_core::query::FilterBuilder;
    use super::*;

    fn build(f: impl FnOnce(FilterBuilder) -> FilterBuilder) -> (String, Vec<DbValue>) {
        let def = f(FilterBuilder::new()).build();
        compile_filters(&def)
    }

    #[test]
    fn empty_returns_empty() {
        let (sql, vals) = compile_filters(&Default::default());
        assert_eq!(sql, "");
        assert!(vals.is_empty());
    }

    #[test]
    fn is_null() {
        let (sql, vals) = build(|b| b.is_null("col"));
        assert_eq!(sql, "col IS NULL");
        assert!(vals.is_empty());
    }

    #[test]
    fn eq_uses_question_mark_placeholder() {
        let (sql, vals) = build(|b| b.eq("age", 30i32));
        assert_eq!(sql, "age = ?");
        assert_eq!(vals.len(), 1);
    }

    #[test]
    fn neq() {
        let (sql, _) = build(|b| b.neq("status", "deleted"));
        assert_eq!(sql, "status != ?");
    }

    #[test]
    fn comparison_operators() {
        let (sql, _) = build(|b| b.lt("score", 10i32));
        assert_eq!(sql, "score < ?");
        let (sql, _) = build(|b| b.lte("score", 10i32));
        assert_eq!(sql, "score <= ?");
        let (sql, _) = build(|b| b.gt("score", 0i32));
        assert_eq!(sql, "score > ?");
        let (sql, _) = build(|b| b.gte("score", 0i32));
        assert_eq!(sql, "score >= ?");
    }

    #[test]
    fn starts_with() {
        let (sql, _) = build(|b| b.starts_with("name", "Al"));
        assert_eq!(sql, "name LIKE ? || '%'");
    }

    #[test]
    fn ends_with() {
        let (sql, _) = build(|b| b.ends_with("email", ".com"));
        assert_eq!(sql, "email LIKE '%' || ?");
    }

    #[test]
    fn contains() {
        let (sql, _) = build(|b| b.contains("bio", "rust"));
        assert_eq!(sql, "bio LIKE '%' || ? || '%'");
    }

    #[test]
    fn regex_uses_glob_operator() {
        let (sql, vals) = build(|b| b.regex("name", "Al*"));
        assert_eq!(sql, "name GLOB ?");
        assert_eq!(vals.len(), 1);
    }

    #[test]
    fn between() {
        let (sql, vals) = build(|b| b.between("age", 18i32, 65i32));
        assert_eq!(sql, "age BETWEEN ? AND ?");
        assert_eq!(vals.len(), 2);
    }

    #[test]
    fn is_in_with_values() {
        let (sql, vals) = build(|b| b.is_in("id", vec![1i32, 2i32, 3i32]));
        assert_eq!(sql, "id IN (?, ?, ?)");
        assert_eq!(vals.len(), 3);
    }

    #[test]
    fn is_in_empty_returns_always_false() {
        let (sql, _) = build(|b| b.is_in("id", Vec::<i32>::new()));
        assert_eq!(sql, "1=0");
    }

    #[test]
    fn not_in_empty_returns_always_true() {
        let (sql, _) = build(|b| b.not_in("id", Vec::<i32>::new()));
        assert_eq!(sql, "1=1");
    }

    #[test]
    fn and_logical_group() {
        let (sql, vals) = build(|b| b.and([
            Filter::Eq("a".into(), 1i32.into()),
            Filter::Eq("b".into(), 2i32.into()),
        ]));
        assert_eq!(sql, "(a = ? AND b = ?)");
        assert_eq!(vals.len(), 2);
    }

    #[test]
    fn or_logical_group() {
        let (sql, _) = build(|b| b.or([
            Filter::IsNull("x".into()),
            Filter::IsNull("y".into()),
        ]));
        assert_eq!(sql, "(x IS NULL OR y IS NULL)");
    }

    #[test]
    fn not_wraps_expression() {
        let (sql, _) = build(|b| b.not([Filter::IsNull("x".into())]));
        assert_eq!(sql, "NOT (x IS NULL)");
    }

    #[test]
    fn multiple_top_level_filters_joined_with_and() {
        let (sql, _) = build(|b| b.is_null("a").is_null("b"));
        assert_eq!(sql, "a IS NULL AND b IS NULL");
    }
}
