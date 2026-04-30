use smallvec::SmallVec;
use smol_str::SmolStr;

use crate::{query::builders::filters::{Filter, FilterDefinition}, types::DbValue};

/// Fluent builder for constructing a [`FilterDefinition`].
///
/// Each method appends one filter to the internal list. Call [`build`](FilterBuilder::build)
/// to obtain the final [`FilterDefinition`].
pub struct FilterBuilder(FilterDefinition);

impl FilterBuilder {
    /// Creates a new, empty builder.
    pub fn new() -> Self {
        Self(SmallVec::new())
    }

    /// Appends a pre-constructed [`Filter`].
    pub fn add(mut self, filter: Filter) -> Self {
        self.0.push(filter);
        self
    }

    /// Appends all filters from an iterator.
    pub fn extend<I>(mut self, filters: I) -> Self
    where I: IntoIterator<Item = Filter> {
        self.0.extend(filters);
        self
    }

    /// Appends `field IS NULL`.
    pub fn is_null<F: Into<SmolStr>>(self, field: F) -> Self {
        self.add(Filter::IsNull(field.into()))
    }

    /// Appends `field IS NOT NULL`.
    pub fn is_not_null<F: Into<SmolStr>>(self, field: F) -> Self {
        self.add(Filter::IsNotNull(field.into()))
    }

    /// Appends `field = value`.
    pub fn eq<F: Into<SmolStr>, V: Into<DbValue>>(self, field: F, value: V) -> Self {
        self.add(Filter::Eq(field.into(), value.into()))
    }

    /// Appends `field != value`.
    pub fn neq<F: Into<SmolStr>, V: Into<DbValue>>(self, field: F, value: V) -> Self {
        self.add(Filter::Neq(field.into(), value.into()))
    }

    /// Appends `field < value`.
    pub fn lt<F: Into<SmolStr>, V: Into<DbValue>>(self, field: F, value: V) -> Self {
        self.add(Filter::Lt(field.into(), value.into()))
    }

    /// Appends `field <= value`.
    pub fn lte<F: Into<SmolStr>, V: Into<DbValue>>(self, field: F, value: V) -> Self {
        self.add(Filter::Lte(field.into(), value.into()))
    }

    /// Appends `field > value`.
    pub fn gt<F: Into<SmolStr>, V: Into<DbValue>>(self, field: F, value: V) -> Self {
        self.add(Filter::Gt(field.into(), value.into()))
    }

    /// Appends `field >= value`.
    pub fn gte<F: Into<SmolStr>, V: Into<DbValue>>(self, field: F, value: V) -> Self {
        self.add(Filter::Gte(field.into(), value.into()))
    }

    /// Appends `field LIKE value || '%'` (starts-with).
    pub fn starts_with<F: Into<SmolStr>, V: Into<DbValue>>(self, field: F, value: V) -> Self {
        self.add(Filter::StartsWith(field.into(), value.into()))
    }

    /// Appends `field NOT LIKE value || '%'` (does not start with).
    pub fn not_starts_with<F: Into<SmolStr>, V: Into<DbValue>>(self, field: F, value: V) -> Self {
        self.add(Filter::NotStartsWith(field.into(), value.into()))
    }

    /// Appends `field LIKE '%' || value || '%'` (contains).
    pub fn contains<F: Into<SmolStr>, V: Into<DbValue>>(self, field: F, value: V) -> Self {
        self.add(Filter::Contains(field.into(), value.into()))
    }

    /// Appends `field NOT LIKE '%' || value || '%'` (does not contain).
    pub fn not_contains<F: Into<SmolStr>, V: Into<DbValue>>(self, field: F, value: V) -> Self {
        self.add(Filter::NotContains(field.into(), value.into()))
    }

    /// Appends `field LIKE '%' || value` (ends-with).
    pub fn ends_with<F: Into<SmolStr>, V: Into<DbValue>>(self, field: F, value: V) -> Self {
        self.add(Filter::EndsWith(field.into(), value.into()))
    }

    /// Appends `field NOT LIKE '%' || value` (does not end with).
    pub fn not_ends_with<F: Into<SmolStr>, V: Into<DbValue>>(self, field: F, value: V) -> Self {
        self.add(Filter::NotEndsWith(field.into(), value.into()))
    }

    /// Appends a regex match filter.
    pub fn regex<F: Into<SmolStr>, R: Into<SmolStr>>(self, field: F, regex: R) -> Self {
        self.add(Filter::Regex(field.into(), regex.into()))
    }

    /// Appends `field BETWEEN low AND high`.
    pub fn between<F: Into<SmolStr>, V: Into<DbValue>>(self, field: F, low: V, high: V) -> Self {
        self.add(Filter::Between(field.into(), (low.into(), high.into())))
    }

    /// Appends `field NOT BETWEEN low AND high`.
    pub fn not_between<F: Into<SmolStr>, V: Into<DbValue>>(self, field: F, low: V, high: V) -> Self {
        self.add(Filter::NotBetween(field.into(), (low.into(), high.into())))
    }

    /// Appends `field IN (values)`. An empty `values` list is a no-op.
    pub fn is_in<F: Into<SmolStr>, V: Into<DbValue>>(self, field: F, values: Vec<V>) -> Self {
        let db_values: Vec<DbValue> = values.into_iter().map(Into::into).collect();
        self.add(Filter::In(field.into(), db_values))
    }

    /// Appends `field NOT IN (values)`. An empty `values` list is a no-op.
    pub fn not_in<F: Into<SmolStr>, V: Into<DbValue>>(self, field: F, values: Vec<V>) -> Self {
        let db_values: Vec<DbValue> = values.into_iter().map(Into::into).collect();
        self.add(Filter::NotIn(field.into(), db_values))
    }

    /// Appends an `AND` group. If `filters` is empty the call is a no-op.
    pub fn and<I>(self, filters: I) -> Self
    where I: IntoIterator<Item = Filter> {
        let vec: Vec<Filter> = filters.into_iter().collect();
        if vec.is_empty() {
            self
        } else {
            self.add(Filter::And(vec))
        }
    }

    /// Appends an `OR` group. If `filters` is empty the call is a no-op.
    pub fn or<I>(self, filters: I) -> Self
    where I: IntoIterator<Item = Filter> {
        let vec: Vec<Filter> = filters.into_iter().collect();
        if vec.is_empty() {
            self
        } else {
            self.add(Filter::Or(vec))
        }
    }

    /// Appends a `NOT` wrapper.
    ///
    /// - Zero filters: no-op.
    /// - One filter: wraps it directly in `NOT`.
    /// - Multiple filters: wraps them in `NOT(AND(...))`.
    pub fn not<I>(self, filters: I) -> Self
    where I: IntoIterator<Item = Filter> {
        let mut vec: Vec<Filter> = filters.into_iter().collect();
        match vec.len() {
            0 => self,
            1 => self.add(Filter::Not(Box::new(vec.pop().unwrap()))),
            _ => self.add(Filter::Not(Box::new(Filter::And(vec)))),
        }
    }

    /// Consumes the builder and returns the collected [`FilterDefinition`].
    pub fn build(self) -> SmallVec<[Filter; 8]> {
        self.0
    }
}

impl Default for FilterBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Shorthand macro for constructing a [`FilterDefinition`] without holding a builder variable.
///
/// # Examples
/// ```ignore
/// let f = filter!(); // empty
/// let f = filter!(eq("age", 30i32), gt("score", 0i32));
/// ```
#[macro_export]
macro_rules! filter {
    () => {
        $crate::query::FilterBuilder::new().build()
    };

    ( $( $method:ident ( $( $arg:expr ),* ) ),+ $(,)? ) => {
        {
            let builder = $crate::query::FilterBuilder::new();
            $( let builder = builder.$method( $( $arg ),* ); )+
            builder.build()
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_build_produces_empty_list() {
        assert!(FilterBuilder::new().build().is_empty());
    }

    #[test]
    fn default_is_empty() {
        assert!(FilterBuilder::default().build().is_empty());
    }

    #[test]
    fn is_null_appends_correct_variant() {
        let f = FilterBuilder::new().is_null("col").build();
        assert_eq!(f.len(), 1);
        assert!(matches!(&f[0], Filter::IsNull(c) if c.as_str() == "col"));
    }

    #[test]
    fn is_not_null_appends_correct_variant() {
        let f = FilterBuilder::new().is_not_null("col").build();
        assert!(matches!(&f[0], Filter::IsNotNull(c) if c.as_str() == "col"));
    }

    #[test]
    fn eq_appends_correct_variant() {
        let f = FilterBuilder::new().eq("name", "Alice").build();
        match &f[0] {
            Filter::Eq(col, val) => {
                assert_eq!(col.as_str(), "name");
                assert_eq!(val.get::<String>(), Some("Alice".to_string()));
            }
            _ => panic!("expected Eq"),
        }
    }

    #[test]
    fn comparison_variants() {
        let f = FilterBuilder::new()
            .neq("a", 1i32)
            .lt("b", 2i32)
            .lte("c", 3i32)
            .gt("d", 4i32)
            .gte("e", 5i32)
            .build();
        assert_eq!(f.len(), 5);
        assert!(matches!(&f[0], Filter::Neq(..)));
        assert!(matches!(&f[1], Filter::Lt(..)));
        assert!(matches!(&f[2], Filter::Lte(..)));
        assert!(matches!(&f[3], Filter::Gt(..)));
        assert!(matches!(&f[4], Filter::Gte(..)));
    }

    #[test]
    fn string_match_variants() {
        let f = FilterBuilder::new()
            .starts_with("col", "pre")
            .not_starts_with("col", "pre")
            .ends_with("col", "suf")
            .not_ends_with("col", "suf")
            .contains("col", "mid")
            .not_contains("col", "mid")
            .build();
        assert_eq!(f.len(), 6);
        assert!(matches!(&f[0], Filter::StartsWith(..)));
        assert!(matches!(&f[1], Filter::NotStartsWith(..)));
        assert!(matches!(&f[2], Filter::EndsWith(..)));
        assert!(matches!(&f[3], Filter::NotEndsWith(..)));
        assert!(matches!(&f[4], Filter::Contains(..)));
        assert!(matches!(&f[5], Filter::NotContains(..)));
    }

    #[test]
    fn between_variants() {
        let f = FilterBuilder::new()
            .between("age", 18i32, 65i32)
            .not_between("score", 0i32, 100i32)
            .build();
        assert!(matches!(&f[0], Filter::Between(..)));
        assert!(matches!(&f[1], Filter::NotBetween(..)));
    }

    #[test]
    fn in_variants() {
        let f = FilterBuilder::new()
            .is_in("status", vec![1i32, 2i32, 3i32])
            .not_in("id", vec![10i32, 20i32])
            .build();
        assert!(matches!(&f[0], Filter::In(_, vals) if vals.len() == 3));
        assert!(matches!(&f[1], Filter::NotIn(_, vals) if vals.len() == 2));
    }

    #[test]
    fn regex_appends_correct_variant() {
        let f = FilterBuilder::new().regex("email", r"@example\.com$").build();
        assert!(matches!(&f[0], Filter::Regex(col, pat) if col.as_str() == "email" && pat.as_str() == r"@example\.com$"));
    }

    #[test]
    fn and_empty_is_noop() {
        let f = FilterBuilder::new().and([]).build();
        assert!(f.is_empty());
    }

    #[test]
    fn and_with_filters_wraps_them() {
        let f = FilterBuilder::new()
            .and([Filter::IsNull("x".into()), Filter::IsNull("y".into())])
            .build();
        assert_eq!(f.len(), 1);
        assert!(matches!(&f[0], Filter::And(inner) if inner.len() == 2));
    }

    #[test]
    fn or_empty_is_noop() {
        let f = FilterBuilder::new().or([]).build();
        assert!(f.is_empty());
    }

    #[test]
    fn or_with_filters_wraps_them() {
        let f = FilterBuilder::new()
            .or([Filter::IsNull("x".into()), Filter::IsNull("y".into())])
            .build();
        assert!(matches!(&f[0], Filter::Or(inner) if inner.len() == 2));
    }

    #[test]
    fn not_empty_is_noop() {
        let f = FilterBuilder::new().not([]).build();
        assert!(f.is_empty());
    }

    #[test]
    fn not_single_filter_wraps_directly() {
        let f = FilterBuilder::new().not([Filter::IsNull("x".into())]).build();
        assert!(matches!(&f[0], Filter::Not(inner) if matches!(inner.as_ref(), Filter::IsNull(_))));
    }

    #[test]
    fn not_multiple_filters_wraps_in_and() {
        let f = FilterBuilder::new()
            .not([Filter::IsNull("x".into()), Filter::IsNull("y".into())])
            .build();
        assert!(matches!(&f[0], Filter::Not(inner) if matches!(inner.as_ref(), Filter::And(v) if v.len() == 2)));
    }

    #[test]
    fn chaining_produces_multiple_filters() {
        let f = FilterBuilder::new()
            .eq("a", 1i32)
            .gt("b", 2i32)
            .is_null("c")
            .build();
        assert_eq!(f.len(), 3);
    }

    #[test]
    fn extend_appends_all_filters() {
        let extra = vec![Filter::IsNull("x".into()), Filter::IsNull("y".into())];
        let f = FilterBuilder::new().extend(extra).build();
        assert_eq!(f.len(), 2);
    }

    #[test]
    fn filter_macro_empty() {
        let f = crate::filter!();
        assert!(f.is_empty());
    }

    #[test]
    fn filter_macro_with_single_method() {
        let f = crate::filter!(eq("col", 1i32));
        assert_eq!(f.len(), 1);
        assert!(matches!(&f[0], Filter::Eq(..)));
    }

    #[test]
    fn filter_macro_with_multiple_methods() {
        let f = crate::filter!(eq("a", 1i32), gt("b", 2i32));
        assert_eq!(f.len(), 2);
    }
}
