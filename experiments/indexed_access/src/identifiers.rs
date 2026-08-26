use std::fmt;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct RoleId(String);

impl RoleId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for RoleId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for RoleId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl fmt::Display for RoleId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ActionId(String);

impl ActionId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ActionId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

pub trait Action {
    fn action_id(&self) -> ActionId;
}

impl Action for ActionId {
    fn action_id(&self) -> ActionId {
        self.clone()
    }
}

impl<A> Action for &A
where
    A: Action + ?Sized,
{
    fn action_id(&self) -> ActionId {
        (*self).action_id()
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct TargetKey {
    kind: String,
    identifier: String,
}

impl TargetKey {
    pub fn new(kind: impl Into<String>, identifier: impl ToString) -> Self {
        Self {
            kind: kind.into(),
            identifier: identifier.to_string(),
        }
    }

    pub fn kind(&self) -> &str {
        &self.kind
    }

    pub fn identifier(&self) -> &str {
        &self.identifier
    }
}

impl fmt::Display for TargetKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}::{}", self.kind, self.identifier)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoleAssignment {
    role: RoleId,
    target: TargetKey,
}

impl RoleAssignment {
    /// Records that `role` applies to this typed target.
    pub fn new<T>(role: impl Into<RoleId>, target: crate::Target<T>) -> Self
    where
        T: crate::TargetType,
    {
        Self::from_key(role, target.into_target_key())
    }

    /// Creates an assignment from an erased runtime target key.
    pub fn from_key(role: impl Into<RoleId>, target: TargetKey) -> Self {
        Self {
            role: role.into(),
            target,
        }
    }

    pub fn role(&self) -> &RoleId {
        &self.role
    }

    pub fn target(&self) -> &TargetKey {
        &self.target
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_values_preserve_consumer_data() {
        let role = RoleId::new("parent");
        let action = ActionId::new("scheduling.edit");
        let target = TargetKey::new("child", 123_u64);
        let assignment = RoleAssignment::from_key(role.clone(), target.clone());

        assert_eq!(role.as_str(), "parent");
        assert_eq!(action.as_str(), "scheduling.edit");
        assert_eq!(action.to_string(), "scheduling.edit");
        assert_eq!(target.kind(), "child");
        assert_eq!(target.identifier(), "123");
        assert_eq!(target.to_string(), "child::123");
        assert_eq!(assignment.role(), &role);
        assert_eq!(assignment.target(), &target);
    }
}
