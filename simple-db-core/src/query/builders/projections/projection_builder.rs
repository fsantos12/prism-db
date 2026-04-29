use smallvec::SmallVec;
use smol_str::SmolStr;

use crate::query::builders::{Projection, ProjectionDefinition};

pub struct ProjectionBuilder(ProjectionDefinition);

impl ProjectionBuilder {
    pub fn new() -> Self {
        Self(SmallVec::new())
    }

    fn add(mut self, projection: Projection) -> Self {
        self.0.push(projection);
        self
    }

    pub fn extend<I>(mut self, projections: I) -> Self
    where I: IntoIterator<Item = Projection>,
    {
        self.0.extend(projections);
        self
    }

    pub fn field<F: Into<SmolStr>>(self, field: F) -> Self {
        self.add(Projection::Field(field.into()))
    }

    pub fn count_all(self) -> Self {
        self.add(Projection::CountAll)
    }

    pub fn count<F: Into<SmolStr>>(self, field: F) -> Self {
        self.add(Projection::Count(field.into()))
    }

    pub fn sum<F: Into<SmolStr>>(self, field: F) -> Self {
        self.add(Projection::Sum(field.into()))
    }

    pub fn avg<F: Into<SmolStr>>(self, field: F) -> Self {
        self.add(Projection::Avg(field.into()))
    }

    pub fn min<F: Into<SmolStr>>(self, field: F) -> Self {
        self.add(Projection::Min(field.into()))
    }

    pub fn max<F: Into<SmolStr>>(self, field: F) -> Self {
        self.add(Projection::Max(field.into()))
    }

    pub fn r#as<A: Into<SmolStr>>(self, proj: Projection, alias: A) -> Self {
        self.add(proj.r#as(alias))
    }

    pub fn build(self) -> SmallVec<[Projection; 10]> {
        self.0
    }
}

impl Default for ProjectionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[macro_export]
macro_rules! project {
    // Matches an empty macro invocation
    () => {
        $crate::query::ProjectionBuilder::new().build()
    };

    // Matches a comma-separated list of builder method calls
    ( $( $method:ident ( $( $arg:expr ),* ) ),+ $(,)? ) => {
        {
            let builder = $crate::query::ProjectionBuilder::new();
            $( let builder = builder.$method( $( $arg ),* ); )+
            builder.build()
        }
    };
}
