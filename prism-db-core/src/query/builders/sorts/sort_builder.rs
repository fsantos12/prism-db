use smallvec::SmallVec;
use smol_str::SmolStr;

use crate::query::builders::{Sort, SortDefinition};

/// Fluent builder for constructing a [`SortDefinition`].
///
/// Each method appends one sort directive. Call [`build`](SortBuilder::build) to obtain
/// the final [`SortDefinition`].
pub struct SortBuilder(SortDefinition);

impl SortBuilder {
    /// Creates a new, empty builder.
    pub fn new() -> Self {
        Self(SmallVec::new())
    }

    fn add(mut self, sort: Sort) -> Self {
        self.0.push(sort);
        self
    }

    /// Appends all sorts from an iterator.
    pub fn extend<I>(mut self, sorts: I) -> Self
    where I: IntoIterator<Item = Sort>,
    {
        self.0.extend(sorts);
        self
    }

    /// Appends `field ASC`.
    pub fn asc<F: Into<SmolStr>>(self, field: F) -> Self {
        self.add(Sort::Asc(field.into()))
    }

    /// Appends `field DESC`.
    pub fn desc<F: Into<SmolStr>>(self, field: F) -> Self {
        self.add(Sort::Desc(field.into()))
    }

    /// Appends `field ASC NULLS FIRST`.
    pub fn asc_nulls_first<F: Into<SmolStr>>(self, field: F) -> Self {
        self.add(Sort::AscNullsFirst(field.into()))
    }

    /// Appends `field ASC NULLS LAST`.
    pub fn asc_nulls_last<F: Into<SmolStr>>(self, field: F) -> Self {
        self.add(Sort::AscNullsLast(field.into()))
    }

    /// Appends `field DESC NULLS FIRST`.
    pub fn desc_nulls_first<F: Into<SmolStr>>(self, field: F) -> Self {
        self.add(Sort::DescNullsFirst(field.into()))
    }

    /// Appends `field DESC NULLS LAST`.
    pub fn desc_nulls_last<F: Into<SmolStr>>(self, field: F) -> Self {
        self.add(Sort::DescNullsLast(field.into()))
    }

    /// Appends `RANDOM()`.
    pub fn random(self) -> Self {
        self.add(Sort::Random)
    }

    /// Consumes the builder and returns the collected [`SortDefinition`].
    pub fn build(self) -> SmallVec<[Sort; 4]> {
        self.0
    }
}

impl Default for SortBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Shorthand macro for constructing a [`SortDefinition`] without holding a builder variable.
///
/// # Examples
/// ```ignore
/// let s = sort!(); // empty
/// let s = sort!(asc("name"), desc("created_at"));
/// ```
#[macro_export]
macro_rules! sort {
    () => {
        $crate::query::SortBuilder::new().build()
    };

    ( $( $method:ident ( $( $arg:expr ),* ) ),+ $(,)? ) => {
        {
            let builder = $crate::query::SortBuilder::new();
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
        assert!(SortBuilder::new().build().is_empty());
    }

    #[test]
    fn default_is_empty() {
        assert!(SortBuilder::default().build().is_empty());
    }

    #[test]
    fn asc_appends_correct_variant() {
        let s = SortBuilder::new().asc("name").build();
        assert_eq!(s.len(), 1);
        assert!(matches!(&s[0], Sort::Asc(f) if f.as_str() == "name"));
    }

    #[test]
    fn desc_appends_correct_variant() {
        let s = SortBuilder::new().desc("created_at").build();
        assert!(matches!(&s[0], Sort::Desc(f) if f.as_str() == "created_at"));
    }

    #[test]
    fn nulls_variants() {
        let s = SortBuilder::new()
            .asc_nulls_first("a")
            .asc_nulls_last("b")
            .desc_nulls_first("c")
            .desc_nulls_last("d")
            .build();
        assert_eq!(s.len(), 4);
        assert!(matches!(&s[0], Sort::AscNullsFirst(_)));
        assert!(matches!(&s[1], Sort::AscNullsLast(_)));
        assert!(matches!(&s[2], Sort::DescNullsFirst(_)));
        assert!(matches!(&s[3], Sort::DescNullsLast(_)));
    }

    #[test]
    fn random_appends_random_variant() {
        let s = SortBuilder::new().random().build();
        assert!(matches!(&s[0], Sort::Random));
    }

    #[test]
    fn chaining_produces_multiple_sorts() {
        let s = SortBuilder::new().asc("a").desc("b").random().build();
        assert_eq!(s.len(), 3);
    }

    #[test]
    fn extend_appends_all_sorts() {
        let extra = vec![Sort::Asc("x".into()), Sort::Desc("y".into())];
        let s = SortBuilder::new().extend(extra).build();
        assert_eq!(s.len(), 2);
    }

    #[test]
    fn sort_macro_empty() {
        let s = crate::sort!();
        assert!(s.is_empty());
    }

    #[test]
    fn sort_macro_with_methods() {
        let s = crate::sort!(asc("name"), desc("age"));
        assert_eq!(s.len(), 2);
    }
}
