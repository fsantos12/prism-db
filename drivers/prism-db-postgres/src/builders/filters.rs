use prism_db_core::{query::{Filter, FilterDefinition}, types::DbValue};

/// Compiles a [`FilterDefinition`] into a PostgreSQL `WHERE` clause fragment and bound values.
///
/// Returns an empty string and no values if `filters` is empty.
/// Multiple top-level filters are combined with `AND`.
/// PostgreSQL uses numbered `$1`, `$2`, ... placeholders; `counter` tracks the next index and is
/// shared across the entire query so that parameter numbers remain globally unique.
pub(crate) fn compile_filters(filters: &FilterDefinition, counter: &mut usize) -> (String, Vec<DbValue>) {
    if filters.is_empty() { return ("".to_string(), vec![]) }

    let mut sql_parts = Vec::new();
    let mut values = Vec::new();

    for filter in filters {
        let (sql, mut params) = compile_filter(filter, counter);
        sql_parts.push(sql);
        values.append(&mut params);
    }

    let final_sql = sql_parts.join(" AND ");
    (final_sql, values)
}

/// Compiles a slice of filters joined by `operator` and wrapped in parentheses.
fn compile_logical_filters(filters: &[Filter], operator: &str, counter: &mut usize) -> (String, Vec<DbValue>) {
    if filters.is_empty() { return ("".to_string(), vec![]) }

    let mut sql_parts = Vec::new();
    let mut values = Vec::new();

    for filter in filters {
        let (sql, mut params) = compile_filter(filter, counter);
        sql_parts.push(sql);
        values.append(&mut params);
    }

    let final_sql = format!("({})", sql_parts.join(operator));
    (final_sql, values)
}

/// Returns the next `$N` placeholder and increments the counter.
fn next_placeholder(counter: &mut usize) -> String {
    let p = format!("${}", *counter);
    *counter += 1;
    p
}

/// Compiles a single [`Filter`] variant into a SQL fragment and its bound values.
fn compile_filter(filter: &Filter, counter: &mut usize) -> (String, Vec<DbValue>) {
    match filter {
        Filter::IsNull(col) => (format!("{} IS NULL", col), vec![]),
        Filter::IsNotNull(col) => (format!("{} IS NOT NULL", col), vec![]),

        Filter::Eq(col, val) => { let p = next_placeholder(counter); (format!("{} = {}", col, p), vec![val.clone()]) }
        Filter::Neq(col, val) => { let p = next_placeholder(counter); (format!("{} != {}", col, p), vec![val.clone()]) }
        Filter::Lt(col, val) => { let p = next_placeholder(counter); (format!("{} < {}", col, p), vec![val.clone()]) }
        Filter::Lte(col, val) => { let p = next_placeholder(counter); (format!("{} <= {}", col, p), vec![val.clone()]) }
        Filter::Gt(col, val) => { let p = next_placeholder(counter); (format!("{} > {}", col, p), vec![val.clone()]) }
        Filter::Gte(col, val) => { let p = next_placeholder(counter); (format!("{} >= {}", col, p), vec![val.clone()]) }

        Filter::StartsWith(col, val) => { let p = next_placeholder(counter); (format!("{} LIKE {} || '%'", col, p), vec![val.clone()]) }
        Filter::NotStartsWith(col, val) => { let p = next_placeholder(counter); (format!("{} NOT LIKE {} || '%'", col, p), vec![val.clone()]) }
        Filter::EndsWith(col, val) => { let p = next_placeholder(counter); (format!("{} LIKE '%' || {}", col, p), vec![val.clone()]) }
        Filter::NotEndsWith(col, val) => { let p = next_placeholder(counter); (format!("{} NOT LIKE '%' || {}", col, p), vec![val.clone()]) }
        Filter::Contains(col, val) => { let p = next_placeholder(counter); (format!("{} LIKE '%' || {} || '%'", col, p), vec![val.clone()]) }
        Filter::NotContains(col, val) => { let p = next_placeholder(counter); (format!("{} NOT LIKE '%' || {} || '%'", col, p), vec![val.clone()]) }

        // PostgreSQL uses `~` for POSIX regex matching
        Filter::Regex(col, pattern) => { let p = next_placeholder(counter); (format!("{} ~ {}", col, p), vec![DbValue::from(pattern.clone())]) }

        Filter::Between(col, (low, high)) => {
            let p1 = next_placeholder(counter);
            let p2 = next_placeholder(counter);
            (format!("{} BETWEEN {} AND {}", col, p1, p2), vec![low.clone(), high.clone()])
        }
        Filter::NotBetween(col, (low, high)) => {
            let p1 = next_placeholder(counter);
            let p2 = next_placeholder(counter);
            (format!("{} NOT BETWEEN {} AND {}", col, p1, p2), vec![low.clone(), high.clone()])
        }

        Filter::In(col, vals) => {
            if vals.is_empty() { return ("1=0".to_string(), vec![]); }
            let placeholders: Vec<String> = vals.iter().map(|_| next_placeholder(counter)).collect();
            (format!("{} IN ({})", col, placeholders.join(", ")), vals.clone())
        }
        Filter::NotIn(col, vals) => {
            if vals.is_empty() { return ("1=1".to_string(), vec![]); }
            let placeholders: Vec<String> = vals.iter().map(|_| next_placeholder(counter)).collect();
            (format!("{} NOT IN ({})", col, placeholders.join(", ")), vals.clone())
        }

        Filter::And(filters) => compile_logical_filters(filters, " AND ", counter),
        Filter::Or(filters) => compile_logical_filters(filters, " OR ", counter),
        Filter::Not(filter) => {
            let (sql, params) = compile_filter(filter, counter);
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
        compile_filters(&def, &mut 1)
    }

    #[test]
    fn empty_returns_empty() {
        let (sql, vals) = compile_filters(&Default::default(), &mut 1);
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
    fn eq_uses_dollar_placeholder() {
        let (sql, vals) = build(|b| b.eq("age", 30i32));
        assert_eq!(sql, "age = $1");
        assert_eq!(vals.len(), 1);
    }

    #[test]
    fn neq() {
        let (sql, _) = build(|b| b.neq("status", "deleted"));
        assert_eq!(sql, "status != $1");
    }

    #[test]
    fn between_uses_sequential_placeholders() {
        let (sql, vals) = build(|b| b.between("age", 18i32, 65i32));
        assert_eq!(sql, "age BETWEEN $1 AND $2");
        assert_eq!(vals.len(), 2);
    }

    #[test]
    fn in_uses_sequential_placeholders() {
        let (sql, vals) = build(|b| b.is_in("id", vec![1i32, 2i32, 3i32]));
        assert_eq!(sql, "id IN ($1, $2, $3)");
        assert_eq!(vals.len(), 3);
    }

    #[test]
    fn in_empty_returns_always_false() {
        let (sql, _) = build(|b| b.is_in("id", Vec::<i32>::new()));
        assert_eq!(sql, "1=0");
    }

    #[test]
    fn not_in_empty_returns_always_true() {
        let (sql, _) = build(|b| b.not_in("id", Vec::<i32>::new()));
        assert_eq!(sql, "1=1");
    }

    #[test]
    fn regex_uses_tilde_operator() {
        let (sql, _) = build(|b| b.regex("email", r"@example\.com"));
        assert_eq!(sql, "email ~ $1");
    }

    #[test]
    fn and_logical_group() {
        let (sql, vals) = build(|b| b.and([
            Filter::Eq("a".into(), 1i32.into()),
            Filter::Eq("b".into(), 2i32.into()),
        ]));
        assert_eq!(sql, "(a = $1 AND b = $2)");
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
    fn counter_is_shared_across_top_level_filters() {
        let (sql, _) = build(|b| b.eq("a", 1i32).eq("b", 2i32));
        assert_eq!(sql, "a = $1 AND b = $2");
    }

    #[test]
    fn counter_initial_value_is_respected() {
        let def = FilterBuilder::new().eq("a", 1i32).build();
        let (sql, _) = compile_filters(&def, &mut 5);
        assert_eq!(sql, "a = $5");
    }
}
