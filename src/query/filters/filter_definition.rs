use crate::{query::filters::filter::Filter, types::DbValue};

/// A builder to construct complex, nested filter conditions.
/// It acts as a root container (implicitly an AND group) that can be built into a Filter tree.
#[derive(Debug, Clone, Default)]
pub struct FilterDefinition {
    conditions: Vec<Filter>,
}

impl FilterDefinition {
/// Creates a new, empty FilterDefinition.
    pub fn new() -> Self {
        Self::default()
    }

    /// Internal helper to push a filter and return Self for chaining.
    fn add(mut self, filter: Filter) -> Self {
        self.conditions.push(filter);
        self
    }

    // --- Logical Grouping (Closures) ---

    /// Creates an AND group. 
    /// Usage:.and(|f| f.eq("status", "active").gt("price", 100))
    pub fn and<F>(self, build: F) -> Self 
    where 
        F: FnOnce(FilterDefinition) -> FilterDefinition 
    {
        let sub_builder = build(FilterDefinition::new());
        if sub_builder.conditions.is_empty() {
            self
        } else {
            self.add(Filter::And(sub_builder.conditions))
        }
    }

    /// Creates an OR group.
    /// Usage:.or(|f| f.eq("id", 1).eq("id", 2))
    pub fn or<F>(self, build: F) -> Self 
    where 
        F: FnOnce(FilterDefinition) -> FilterDefinition 
    {
        let sub_builder = build(FilterDefinition::new());
        if sub_builder.conditions.is_empty() {
            self
        } else {
            self.add(Filter::Or(sub_builder.conditions))
        }
    }

    /// Negates a single filter condition.
    pub fn not(self, filter: Filter) -> Self {
        self.add(Filter::Not(Box::new(filter)))
    }

    // --- Null Checks ---

    pub fn is_null<F: Into<String>>(self, field: F) -> Self {
        self.add(Filter::IsNull(field.into()))
    }

    pub fn is_not_null<F: Into<String>>(self, field: F) -> Self {
        self.add(Filter::IsNotNull(field.into()))
    }

    // --- Basic Comparisons ---

    pub fn eq<F: Into<String>, V: Into<DbValue>>(self, field: F, value: V) -> Self {
        self.add(Filter::Eq(field.into(), value.into()))
    }

    pub fn neq<F: Into<String>, V: Into<DbValue>>(self, field: F, value: V) -> Self {
        self.add(Filter::Neq(field.into(), value.into()))
    }

    pub fn lt<F: Into<String>, V: Into<DbValue>>(self, field: F, value: V) -> Self {
        self.add(Filter::Lt(field.into(), value.into()))
    }

    pub fn lte<F: Into<String>, V: Into<DbValue>>(self, field: F, value: V) -> Self {
        self.add(Filter::Lte(field.into(), value.into()))
    }

    pub fn gt<F: Into<String>, V: Into<DbValue>>(self, field: F, value: V) -> Self {
        self.add(Filter::Gt(field.into(), value.into()))
    }

    pub fn gte<F: Into<String>, V: Into<DbValue>>(self, field: F, value: V) -> Self {
        self.add(Filter::Gte(field.into(), value.into()))
    }

    // --- Pattern Matching ---

    pub fn starts_with<F: Into<String>, V: Into<DbValue>>(self, field: F, value: V) -> Self {
        self.add(Filter::StartsWith(field.into(), value.into()))
    }

    pub fn contains<F: Into<String>, V: Into<DbValue>>(self, field: F, value: V) -> Self {
        self.add(Filter::Contains(field.into(), value.into()))
    }

    pub fn ends_with<F: Into<String>, V: Into<DbValue>>(self, field: F, value: V) -> Self {
        self.add(Filter::EndsWith(field.into(), value.into()))
    }

    pub fn regex<F: Into<String>, R: Into<String>>(self, field: F, regex: R) -> Self {
        self.add(Filter::Regex(field.into(), regex.into()))
    }

    // --- Range Checks ---

    pub fn between<F: Into<String>, V: Into<DbValue>>(self, field: F, low: V, high: V) -> Self {
        self.add(Filter::Between(field.into(), low.into(), high.into()))
    }

    // --- Set Membership ---

    pub fn is_in<F: Into<String>, V: Into<DbValue>>(self, field: F, values: Vec<V>) -> Self {
        self.add(Filter::In(
            field.into(),
            values.into_iter().map(Into::into).collect(),
        ))
    }

    pub fn not_in<F: Into<String>, V: Into<DbValue>>(self, field: F, values: Vec<V>) -> Self {
        self.add(Filter::NotIn(
            field.into(),
            values.into_iter().map(Into::into).collect(),
        ))
    }

    // --- Finalization ---

    /// Consumes the builder and returns a single root Filter.
    /// If there are multiple conditions, they are wrapped in an And.
    pub fn build(self) -> Option<Filter> {
        match self.conditions.len() {
            0 => None,
            1 => self.conditions.into_iter().next(),
            _ => Some(Filter::And(self.conditions)),
        }
    }
}

impl IntoIterator for FilterDefinition {
    type Item = Filter;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.conditions.into_iter()
    }
}

impl<'a> IntoIterator for &'a FilterDefinition {
    type Item = &'a Filter;
    type IntoIter = std::slice::Iter<'a, Filter>;
    fn into_iter(self) -> Self::IntoIter {
        self.conditions.iter()
    }
}