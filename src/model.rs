use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt;

use crate::{Action, ActionKey, RoleId};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Model {
    grants: HashMap<RoleId, HashSet<ActionKey>>,
}

impl Model {
    pub fn builder() -> ModelBuilder {
        ModelBuilder::new()
    }

    pub fn role_grants(&self, role: &RoleId, action: &ActionKey) -> bool {
        self.grants
            .get(role)
            .is_some_and(|actions| actions.contains(action))
    }
}

#[derive(Clone, Debug, Default)]
pub struct ModelBuilder {
    grants: HashMap<RoleId, HashSet<ActionKey>>,
    known_actions: Option<HashSet<ActionKey>>,
}

impl ModelBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn grant(&mut self, role: impl Into<RoleId>, action: impl Action) -> &mut Self {
        self.grants
            .entry(role.into())
            .or_default()
            .insert(action.action_key());
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
                .map(|action| action.action_key())
                .collect(),
        );
        self
    }

    pub fn build(self) -> Result<Model, ModelError> {
        for (role, actions) in &self.grants {
            if role.as_str().is_empty() {
                return Err(ModelError::EmptyRole);
            }

            for action in actions {
                if action.namespace().is_empty() || action.name().is_empty() {
                    return Err(ModelError::EmptyActionPart {
                        action: action.clone(),
                    });
                }

                if self
                    .known_actions
                    .as_ref()
                    .is_some_and(|known| !known.contains(action))
                {
                    return Err(ModelError::UnknownAction {
                        action: action.clone(),
                    });
                }
            }
        }

        Ok(Model {
            grants: self.grants,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ModelError {
    EmptyRole,
    EmptyActionPart { action: ActionKey },
    UnknownAction { action: ActionKey },
}

impl fmt::Display for ModelError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyRole => formatter.write_str("role identifier must not be empty"),
            Self::EmptyActionPart { action } => {
                write!(
                    formatter,
                    "action `{action}` must have a namespace and name"
                )
            }
            Self::UnknownAction { action } => write!(formatter, "unknown action `{action}`"),
        }
    }
}

impl Error for ModelError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy)]
    struct Edit;

    impl Action for Edit {
        fn action_key(&self) -> ActionKey {
            ActionKey::new("scheduling", "edit")
        }
    }

    #[test]
    fn builds_runtime_defined_roles_and_grants() {
        let edit = ActionKey::new("scheduling", "edit");
        let mut builder = Model::builder();
        builder.grant("parent", edit.clone());

        let model = builder.build().expect("runtime model should be valid");

        assert!(model.role_grants(&RoleId::new("parent"), &edit));
        assert!(!model.role_grants(&RoleId::new("youth"), &edit));
    }

    #[test]
    fn accepts_typed_actions_without_catalog_registration() {
        let mut builder = Model::builder();
        builder.grant("parent", Edit);

        let model = builder.build().expect("typed grant should be valid");

        assert!(model.role_grants(
            &RoleId::new("parent"),
            &ActionKey::new("scheduling", "edit")
        ));
    }

    #[test]
    fn optional_validation_rejects_unknown_actions() {
        let edit = ActionKey::new("scheduling", "edit");
        let typo = ActionKey::new("scheduling", "edti");
        let mut builder = Model::builder();
        builder
            .validate_actions([edit])
            .grant("parent", typo.clone());

        assert_eq!(
            builder.build(),
            Err(ModelError::UnknownAction { action: typo })
        );
    }

    #[test]
    fn runtime_actions_do_not_require_a_catalog() {
        let custom = ActionKey::new("application", "runtime-defined");
        let mut builder = Model::builder();
        builder.grant("custom-role", custom.clone());

        let model = builder.build().expect("catalog should be optional");

        assert!(model.role_grants(&RoleId::new("custom-role"), &custom));
    }

    #[test]
    fn rejects_empty_runtime_identifiers() {
        let mut empty_role = Model::builder();
        empty_role.grant("", ActionKey::new("scheduling", "edit"));
        assert_eq!(empty_role.build(), Err(ModelError::EmptyRole));

        let empty_name = ActionKey::new("scheduling", "");
        let mut empty_action = Model::builder();
        empty_action.grant("parent", empty_name.clone());
        assert_eq!(
            empty_action.build(),
            Err(ModelError::EmptyActionPart { action: empty_name })
        );
    }
}
