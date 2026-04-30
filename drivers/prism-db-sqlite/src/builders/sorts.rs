use prism_db_core::query::{Sort, SortDefinition};

/// Compiles a [`SortDefinition`] into a SQLite `ORDER BY` clause fragment (without the keyword).
///
/// Returns an empty string if `sorts` is empty.
pub(crate) fn compile_sorts(sorts: &SortDefinition) -> String {
    if sorts.is_empty() { return "".to_string() }

    let sort_sql = sorts.into_iter()
        .map(compile_sort)
        .collect::<Vec<_>>()
        .join(", ");

    return sort_sql;
}

/// Compiles a single [`Sort`] variant into its SQL fragment.
fn compile_sort(sort: &Sort) -> String {
    match sort {
        Sort::Asc(smol_str) => format!("{} ASC", smol_str),
        Sort::Desc(smol_str) => format!("{} DESC", smol_str),
        Sort::AscNullsFirst(smol_str) => format!("{} ASC NULLS FIRST", smol_str),
        Sort::AscNullsLast(smol_str) => format!("{} ASC NULLS LAST", smol_str),
        Sort::DescNullsFirst(smol_str) => format!("{} DESC NULLS FIRST", smol_str),
        Sort::DescNullsLast(smol_str) => format!("{} DESC NULLS LAST", smol_str),
        Sort::Random => "RANDOM()".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use prism_db_core::query::SortBuilder;
    use super::*;

    fn build(f: impl FnOnce(SortBuilder) -> SortBuilder) -> String {
        let def = f(SortBuilder::new()).build();
        compile_sorts(&def)
    }

    #[test]
    fn empty_returns_empty() {
        assert_eq!(compile_sorts(&Default::default()), "");
    }

    #[test]
    fn asc() {
        assert_eq!(build(|b| b.asc("name")), "name ASC");
    }

    #[test]
    fn desc() {
        assert_eq!(build(|b| b.desc("created_at")), "created_at DESC");
    }

    #[test]
    fn nulls_variants() {
        assert_eq!(build(|b| b.asc_nulls_first("col")), "col ASC NULLS FIRST");
        assert_eq!(build(|b| b.asc_nulls_last("col")), "col ASC NULLS LAST");
        assert_eq!(build(|b| b.desc_nulls_first("col")), "col DESC NULLS FIRST");
        assert_eq!(build(|b| b.desc_nulls_last("col")), "col DESC NULLS LAST");
    }

    #[test]
    fn random() {
        assert_eq!(build(|b| b.random()), "RANDOM()");
    }

    #[test]
    fn multiple_sorts_joined_with_comma() {
        assert_eq!(build(|b| b.asc("a").desc("b")), "a ASC, b DESC");
    }
}
