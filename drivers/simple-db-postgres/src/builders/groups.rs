use simple_db_core::query::GroupDefinition;

pub fn compile_groups(groups: &GroupDefinition) -> String {
    if groups.is_empty() { return "".to_string() }

    let group_sql = groups.iter()
        .map(|col| col.to_string())
        .collect::<Vec<_>>()
        .join(", ");

    return group_sql;
}
