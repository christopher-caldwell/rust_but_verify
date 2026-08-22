use std::marker::PhantomData;

use crate::{RoleAssignment, TargetKey};

/// An application-defined kind of authorization target.
pub trait Scope {
    const KIND: &'static str;

    fn ident(identifier: impl ToString) -> Target<Self>
    where
        Self: Sized,
    {
        Target::new(identifier)
    }
}

/// A typed application boundary around an opaque runtime target key.
pub struct Target<S> {
    key: TargetKey,
    scope: PhantomData<fn() -> S>,
}

impl RoleAssignment {
    /// Records that `role` applies to this typed target.
    pub fn for_target<S>(role: impl Into<crate::RoleId>, target: Target<S>) -> Self
    where
        S: Scope,
    {
        Self::new(role, target.into_target_key())
    }
}

impl<S> Target<S>
where
    S: Scope,
{
    fn new(identifier: impl ToString) -> Self {
        Self {
            key: TargetKey::new(S::KIND, identifier),
            scope: PhantomData,
        }
    }

    pub fn target_key(&self) -> &TargetKey {
        &self.key
    }

    pub fn into_target_key(self) -> TargetKey {
        self.key
    }
}

/// Declares that an action may be checked within a scope type.
///
/// Applications opt valid pairs in with ordinary trait implementations. A
/// pair without an implementation cannot satisfy the bound used by typed
/// authorization calls:
///
/// ```compile_fail
/// use rust_but_verify::{Action, ActionId, Scope, ScopedBy, Target};
///
/// struct Edit;
/// impl Action for Edit {
///     fn action_id(&self) -> ActionId {
///         ActionId::new("scheduling.edit")
///     }
/// }
///
/// struct Document;
/// impl Scope for Document {
///     const KIND: &'static str = "document";
/// }
///
/// fn accepts_pair<A, S>(_action: A, _target: Target<S>)
/// where
///     A: ScopedBy<S>,
///     S: Scope,
/// {}
///
/// // No `impl ScopedBy<Document> for Edit` exists.
/// accepts_pair(Edit, Document::ident("789"));
/// ```
pub trait ScopedBy<S>: crate::Action
where
    S: Scope,
{
}

#[cfg(test)]
mod tests {
    use crate::{Action, ActionId};

    use super::*;

    #[derive(Clone, Copy)]
    struct Edit;

    impl Action for Edit {
        fn action_id(&self) -> ActionId {
            ActionId::new("scheduling.edit")
        }
    }

    struct Child;

    impl Scope for Child {
        const KIND: &'static str = "child";
    }

    struct Schedule;

    impl Scope for Schedule {
        const KIND: &'static str = "schedule";
    }

    impl ScopedBy<Child> for Edit {}
    impl ScopedBy<Schedule> for Edit {}

    fn erase<A, S>(action: A, target: Target<S>) -> (ActionId, TargetKey)
    where
        A: ScopedBy<S>,
        S: Scope,
    {
        (action.action_id(), target.into_target_key())
    }

    #[test]
    fn declared_pair_erases_to_runtime_keys() {
        let (action, scope) = erase(Edit, Child::ident(123_u64));

        assert_eq!(action, ActionId::new("scheduling.edit"));
        assert_eq!(scope, TargetKey::new("child", "123"));
    }

    #[test]
    fn one_action_supports_multiple_scope_types() {
        let (_, child) = erase(Edit, Child::ident("123"));
        let (_, schedule) = erase(Edit, Schedule::ident("456"));

        assert_eq!(child, TargetKey::new("child", "123"));
        assert_eq!(schedule, TargetKey::new("schedule", "456"));
    }

    #[test]
    fn target_can_be_borrowed_as_an_erased_target_key() {
        let target = Child::ident("123");

        assert_eq!(target.target_key(), &TargetKey::new("child", "123"));
    }
}
