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
pub struct ActionKey {
    namespace: String,
    name: String,
}

impl ActionKey {
    pub fn new(namespace: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
            name: name.into(),
        }
    }

    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl fmt::Display for ActionKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}.{}", self.namespace, self.name)
    }
}

pub trait Action {
    fn action_key(&self) -> ActionKey;
}

impl Action for ActionKey {
    fn action_key(&self) -> ActionKey {
        self.clone()
    }
}

impl<A> Action for &A
where
    A: Action + ?Sized,
{
    fn action_key(&self) -> ActionKey {
        (*self).action_key()
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ScopeKey {
    namespace: String,
    identifier: String,
}

impl ScopeKey {
    pub fn new(namespace: impl Into<String>, identifier: impl ToString) -> Self {
        Self {
            namespace: namespace.into(),
            identifier: identifier.to_string(),
        }
    }

    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    pub fn identifier(&self) -> &str {
        &self.identifier
    }
}

impl fmt::Display for ScopeKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}::{}", self.namespace, self.identifier)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoleAssignment {
    role: RoleId,
    scope: ScopeKey,
}

impl RoleAssignment {
    pub fn new(role: impl Into<RoleId>, scope: ScopeKey) -> Self {
        Self {
            role: role.into(),
            scope,
        }
    }

    pub fn role(&self) -> &RoleId {
        &self.role
    }

    pub fn scope(&self) -> &ScopeKey {
        &self.scope
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_values_preserve_consumer_data() {
        let role = RoleId::new("parent");
        let action = ActionKey::new("scheduling", "edit");
        let scope = ScopeKey::new("child", 123_u64);
        let assignment = RoleAssignment::new(role.clone(), scope.clone());

        assert_eq!(role.as_str(), "parent");
        assert_eq!(action.namespace(), "scheduling");
        assert_eq!(action.name(), "edit");
        assert_eq!(action.to_string(), "scheduling.edit");
        assert_eq!(scope.namespace(), "child");
        assert_eq!(scope.identifier(), "123");
        assert_eq!(scope.to_string(), "child::123");
        assert_eq!(assignment.role(), &role);
        assert_eq!(assignment.scope(), &scope);
    }
}
