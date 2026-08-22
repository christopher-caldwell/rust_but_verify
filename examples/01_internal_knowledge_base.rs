//! Example 1: roles for an internal knowledge base.
//!
//! Read this first. Every role assignment is made at the one shared knowledge
//! base scope, so the only relationship to learn is `User -> Role -> Permission`.

use rust_but_verify::{
    Access, Action, ActionKey, Authorizer, Model, RoleAssignment, Scope, ScopeKey, ScopedBy,
};

#[derive(Clone, Copy)]
struct ReadArticle;
#[derive(Clone, Copy)]
struct CreateArticle;
#[derive(Clone, Copy)]
struct EditArticle;
#[derive(Clone, Copy)]
struct DeleteArticle;

impl Action for ReadArticle {
    fn action_key(&self) -> ActionKey {
        ActionKey::new("article", "read")
    }
}

impl Action for CreateArticle {
    fn action_key(&self) -> ActionKey {
        ActionKey::new("article", "create")
    }
}

impl Action for EditArticle {
    fn action_key(&self) -> ActionKey {
        ActionKey::new("article", "edit")
    }
}

impl Action for DeleteArticle {
    fn action_key(&self) -> ActionKey {
        ActionKey::new("article", "delete")
    }
}

/// An article is a domain object. This small example has one internal knowledge
/// base, so article permissions are checked at that one shared scope.
struct Article {
    title: &'static str,
}

struct KnowledgeBase;

impl Scope for KnowledgeBase {
    const NAMESPACE: &'static str = "knowledge_base";
}

impl ScopedBy<KnowledgeBase> for ReadArticle {}
impl ScopedBy<KnowledgeBase> for CreateArticle {}
impl ScopedBy<KnowledgeBase> for EditArticle {}
impl ScopedBy<KnowledgeBase> for DeleteArticle {}

struct User {
    name: &'static str,
    access: Access,
}

fn report(label: &str, allowed: bool) {
    println!("{label}: {}", if allowed { "ALLOWED" } else { "DENIED" });
}

fn main() -> Result<(), rust_but_verify::ModelError> {
    let mut policy = Model::builder();

    // A role grants permissions. The role names are runtime data, but typed
    // actions keep application call sites readable.
    policy.grant("viewer", ReadArticle);
    policy
        .grant("editor", ReadArticle)
        .grant("editor", CreateArticle)
        .grant("editor", EditArticle);
    policy
        .grant("admin", ReadArticle)
        .grant("admin", CreateArticle)
        .grant("admin", EditArticle)
        .grant("admin", DeleteArticle);
    let authz = Authorizer::new(policy.build()?);

    // Every role applies to this one shared knowledge base. Later examples
    // give role assignments distinct scope identifiers.
    let alice = User {
        name: "Alice",
        access: Access::new([RoleAssignment::new(
            "editor",
            ScopeKey::new("knowledge_base", "internal"),
        )]),
    };
    let bob = User {
        name: "Bob",
        access: Access::new([RoleAssignment::new(
            "viewer",
            ScopeKey::new("knowledge_base", "internal"),
        )]),
    };
    let carol = User {
        name: "Carol",
        access: Access::new([RoleAssignment::new(
            "admin",
            ScopeKey::new("knowledge_base", "internal"),
        )]),
    };
    let handbook = Article {
        title: "engineering handbook",
    };

    // The matching role grant and shared knowledge-base assignment allow it.
    report(
        &format!("{} edits the {}", alice.name, handbook.title),
        authz
            .can(&alice.access, EditArticle, KnowledgeBase::ident("internal"))
            .is_allowed(),
    );
    report(
        &format!("{} reads the {}", bob.name, handbook.title),
        authz
            .can(&bob.access, ReadArticle, KnowledgeBase::ident("internal"))
            .is_allowed(),
    );
    report(
        &format!("{} edits the {}", bob.name, handbook.title),
        authz
            .can(&bob.access, EditArticle, KnowledgeBase::ident("internal"))
            .is_allowed(),
    );
    report(
        &format!("{} deletes the {}", carol.name, handbook.title),
        authz
            .can(
                &carol.access,
                DeleteArticle,
                KnowledgeBase::ident("internal"),
            )
            .is_allowed(),
    );

    // A viewer receives no edit permission even at the matching scope.
    assert!(
        authz
            .can(&alice.access, EditArticle, KnowledgeBase::ident("internal"))
            .is_allowed()
    );
    assert!(
        !authz
            .can(&bob.access, EditArticle, KnowledgeBase::ident("internal"))
            .is_allowed()
    );

    Ok(())
}
