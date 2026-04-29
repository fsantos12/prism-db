use smallvec::SmallVec;
use smol_str::SmolStr;

#[derive(Debug, Clone)]
pub enum Projection {
    Field(SmolStr),
    Aliased(Box<Projection>, SmolStr),
    CountAll,
    Count(SmolStr),
    Sum(SmolStr),
    Avg(SmolStr),
    Min(SmolStr),
    Max(SmolStr),
}

impl Projection {
    pub fn r#as<S: Into<SmolStr>>(self, alias: S) -> Self {
        Projection::Aliased(Box::new(self), alias.into())
    }
}

pub type ProjectionDefinition = SmallVec<[Projection; 10]>;