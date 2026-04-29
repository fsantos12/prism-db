use simple_db_core::{query::{Filter, FilterDefinition}, types::DbValue};

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

fn next_placeholder(counter: &mut usize) -> String {
    let p = format!("${}", *counter);
    *counter += 1;
    p
}

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
