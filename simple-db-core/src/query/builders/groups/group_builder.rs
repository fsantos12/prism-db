use smallvec::SmallVec;
use smol_str::SmolStr;

use crate::query::builders::GroupDefinition;

pub struct GroupBuilder(GroupDefinition);

impl GroupBuilder {
    pub fn new() -> Self {
        Self(SmallVec::new())
    }

    pub fn field<F: Into<SmolStr>>(mut self, field: F) -> Self {
        self.0.push(field.into());
        self
    }

    pub fn fields<F, I>(mut self, fields: I) -> Self
    where F: Into<SmolStr>, I: IntoIterator<Item = F> {
        self.0.extend(fields.into_iter().map(Into::into));
        self
    }

    pub fn build(self) -> SmallVec<[SmolStr; 4]> {
        self.0
    }
}

impl Default for GroupBuilder {
    fn default() -> Self {
        Self::new()
    }
}

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