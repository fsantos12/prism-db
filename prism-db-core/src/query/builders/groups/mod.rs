mod group_builder;

use smallvec::SmallVec;
use smol_str::SmolStr;

pub use group_builder::GroupBuilder;

pub type GroupDefinition = SmallVec<[SmolStr; 4]>;