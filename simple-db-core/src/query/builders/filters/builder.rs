use smallvec::SmallVec;
use smol_str::SmolStr;

use crate::{query::builders::filters::{Filter, FilterDefinition}, types::DbValue};

pub struct FilterBuilder(FilterDefinition);

impl FilterBuilder {
    pub fn new() -> Self {
        Self(SmallVec::new())
    }

    pub fn add(mut self, filter: Filter) -> Self {
        self.0.push(filter);
        self
    }

    pub fn extend<I>(mut self, filters: I) -> Self
    where I: IntoIterator<Item = Filter> {
        self.0.extend(filters);
        self
    }

    pub fn is_null<F: Into<SmolStr>>(self, field: F) -> Self {
        self.add(Filter::IsNull(field.into()))
    }

    pub fn is_not_null<F: Into<SmolStr>>(self, field: F) -> Self {
        self.add(Filter::IsNotNull(field.into()))
    }

    pub fn eq<F: Into<SmolStr>, V: Into<DbValue>>(self, field: F, value: V) -> Self {
        self.add(Filter::Eq(field.into(), value.into()))
    }

    pub fn neq<F: Into<SmolStr>, V: Into<DbValue>>(self, field: F, value: V) -> Self {
        self.add(Filter::Neq(field.into(), value.into()))
    }

    pub fn lt<F: Into<SmolStr>, V: Into<DbValue>>(self, field: F, value: V) -> Self {
        self.add(Filter::Lt(field.into(), value.into()))
    }

    pub fn lte<F: Into<SmolStr>, V: Into<DbValue>>(self, field: F, value: V) -> Self {
        self.add(Filter::Lte(field.into(), value.into()))
    }

    pub fn gt<F: Into<SmolStr>, V: Into<DbValue>>(self, field: F, value: V) -> Self {
        self.add(Filter::Gt(field.into(), value.into()))
    }

    pub fn gte<F: Into<SmolStr>, V: Into<DbValue>>(self, field: F, value: V) -> Self {
        self.add(Filter::Gte(field.into(), value.into()))
    }

    pub fn starts_with<F: Into<SmolStr>, V: Into<DbValue>>(self, field: F, value: V) -> Self {
        self.add(Filter::StartsWith(field.into(), value.into()))
    }

    pub fn not_starts_with<F: Into<SmolStr>, V: Into<DbValue>>(self, field: F, value: V) -> Self {
        self.add(Filter::NotStartsWith(field.into(), value.into()))
    }

    pub fn contains<F: Into<SmolStr>, V: Into<DbValue>>(self, field: F, value: V) -> Self {
        self.add(Filter::Contains(field.into(), value.into()))
    }

    pub fn not_contains<F: Into<SmolStr>, V: Into<DbValue>>(self, field: F, value: V) -> Self {
        self.add(Filter::NotContains(field.into(), value.into()))
    }

    pub fn ends_with<F: Into<SmolStr>, V: Into<DbValue>>(self, field: F, value: V) -> Self {
        self.add(Filter::EndsWith(field.into(), value.into()))
    }

    pub fn not_ends_with<F: Into<SmolStr>, V: Into<DbValue>>(self, field: F, value: V) -> Self {
        self.add(Filter::NotEndsWith(field.into(), value.into()))
    }

    pub fn regex<F: Into<SmolStr>, R: Into<SmolStr>>(self, field: F, regex: R) -> Self {
        self.add(Filter::Regex(field.into(), regex.into()))
    }

    pub fn between<F: Into<SmolStr>, V: Into<DbValue>>(self, field: F, low: V, high: V) -> Self {
        self.add(Filter::Between(field.into(), (low.into(), high.into())))
    }

    pub fn not_between<F: Into<SmolStr>, V: Into<DbValue>>(self, field: F, low: V, high: V) -> Self {
        self.add(Filter::NotBetween(field.into(), (low.into(), high.into())))
    }

    pub fn is_in<F: Into<SmolStr>, V: Into<DbValue>>(self, field: F, values: Vec<V>) -> Self {
        let db_values: Vec<DbValue> = values.into_iter().map(Into::into).collect();
        self.add(Filter::In(field.into(), db_values))
    }

    pub fn not_in<F: Into<SmolStr>, V: Into<DbValue>>(self, field: F, values: Vec<V>) -> Self {
        let db_values: Vec<DbValue> = values.into_iter().map(Into::into).collect();
        self.add(Filter::NotIn(field.into(), db_values))
    }

    pub fn and<I>(self, filters: I) -> Self
    where I: IntoIterator<Item = Filter> {
        let vec: Vec<Filter> = filters.into_iter().collect();
        if vec.is_empty() {
            self
        } else {
            self.add(Filter::And(vec))
        }
    }

    pub fn or<I>(self, filters: I) -> Self
    where I: IntoIterator<Item = Filter> {
        let vec: Vec<Filter> = filters.into_iter().collect();
        if vec.is_empty() {
            self
        } else {
            self.add(Filter::Or(vec))
        }
    }

    pub fn not<I>(self, filters: I) -> Self
    where I: IntoIterator<Item = Filter> {
        let mut vec: Vec<Filter> = filters.into_iter().collect();
        match vec.len() {
            0 => self,
            1 => self.add(Filter::Not(Box::new(vec.pop().unwrap()))),
            _ => self.add(Filter::Not(Box::new(Filter::And(vec)))),
        }
    }

    pub fn build(self) -> SmallVec<[Filter; 8]> {
        self.0
    }
}

impl Default for FilterBuilder {
    fn default() -> Self {
        Self::new()
    }
}

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