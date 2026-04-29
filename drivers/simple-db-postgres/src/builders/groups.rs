use simple_db_core::query::GroupDefinition;

pub(crate) fn compile_groups(groups: &GroupDefinition) -> String {
    if groups.is_empty() { return "".to_string() }

    groups.iter()
        .map(|col| col.to_string())
        .collect::<Vec<_>>()
        .join(", ")
}
