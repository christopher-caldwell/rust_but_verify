use std::collections::HashMap;
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::{
    Action, ActionId, ActionOn, Policy, PolicyBuilder, PolicyError, RoleAssignment, RoleId, Target,
    TargetKey, TargetType, Targets,
};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Access {
    assignments: Vec<RoleAssignment>,
    by_target: HashMap<TargetKey, Vec<usize>>,
}

impl Access {
    /// Creates access from role assignments in their supplied order.
    pub fn new(assignments: impl IntoIterator<Item = RoleAssignment>) -> Self {
        let mut access = Self::empty();
        for assignment in assignments {
            access.push(assignment);
        }
        access
    }
    /// Creates access with no role assignments.
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
    /// Appends a role assignment, preserving assignment evaluation order.
    pub fn push(&mut self, assignment: RoleAssignment) {
        self.by_target
            .entry(assignment.target().clone())
            .or_default()
            .push(self.assignments.len());
        self.assignments.push(assignment);
    }
    /// Returns role assignments in their evaluation order.
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
/// Optional first-success provenance for an authorization check.
///
/// This is not an exhaustive explanation engine. When multiple paths could
/// allow a check, [`Authorizer::decide`] and [`Authorizer::decide_any`] return
/// only the first successful path in their documented evaluation order.
pub enum Decision {
    /// The role assignment and target that first allowed the action.
    Allow {
        via_role: RoleId,
        via_target: TargetKey,
    },
    /// No supplied role assignment allowed the action on a candidate target.
    Deny,
}

impl Decision {
    /// Returns whether this provenance result allowed the action.
    ///
    /// Prefer [`Authorizer::can`] or [`Authorizer::can_any`] when provenance is
    /// not needed: those methods answer the usual authorization question
    /// directly with `bool`.
    pub fn is_allowed(&self) -> bool {
        matches!(self, Self::Allow { .. })
    }
}

pub(crate) mod sealed {
    pub trait CanAnyTargets {}
}

/// Hidden implementation machinery for supported multi-target inputs.
///
/// This trait is sealed: pass an exactly-two-target tuple or [`Targets`] to
/// [`Authorizer::can_any`] and [`Authorizer::decide_any`].
#[doc(hidden)]
pub trait CanAnyTargets<A>: sealed::CanAnyTargets + Sized {
    #[doc(hidden)]
    fn evaluate<R, F>(
        self,
        policy: &Policy,
        access: &Access,
        action: &ActionId,
        on_match: F,
    ) -> Option<R>
    where
        F: FnMut(&RoleAssignment, &TargetKey) -> R;
}

impl<First, Second> sealed::CanAnyTargets for (Target<First>, Target<Second>) {}

impl<A, First, Second> CanAnyTargets<A> for (Target<First>, Target<Second>)
where
    A: ActionOn<First> + ActionOn<Second>,
    First: TargetType,
    Second: TargetType,
{
    fn evaluate<R, F>(
        self,
        policy: &Policy,
        access: &Access,
        action: &ActionId,
        mut on_match: F,
    ) -> Option<R>
    where
        F: FnMut(&RoleAssignment, &TargetKey) -> R,
    {
        let (first, second) = self;
        match_assignment(policy, access, action, first.target_key())
            .map(|assignment| on_match(assignment, first.target_key()))
            .or_else(|| {
                match_assignment(policy, access, action, second.target_key())
                    .map(|assignment| on_match(assignment, second.target_key()))
            })
    }
}

impl<A> sealed::CanAnyTargets for Targets<A> {}

impl<A> CanAnyTargets<A> for Targets<A> {
    fn evaluate<R, F>(
        self,
        policy: &Policy,
        access: &Access,
        action: &ActionId,
        mut on_match: F,
    ) -> Option<R>
    where
        F: FnMut(&RoleAssignment, &TargetKey) -> R,
    {
        self.targets.into_iter().find_map(|target| {
            match_assignment(policy, access, action, &target)
                .map(|assignment| on_match(assignment, &target))
        })
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

    /// Answers whether an action is allowed on one typed target.
    pub fn can<A, S>(&self, access: &Access, action: A, target: Target<S>) -> bool
    where
        A: ActionOn<S>,
        S: TargetType,
    {
        self.can_runtime(access, &action.action_id(), target.target_key())
    }

    /// Returns optional first-success provenance for one typed target.
    ///
    /// Prefer [`Self::can`] for an ordinary yes/no authorization decision.
    /// This method reports the first matching access assignment, not every
    /// possible reason an action could be allowed.
    pub fn decide<A, S>(&self, access: &Access, action: A, target: Target<S>) -> Decision
    where
        A: ActionOn<S>,
        S: TargetType,
    {
        self.decide_runtime(access, &action.action_id(), target.target_key())
    }

    /// Answers whether an action is allowed by a typed pair or [`Targets`]. A
    /// single policy read guard spans all candidates, so a call observes one
    /// complete policy snapshot.
    ///
    /// ```compile_fail
    /// use rust_but_verify_indexed_experiment::{Access, Action, ActionId, ActionOn, Authorizer, Policy, TargetType};
    /// struct Edit; impl Action for Edit { fn action_id(&self) -> ActionId { ActionId::new("edit") } }
    /// struct Article; impl TargetType for Article { const KIND: &'static str = "article"; }
    /// struct Wiki; impl TargetType for Wiki { const KIND: &'static str = "wiki"; }
    /// impl ActionOn<Article> for Edit {}
    /// let authz = Authorizer::new(Policy::builder().build().unwrap());
    /// authz.can_any(&Access::empty(), Edit, (Article::ident(1), Wiki::ident(1)));
    /// ```
    pub fn can_any<A, T>(&self, access: &Access, action: A, targets: T) -> bool
    where
        A: Action,
        T: CanAnyTargets<A>,
    {
        let policy = self.read_policy();
        targets
            .evaluate(&policy, access, &action.action_id(), |_, _| ())
            .is_some()
    }

    /// Returns optional first-success provenance for a typed pair or
    /// [`Targets`].
    ///
    /// One policy snapshot spans the complete multi-target operation. Tuple
    /// order, or [`Targets`] insertion order, takes precedence over access
    /// assignment order. This is provenance, not an exhaustive explanation;
    /// use [`Self::can_any`] for an ordinary yes/no check.
    pub fn decide_any<A, T>(&self, access: &Access, action: A, targets: T) -> Decision
    where
        A: Action,
        T: CanAnyTargets<A>,
    {
        let policy = self.read_policy();
        targets
            .evaluate(
                &policy,
                access,
                &action.action_id(),
                |assignment, target| Decision::Allow {
                    via_role: assignment.role().clone(),
                    via_target: target.clone(),
                },
            )
            .unwrap_or(Decision::Deny)
    }

    /// Answers whether an action is allowed on one runtime target key.
    ///
    /// The runtime form has the same matching semantics as [`Self::can`], but
    /// accepts opaque action and target identifiers supplied by the caller.
    pub fn can_runtime(&self, access: &Access, action: &ActionId, target: &TargetKey) -> bool {
        let policy = self.read_policy();
        match_assignment(&policy, access, action, target).is_some()
    }

    /// Returns optional first-success provenance for one runtime target key.
    ///
    /// Prefer [`Self::can_runtime`] when only the boolean answer is needed.
    pub fn decide_runtime(
        &self,
        access: &Access,
        action: &ActionId,
        target: &TargetKey,
    ) -> Decision {
        let policy = self.read_policy();
        decision_for(match_assignment(&policy, access, action, target), target)
    }

    pub fn replace_policy(&self, policy: Policy) {
        *self.write_policy() = policy;
    }
    pub fn try_replace(&self, candidate: PolicyBuilder) -> Result<(), PolicyError> {
        self.replace_policy(candidate.build()?);
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

/// The one matching primitive shared by boolean and provenance APIs.
fn match_assignment<'a>(
    policy: &Policy,
    access: &'a Access,
    action: &ActionId,
    target: &TargetKey,
) -> Option<&'a RoleAssignment> {
    access.by_target.get(target)?.iter().find_map(|index| {
        let assignment = &access.assignments[*index];
        policy
            .allows(assignment.role(), action)
            .then_some(assignment)
    })
}

fn decision_for(assignment: Option<&RoleAssignment>, target: &TargetKey) -> Decision {
    assignment.map_or(Decision::Deny, |assignment| Decision::Allow {
        via_role: assignment.role().clone(),
        via_target: target.clone(),
    })
}
