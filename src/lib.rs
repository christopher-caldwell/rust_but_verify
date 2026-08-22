#![doc = include_str!("../README.md")]

mod authorizer;
mod identifiers;
mod model;
mod typed;

pub use authorizer::{Access, Authorizer, Decision};
pub use identifiers::{Action, ActionId, RoleAssignment, RoleId, TargetKey};
pub use model::{Model, ModelBuilder, ModelError};
pub use typed::{Scope, ScopedBy, Target};
