use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::{
    ActionKey, Model, ModelBuilder, ModelError, RoleAssignment, RoleId, Scope, ScopeKey, ScopedBy,
    Target,
};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Access {
    assignments: Vec<RoleAssignment>,
}

impl Access {
    pub fn new(assignments: impl IntoIterator<Item = RoleAssignment>) -> Self {
        Self {
            assignments: assignments.into_iter().collect(),
        }
    }

    pub fn empty() -> Self {
        Self::default()
    }

    pub fn push(&mut self, assignment: RoleAssignment) {
        self.assignments.push(assignment);
    }

    pub fn assignments(&self) -> &[RoleAssignment] {
        &self.assignments
    }
}

impl FromIterator<RoleAssignment> for Access {
    fn from_iter<T: IntoIterator<Item = RoleAssignment>>(assignments: T) -> Self {
        Self::new(assignments)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Decision {
    Allow { via_role: RoleId },
    Deny,
}

impl Decision {
    pub fn is_allowed(&self) -> bool {
        matches!(self, Self::Allow { .. })
    }
}

#[derive(Debug)]
pub struct Authorizer {
    model: RwLock<Model>,
}

impl Authorizer {
    pub fn new(model: Model) -> Self {
        Self {
            model: RwLock::new(model),
        }
    }

    pub fn can<A, S>(&self, access: &Access, action: A, target: Target<S>) -> Decision
    where
        A: ScopedBy<S>,
        S: Scope,
    {
        self.can_runtime(access, &action.action_key(), target.scope_key())
    }

    pub fn can_runtime(&self, access: &Access, action: &ActionKey, scope: &ScopeKey) -> Decision {
        let model = self.read_model();

        access
            .assignments()
            .iter()
            .find(|assignment| {
                assignment.scope() == scope && model.role_grants(assignment.role(), action)
            })
            .map_or(Decision::Deny, |assignment| Decision::Allow {
                via_role: assignment.role().clone(),
            })
    }

    pub fn replace_model(&self, model: Model) {
        *self.write_model() = model;
    }

    pub fn try_replace(&self, candidate: ModelBuilder) -> Result<(), ModelError> {
        let model = candidate.build()?;
        self.replace_model(model);
        Ok(())
    }

    fn read_model(&self) -> RwLockReadGuard<'_, Model> {
        self.model.read().unwrap_or_else(|error| error.into_inner())
    }

    fn write_model(&self) -> RwLockWriteGuard<'_, Model> {
        self.model
            .write()
            .unwrap_or_else(|error| error.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use crate::Action;

    use super::*;

    #[derive(Clone, Copy)]
    struct Edit;

    impl Action for Edit {
        fn action_key(&self) -> ActionKey {
            ActionKey::new("scheduling", "edit")
        }
    }

    #[derive(Clone, Copy)]
    struct Delete;

    impl Action for Delete {
        fn action_key(&self) -> ActionKey {
            ActionKey::new("scheduling", "delete")
        }
    }

    struct Child;

    impl Scope for Child {
        const NAMESPACE: &'static str = "child";
    }

    impl ScopedBy<Child> for Edit {}
    impl ScopedBy<Child> for Delete {}

    fn model_with(role: &str, action: impl Action) -> Model {
        let mut builder = Model::builder();
        builder.grant(role, action);
        builder.build().expect("test model should be valid")
    }

    fn assigned(role: &str, child: &str) -> RoleAssignment {
        RoleAssignment::new(role, ScopeKey::new("child", child))
    }

    #[test]
    fn matching_assignment_and_grant_allows() {
        let authorizer = Authorizer::new(model_with("parent", Edit));
        let access = Access::new([assigned("parent", "123")]);

        assert_eq!(
            authorizer.can(&access, Edit, Child::ident("123")),
            Decision::Allow {
                via_role: RoleId::new("parent")
            }
        );
    }

    #[test]
    fn missing_facts_default_to_deny() {
        let authorizer = Authorizer::new(model_with("parent", Edit));
        let matching = Access::new([assigned("parent", "123")]);

        assert_eq!(
            authorizer.can(&matching, Delete, Child::ident("123")),
            Decision::Deny
        );
        assert_eq!(
            authorizer.can(&matching, Edit, Child::ident("456")),
            Decision::Deny
        );
        assert_eq!(
            authorizer.can(
                &Access::new([assigned("unknown", "123")]),
                Edit,
                Child::ident("123")
            ),
            Decision::Deny
        );
        assert_eq!(
            authorizer.can_runtime(
                &matching,
                &ActionKey::new("scheduling", "unknown"),
                &ScopeKey::new("child", "123")
            ),
            Decision::Deny
        );
        assert_eq!(
            authorizer.can(&Access::empty(), Edit, Child::ident("123")),
            Decision::Deny
        );
    }

    #[test]
    fn any_matching_assignment_can_allow() {
        let authorizer = Authorizer::new(model_with("parent", Edit));
        let access = Access::new([
            assigned("parent", "wrong-child"),
            assigned("youth", "123"),
            assigned("parent", "123"),
        ]);

        assert!(
            authorizer
                .can(&access, Edit, Child::ident("123"))
                .is_allowed()
        );
    }

    #[test]
    fn successful_replacement_changes_subsequent_decisions() {
        let authorizer = Authorizer::new(model_with("parent", Edit));
        let access = Access::new([assigned("parent", "123")]);
        assert!(
            authorizer
                .can(&access, Edit, Child::ident("123"))
                .is_allowed()
        );

        authorizer.replace_model(model_with("youth", Edit));

        assert_eq!(
            authorizer.can(&access, Edit, Child::ident("123")),
            Decision::Deny
        );
    }

    #[test]
    fn failed_candidate_leaves_active_model_usable() {
        let authorizer = Authorizer::new(model_with("parent", Edit));
        let access = Access::new([assigned("parent", "123")]);
        let edit = ActionKey::new("scheduling", "edit");
        let mut invalid = Model::builder();
        invalid
            .validate_actions([edit])
            .grant("parent", ActionKey::new("scheduling", "edti"));

        assert!(matches!(
            authorizer.try_replace(invalid),
            Err(ModelError::UnknownAction { .. })
        ));
        assert!(
            authorizer
                .can(&access, Edit, Child::ident("123"))
                .is_allowed()
        );
    }
}
