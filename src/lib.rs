#![doc = include_str!("../README.md")]

mod authorizer;
mod identifiers;
mod model;
mod typed;

pub use authorizer::{Access, Authorizer, Decision};
pub use identifiers::{Action, ActionId, RoleAssignment, RoleId, TargetKey};
pub use model::{Model, ModelBuilder, ModelError};
pub use typed::{Scope, ScopedBy, Target};

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
/// use rust_but_verify::{define_authorization, Scope, ScopedBy, Target};
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
///     A: ScopedBy<S>,
///     S: Scope,
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

            impl $crate::Scope for $target {
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

                    impl $crate::ScopedBy<$target> for $action {}
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
                impl $crate::ScopedBy<$shared_target> for $shared_action {}
            )+
        )*
    };
}
