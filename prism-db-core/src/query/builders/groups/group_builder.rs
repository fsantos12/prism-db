use smallvec::SmallVec;
use smol_str::SmolStr;

use crate::query::builders::GroupDefinition;

/// Fluent builder for constructing a [`GroupDefinition`].
///
/// Each method appends one or more column names to the `GROUP BY` list.
/// Call [`build`](GroupBuilder::build) to obtain the final definition.
pub struct GroupBuilder(GroupDefinition);

impl GroupBuilder {
    /// Creates a new, empty builder.
    pub fn new() -> Self {
        Self(SmallVec::new())
    }

    /// Appends a single column name to the `GROUP BY` list.
    pub fn field<F: Into<SmolStr>>(mut self, field: F) -> Self {
        self.0.push(field.into());
        self
    }

    /// Appends multiple column names at once.
    pub fn fields<F, I>(mut self, fields: I) -> Self
    where F: Into<SmolStr>, I: IntoIterator<Item = F> {
        self.0.extend(fields.into_iter().map(Into::into));
        self
    }

    /// Consumes the builder and returns the collected [`GroupDefinition`].
    pub fn build(self) -> SmallVec<[SmolStr; 4]> {
        self.0
    }
}

impl Default for GroupBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Shorthand macro for constructing a [`GroupDefinition`] from a comma-separated field list.
///
/// # Examples
/// ```ignore
/// let g = group!(); // empty
/// let g = group!("department", "region");
/// ```
#[macro_export]
macro_rules! group {
    () => {
        $crate::query::GroupBuilder::new().build()
    };

    ( $( $field:expr ),+ $(,)? ) => {
        $crate::query::GroupBuilder::new()
            $( .field($field) )+
            .build()
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_build_produces_empty_list() {
        assert!(GroupBuilder::new().build().is_empty());
    }

    #[test]
    fn default_is_empty() {
        assert!(GroupBuilder::default().build().is_empty());
    }

    #[test]
    fn field_appends_single_column() {
        let g = GroupBuilder::new().field("department").build();
        assert_eq!(g.len(), 1);
        assert_eq!(g[0].as_str(), "department");
    }

    #[test]
    fn fields_appends_multiple_columns() {
        let g = GroupBuilder::new().fields(["dept", "region"]).build();
        assert_eq!(g.len(), 2);
        assert_eq!(g[0].as_str(), "dept");
        assert_eq!(g[1].as_str(), "region");
    }

    #[test]
    fn chaining_field_calls() {
        let g = GroupBuilder::new().field("a").field("b").field("c").build();
        assert_eq!(g.len(), 3);
    }

    #[test]
    fn group_macro_empty() {
        let g = crate::group!();
        assert!(g.is_empty());
    }

    #[test]
    fn group_macro_with_fields() {
        let g = crate::group!("dept", "region");
        assert_eq!(g.len(), 2);
        assert_eq!(g[0].as_str(), "dept");
        assert_eq!(g[1].as_str(), "region");
    }
}
