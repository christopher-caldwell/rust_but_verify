#![doc = include_str!("../README.md")]

mod authorizer;
mod identifiers;
mod model;
mod typed;

pub use authorizer::{Access, Authorizer, CanAnyTargets, Decision};
pub use identifiers::{Action, ActionId, RoleAssignment, RoleId, TargetKey};
pub use model::{Policy, PolicyBuilder, PolicyError};
pub use typed::{ActionOn, Target, TargetType};

/// Declares authorization target types and the actions that can be checked on
/// them.
///
/// Actions inside a target block are valid for that target. Use a top-level
/// action declaration with `for [...]` when the same action can be checked on
/// more than one target type.
///
/// ```
/// use rust_but_verify::define_authorization;
///
/// define_authorization! {
///     target KnowledgeBase = "knowledge_base" {
///         action ReadArticle = "article.read";
///     };
///
///     target Article = "article";
///     target Wiki = "wiki";
///
///     action EditArticle = "article.edit" for [Article, Wiki];
/// }
/// ```
///
/// Generated declarations keep the same compile-time target checks as their
/// ordinary-Rust equivalents:
///
/// ```compile_fail
/// use rust_but_verify::{define_authorization, ActionOn, Target, TargetType};
///
/// define_authorization! {
///     target KnowledgeBase = "knowledge_base" {
///         action ReadArticle = "article.read";
///     };
///     target Wiki = "wiki";
/// }
///
/// fn accepts_pair<A, S>(_action: A, _target: Target<S>)
/// where
///     A: ActionOn<S>,
///     S: TargetType,
/// {}
///
/// // ReadArticle is declared for KnowledgeBase, not Wiki.
/// accepts_pair(ReadArticle, Wiki::ident("handbook"));
/// ```
#[macro_export]
macro_rules! define_authorization {
    (
        $(
            target $target:ident = $kind:literal $(
                {
                    $(
                        action $action:ident = $action_id:literal;
                    )*
                }
            )?;
        )*
        $(
            action $shared_action:ident = $shared_action_id:literal for [$($shared_target:ident),+ $(,)?];
        )*
    ) => {
        $(
            pub struct $target;

            impl $crate::TargetType for $target {
                const KIND: &'static str = $kind;
            }

            $(
                $(
                    #[derive(Clone, Copy)]
                    pub struct $action;

                    impl $crate::Action for $action {
                        fn action_id(&self) -> $crate::ActionId {
                            $crate::ActionId::new($action_id)
                        }
                    }

                    impl $crate::ActionOn<$target> for $action {}
                )*
            )*
        )*

        $(
            #[derive(Clone, Copy)]
            pub struct $shared_action;

            impl $crate::Action for $shared_action {
                fn action_id(&self) -> $crate::ActionId {
                    $crate::ActionId::new($shared_action_id)
                }
            }

            $(
                impl $crate::ActionOn<$shared_target> for $shared_action {}
            )+
        )*
    };
}
