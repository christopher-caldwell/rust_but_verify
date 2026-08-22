use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::{
    Action, ActionId, ActionOn, Policy, PolicyBuilder, PolicyError, RoleAssignment, RoleId, Target,
    TargetKey, TargetType, Targets,
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
    pub fn with_role<S>(role: impl Into<RoleId>, target: Target<S>) -> Self
    where
        S: TargetType,
    {
        Self::new([RoleAssignment::new(role, target)])
    }
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Decision {
    Allow {
        via_role: RoleId,
        via_target: TargetKey,
    },
    Deny,
}

impl Decision {
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

    /// Returns the first matching role and target for one typed target.
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
    /// use rust_but_verify::{Access, Action, ActionId, ActionOn, Authorizer, Policy, TargetType};
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

    /// Returns the first successful multi-target path. Tuple order or
    /// [`Targets`] insertion order takes precedence over access assignment
    /// order. This is provenance, not an exhaustive explanation.
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

    pub fn can_runtime(&self, access: &Access, action: &ActionId, target: &TargetKey) -> bool {
        let policy = self.read_policy();
        match_assignment(&policy, access, action, target).is_some()
    }

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
    access.assignments().iter().find(|assignment| {
        assignment.target() == target && policy.allows(assignment.role(), action)
    })
}

fn decision_for(assignment: Option<&RoleAssignment>, target: &TargetKey) -> Decision {
    assignment.map_or(Decision::Deny, |assignment| Decision::Allow {
        via_role: assignment.role().clone(),
        via_target: target.clone(),
    })
}
