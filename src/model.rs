use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt;

use crate::{Action, ActionId, RoleId};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Policy {
    grants: HashMap<RoleId, HashSet<ActionId>>,
}

impl Policy {
    pub fn builder() -> PolicyBuilder {
        PolicyBuilder::new()
    }

    pub fn role_grants(&self, role: &RoleId, action: &ActionId) -> bool {
        self.grants
            .get(role)
            .is_some_and(|actions| actions.contains(action))
    }
}

#[derive(Clone, Debug, Default)]
pub struct PolicyBuilder {
    grants: HashMap<RoleId, HashSet<ActionId>>,
    known_actions: Option<HashSet<ActionId>>,
}

impl PolicyBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn allow(&mut self, role: impl Into<RoleId>, action: impl Action) -> &mut Self {
        self.grants
            .entry(role.into())
            .or_default()
            .insert(action.action_id());
        self
    }

    pub fn validate_actions<I, A>(&mut self, actions: I) -> &mut Self
    where
        I: IntoIterator<Item = A>,
        A: Action,
    {
        self.known_actions = Some(
            actions
                .into_iter()
                .map(|action| action.action_id())
                .collect(),
        );
        self
    }

    pub fn build(self) -> Result<Policy, PolicyError> {
        for (role, actions) in &self.grants {
            if role.as_str().is_empty() {
                return Err(PolicyError::EmptyRole);
            }

            for action in actions {
                if action.as_str().is_empty() {
                    return Err(PolicyError::EmptyActionId {
                        action: action.clone(),
                    });
                }

                if self
                    .known_actions
                    .as_ref()
                    .is_some_and(|known| !known.contains(action))
                {
                    return Err(PolicyError::UnknownAction {
                        action: action.clone(),
                    });
                }
            }
        }

        Ok(Policy {
            grants: self.grants,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PolicyError {
    EmptyRole,
    EmptyActionId { action: ActionId },
    UnknownAction { action: ActionId },
}

impl fmt::Display for PolicyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyRole => formatter.write_str("role identifier must not be empty"),
            Self::EmptyActionId { action } => {
                write!(formatter, "action identifier `{action}` must not be empty")
            }
            Self::UnknownAction { action } => write!(formatter, "unknown action `{action}`"),
        }
    }
}

impl Error for PolicyError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy)]
    struct Edit;

    impl Action for Edit {
        fn action_id(&self) -> ActionId {
            ActionId::new("scheduling.edit")
        }
    }

    #[test]
    fn builds_runtime_defined_roles_and_grants() {
        let edit = ActionId::new("scheduling.edit");
        let mut builder = Policy::builder();
        builder.allow("parent", edit.clone());

        let policy = builder.build().expect("runtime policy should be valid");

        assert!(policy.role_grants(&RoleId::new("parent"), &edit));
        assert!(!policy.role_grants(&RoleId::new("youth"), &edit));
    }

    #[test]
    fn accepts_typed_actions_without_catalog_registration() {
        let mut builder = Policy::builder();
        builder.allow("parent", Edit);

        let policy = builder.build().expect("typed allow should be valid");

        assert!(policy.role_grants(&RoleId::new("parent"), &ActionId::new("scheduling.edit")));
    }

    #[test]
    fn optional_validation_rejects_unknown_actions() {
        let edit = ActionId::new("scheduling.edit");
        let typo = ActionId::new("scheduling.edti");
        let mut builder = Policy::builder();
        builder
            .validate_actions([edit])
            .allow("parent", typo.clone());

        assert_eq!(
            builder.build(),
            Err(PolicyError::UnknownAction { action: typo })
        );
    }

    #[test]
    fn runtime_actions_do_not_require_a_catalog() {
        let custom = ActionId::new("application.runtime-defined");
        let mut builder = Policy::builder();
        builder.allow("custom-role", custom.clone());

        let policy = builder.build().expect("catalog should be optional");

        assert!(policy.role_grants(&RoleId::new("custom-role"), &custom));
    }

    #[test]
    fn rejects_empty_runtime_identifiers() {
        let mut empty_role = Policy::builder();
        empty_role.allow("", ActionId::new("scheduling.edit"));
        assert_eq!(empty_role.build(), Err(PolicyError::EmptyRole));

        let empty_name = ActionId::new("");
        let mut empty_action = Policy::builder();
        empty_action.allow("parent", empty_name.clone());
        assert_eq!(
            empty_action.build(),
            Err(PolicyError::EmptyActionId { action: empty_name })
        );
    }
}
