use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::{
    Action, ActionId, ActionOn, Policy, PolicyBuilder, PolicyError, RoleAssignment, RoleId, Target,
    TargetKey, TargetType,
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

    /// Creates access containing one role assignment at a typed target.
    pub fn with_role<S>(role: impl Into<RoleId>, target: Target<S>) -> Self
    where
        S: TargetType,
    {
        Self::new([RoleAssignment::new(role, target)])
    }

    /// Adds one role assignment at a typed target.
    pub fn add_role<S>(&mut self, role: impl Into<RoleId>, target: Target<S>) -> &mut Self
    where
        S: TargetType,
    {
        self.push(RoleAssignment::new(role, target));
        self
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

#[cfg(test)]
mod access_tests {
    use crate::{RoleAssignment, TargetType};

    use super::Access;

    struct Document;

    impl TargetType for Document {
        const KIND: &'static str = "document";
    }

    #[test]
    fn construction_patterns_preserve_supplied_facts() {
        let empty = Access::empty();
        let single = Access::with_role("reader", Document::ident(1));
        let from_iterator: Access = [RoleAssignment::new("writer", Document::ident(2))]
            .into_iter()
            .collect();
        let mut chained = Access::empty();
        chained
            .add_role("reader", Document::ident(3))
            .add_role("writer", Document::ident(4));

        assert!(empty.assignments().is_empty());
        assert_eq!(single.assignments().len(), 1);
        assert_eq!(from_iterator.assignments().len(), 1);
        assert_eq!(chained.assignments().len(), 2);
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Decision {
    Allow {
        via_role: RoleId,
        via_target: TargetKey,
    },
    Deny,
}

/// A heterogeneous tuple of typed targets accepted by [`Authorizer::can_any`].
///
/// Implementations are provided for tuples containing two through four targets.
/// Each target must be declared compatible with the requested action.
pub trait CanAnyTargets<A>: Sized {
    #[doc(hidden)]
    fn evaluate(self, authorizer: &Authorizer, access: &Access, action: &ActionId) -> Decision;
}

macro_rules! impl_can_any_targets {
    ($($target:ident),+) => {
        impl<A, $($target),+> CanAnyTargets<A> for ($(Target<$target>,)+)
        where
            $(A: ActionOn<$target>, $target: TargetType,)+
        {
            #[allow(non_snake_case)]
            fn evaluate(self, authorizer: &Authorizer, access: &Access, action: &ActionId) -> Decision {
                let ($($target,)+) = self;
                $(
                    let decision = authorizer.can_runtime(access, action, $target.target_key());
                    if decision.is_allowed() {
                        return decision;
                    }
                )+
                Decision::Deny
            }
        }
    };
}

impl_can_any_targets!(First, Second);
impl_can_any_targets!(First, Second, Third);
impl_can_any_targets!(First, Second, Third, Fourth);

impl Decision {
    pub fn is_allowed(&self) -> bool {
        matches!(self, Self::Allow { .. })
    }
}

#[derive(Debug)]
pub struct Authorizer {
    policy: RwLock<Policy>,
}

impl Authorizer {
    pub fn new(policy: Policy) -> Self {
        Self {
            policy: RwLock::new(policy),
        }
    }

    pub fn can<A, S>(&self, access: &Access, action: A, target: Target<S>) -> Decision
    where
        A: ActionOn<S>,
        S: TargetType,
    {
        self.can_runtime(access, &action.action_id(), target.target_key())
    }

    /// Checks typed candidate targets in tuple order using the same exact-match
    /// evaluation as [`Self::can`].
    ///
    /// ```compile_fail
    /// use rust_but_verify::{Access, Action, ActionId, ActionOn, Authorizer, Policy, TargetType};
    ///
    /// struct Edit;
    /// impl Action for Edit { fn action_id(&self) -> ActionId { ActionId::new("edit") } }
    /// struct Article;
    /// impl TargetType for Article { const KIND: &'static str = "article"; }
    /// struct Wiki;
    /// impl TargetType for Wiki { const KIND: &'static str = "wiki"; }
    /// impl ActionOn<Article> for Edit {}
    ///
    /// let authz = Authorizer::new(Policy::builder().build().unwrap());
    /// authz.can_any(&Access::empty(), Edit, (Article::ident(1), Wiki::ident(1)));
    /// ```
    pub fn can_any<A, T>(&self, access: &Access, action: A, targets: T) -> Decision
    where
        A: Action,
        T: CanAnyTargets<A>,
    {
        targets.evaluate(self, access, &action.action_id())
    }

    pub fn can_runtime(&self, access: &Access, action: &ActionId, target: &TargetKey) -> Decision {
        let policy = self.read_policy();

        access
            .assignments()
            .iter()
            .find(|assignment| {
                assignment.target() == target && policy.role_grants(assignment.role(), action)
            })
            .map_or(Decision::Deny, |assignment| Decision::Allow {
                via_role: assignment.role().clone(),
                via_target: target.clone(),
            })
    }

    pub fn replace_policy(&self, policy: Policy) {
        *self.write_policy() = policy;
    }

    pub fn try_replace(&self, candidate: PolicyBuilder) -> Result<(), PolicyError> {
        let policy = candidate.build()?;
        self.replace_policy(policy);
        Ok(())
    }

    fn read_policy(&self) -> RwLockReadGuard<'_, Policy> {
        self.policy
            .read()
            .unwrap_or_else(|error| error.into_inner())
    }

    fn write_policy(&self) -> RwLockWriteGuard<'_, Policy> {
        self.policy
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
        fn action_id(&self) -> ActionId {
            ActionId::new("scheduling.edit")
        }
    }

    #[derive(Clone, Copy)]
    struct Delete;

    impl Action for Delete {
        fn action_id(&self) -> ActionId {
            ActionId::new("scheduling.delete")
        }
    }

    struct Child;

    impl TargetType for Child {
        const KIND: &'static str = "child";
    }

    impl ActionOn<Child> for Edit {}
    impl ActionOn<Child> for Delete {}

    fn policy_with(role: &str, action: impl Action) -> Policy {
        let mut builder = Policy::builder();
        builder.allow(role, action);
        builder.build().expect("test policy should be valid")
    }

    fn assigned(role: &str, child: &str) -> RoleAssignment {
        RoleAssignment::from_key(role, TargetKey::new("child", child))
    }

    #[test]
    fn matching_assignment_and_grant_allows() {
        let authorizer = Authorizer::new(policy_with("parent", Edit));
        let access = Access::new([assigned("parent", "123")]);

        assert_eq!(
            authorizer.can(&access, Edit, Child::ident("123")),
            Decision::Allow {
                via_role: RoleId::new("parent"),
                via_target: TargetKey::new("child", "123"),
            }
        );
    }

    #[test]
    fn missing_facts_default_to_deny() {
        let authorizer = Authorizer::new(policy_with("parent", Edit));
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
                &ActionId::new("scheduling.unknown"),
                &TargetKey::new("child", "123")
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
        let authorizer = Authorizer::new(policy_with("parent", Edit));
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
    fn adding_access_facts_requires_no_policy_replacement() {
        let authorizer = Authorizer::new(policy_with("parent", Edit));
        let mut access = Access::empty();

        assert_eq!(
            authorizer.can(&access, Edit, Child::ident("123")),
            Decision::Deny
        );

        access.add_role("parent", Child::ident("123"));

        assert!(
            authorizer
                .can(&access, Edit, Child::ident("123"))
                .is_allowed()
        );
    }

    #[test]
    fn successful_replacement_changes_subsequent_decisions() {
        let authorizer = Authorizer::new(policy_with("parent", Edit));
        let access = Access::new([assigned("parent", "123")]);
        assert!(
            authorizer
                .can(&access, Edit, Child::ident("123"))
                .is_allowed()
        );

        authorizer.replace_policy(policy_with("youth", Edit));

        assert_eq!(
            authorizer.can(&access, Edit, Child::ident("123")),
            Decision::Deny
        );
    }

    #[test]
    fn failed_candidate_leaves_active_policy_usable() {
        let authorizer = Authorizer::new(policy_with("parent", Edit));
        let access = Access::new([assigned("parent", "123")]);
        let edit = ActionId::new("scheduling.edit");
        let mut invalid = Policy::builder();
        invalid
            .validate_actions([edit])
            .allow("parent", ActionId::new("scheduling.edti"));

        assert!(matches!(
            authorizer.try_replace(invalid),
            Err(PolicyError::UnknownAction { .. })
        ));
        assert!(
            authorizer
                .can(&access, Edit, Child::ident("123"))
                .is_allowed()
        );
    }
}
