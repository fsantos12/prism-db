use smallvec::SmallVec;
use smol_str::SmolStr;

use crate::query::builders::{Sort, SortDefinition};

pub struct SortBuilder(SortDefinition);

impl SortBuilder {
    pub fn new() -> Self {
        Self(SmallVec::new())
    }

    fn add(mut self, sort: Sort) -> Self {
        self.0.push(sort);
        self
    }

    pub fn extend<I>(mut self, sorts: I) -> Self
    where I: IntoIterator<Item = Sort>,
    {
        self.0.extend(sorts);
        self
    }

    pub fn asc<F: Into<SmolStr>>(self, field: F) -> Self {
        self.add(Sort::Asc(field.into()))
    }

    pub fn desc<F: Into<SmolStr>>(self, field: F) -> Self {
        self.add(Sort::Desc(field.into()))
    }

    pub fn asc_nulls_first<F: Into<SmolStr>>(self, field: F) -> Self {
        self.add(Sort::AscNullsFirst(field.into()))
    }

    pub fn asc_nulls_last<F: Into<SmolStr>>(self, field: F) -> Self {
        self.add(Sort::AscNullsLast(field.into()))
    }

    pub fn desc_nulls_first<F: Into<SmolStr>>(self, field: F) -> Self {
        self.add(Sort::DescNullsFirst(field.into()))
    }

    pub fn desc_nulls_last<F: Into<SmolStr>>(self, field: F) -> Self {
        self.add(Sort::DescNullsLast(field.into()))
    }

    pub fn random(self) -> Self {
        self.add(Sort::Random)
    }

    pub fn build(self) -> SmallVec<[Sort; 4]> {
        self.0
    }
}

impl Default for SortBuilder {
    fn default() -> Self {
        Self::new()
    }
}

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