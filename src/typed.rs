use std::marker::PhantomData;

use crate::ScopeKey;

/// An application-defined namespace for authorization scopes.
pub trait Scope {
    const NAMESPACE: &'static str;

    fn ident(identifier: impl ToString) -> Target<Self>
    where
        Self: Sized,
    {
        Target::new(identifier)
    }
}

/// A typed application boundary around an opaque runtime scope key.
pub struct Target<S> {
    key: ScopeKey,
    scope: PhantomData<fn() -> S>,
}

impl<S> Target<S>
where
    S: Scope,
{
    fn new(identifier: impl ToString) -> Self {
        Self {
            key: ScopeKey::new(S::NAMESPACE, identifier),
            scope: PhantomData,
        }
    }

    pub fn scope_key(&self) -> &ScopeKey {
        &self.key
    }

    pub fn into_scope_key(self) -> ScopeKey {
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
/// use rust_but_verify::{Action, ActionKey, Scope, ScopedBy, Target};
///
/// struct Edit;
/// impl Action for Edit {
///     fn action_key(&self) -> ActionKey {
///         ActionKey::new("scheduling", "edit")
///     }
/// }
///
/// struct Document;
/// impl Scope for Document {
///     const NAMESPACE: &'static str = "document";
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
    use crate::{Action, ActionKey};

    use super::*;

    #[derive(Clone, Copy)]
    struct Edit;

    impl Action for Edit {
        fn action_key(&self) -> ActionKey {
            ActionKey::new("scheduling", "edit")
        }
    }

    struct Child;

    impl Scope for Child {
        const NAMESPACE: &'static str = "child";
    }

    struct Schedule;

    impl Scope for Schedule {
        const NAMESPACE: &'static str = "schedule";
    }

    impl ScopedBy<Child> for Edit {}
    impl ScopedBy<Schedule> for Edit {}

    fn erase<A, S>(action: A, target: Target<S>) -> (ActionKey, ScopeKey)
    where
        A: ScopedBy<S>,
        S: Scope,
    {
        (action.action_key(), target.into_scope_key())
    }

    #[test]
    fn declared_pair_erases_to_runtime_keys() {
        let (action, scope) = erase(Edit, Child::ident(123_u64));

        assert_eq!(action, ActionKey::new("scheduling", "edit"));
        assert_eq!(scope, ScopeKey::new("child", "123"));
    }

    #[test]
    fn one_action_supports_multiple_scope_types() {
        let (_, child) = erase(Edit, Child::ident("123"));
        let (_, schedule) = erase(Edit, Schedule::ident("456"));

        assert_eq!(child, ScopeKey::new("child", "123"));
        assert_eq!(schedule, ScopeKey::new("schedule", "456"));
    }

    #[test]
    fn target_can_be_borrowed_as_an_erased_scope_key() {
        let target = Child::ident("123");

        assert_eq!(target.scope_key(), &ScopeKey::new("child", "123"));
    }
}
