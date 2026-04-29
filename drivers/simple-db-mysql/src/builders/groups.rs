use simple_db_core::query::GroupDefinition;

/// Compiles a [`GroupDefinition`] into a MySQL `GROUP BY` clause fragment (without the keyword).
///
/// Returns an empty string if `groups` is empty.
pub(crate) fn compile_groups(groups: &GroupDefinition) -> String {
    if groups.is_empty() { return "".to_string() }

    let group_sql = groups.iter()
        .map(|col| col.to_string())
        .collect::<Vec<_>>()
        .join(", ");

    return group_sql;
}

#[cfg(test)]
mod tests {
    use simple_db_core::query::GroupBuilder;
    use super::*;

    fn build(f: impl FnOnce(GroupBuilder) -> GroupBuilder) -> String {
        let def = f(GroupBuilder::new()).build();
        compile_groups(&def)
    }

    #[test]
    fn empty_returns_empty() {
        assert_eq!(compile_groups(&Default::default()), "");
    }

    #[test]
    fn single_group() {
        assert_eq!(build(|b| b.field("department")), "department");
    }

    #[test]
    fn multiple_groups_joined_with_comma() {
        assert_eq!(build(|b| b.field("dept").field("region")), "dept, region");
    }
}
