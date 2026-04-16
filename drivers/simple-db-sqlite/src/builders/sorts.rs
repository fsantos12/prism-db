use simple_db_core::query::{Sort, SortDefinition};

pub fn compile_sorts(sorts: &SortDefinition) -> String {
    if sorts.is_empty() { return "".to_string() }

    let sort_sql = sorts.into_iter()
        .map(compile_sort)
        .collect::<Vec<_>>()
        .join(", ");

    return sort_sql;
}

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