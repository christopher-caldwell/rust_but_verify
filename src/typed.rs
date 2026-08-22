use std::marker::PhantomData;

use crate::TargetKey;

/// An application-defined kind of authorization target.
pub trait TargetType {
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
    target_type: PhantomData<fn() -> S>,
}

impl<S> Target<S>
where
    S: TargetType,
{
    fn new(identifier: impl ToString) -> Self {
        Self {
            key: TargetKey::new(S::KIND, identifier),
            target_type: PhantomData,
        }
    }

    pub fn target_key(&self) -> &TargetKey {
        &self.key
    }

    pub fn into_target_key(self) -> TargetKey {
        self.key
    }
}

/// Declares that an action may be checked on a target type.
///
/// Applications opt valid pairs in with ordinary trait implementations. A
/// pair without an implementation cannot satisfy the bound used by typed
/// authorization calls:
///
/// ```compile_fail
/// use rust_but_verify::{Action, ActionId, ActionOn, Target, TargetType};
///
/// struct Edit;
/// impl Action for Edit {
///     fn action_id(&self) -> ActionId {
///         ActionId::new("scheduling.edit")
///     }
/// }
///
/// struct Document;
/// impl TargetType for Document {
///     const KIND: &'static str = "document";
/// }
///
/// fn accepts_pair<A, S>(_action: A, _target: Target<S>)
/// where
///     A: ActionOn<S>,
///     S: TargetType,
/// {}
///
/// // No `impl ActionOn<Document> for Edit` exists.
/// accepts_pair(Edit, Document::ident("789"));
/// ```
pub trait ActionOn<S>: crate::Action
where
    S: TargetType,
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

    impl TargetType for Child {
        const KIND: &'static str = "child";
    }

    struct Schedule;

    impl TargetType for Schedule {
        const KIND: &'static str = "schedule";
    }

    impl ActionOn<Child> for Edit {}
    impl ActionOn<Schedule> for Edit {}

    fn erase<A, S>(action: A, target: Target<S>) -> (ActionId, TargetKey)
    where
        A: ActionOn<S>,
        S: TargetType,
    {
        (action.action_id(), target.into_target_key())
    }

    #[test]
    fn declared_pair_erases_to_runtime_keys() {
        let (action, target) = erase(Edit, Child::ident(123_u64));

        assert_eq!(action, ActionId::new("scheduling.edit"));
        assert_eq!(target, TargetKey::new("child", "123"));
    }

    #[test]
    fn one_action_supports_multiple_target_types() {
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
