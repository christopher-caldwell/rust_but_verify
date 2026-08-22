//! Example 1: roles for an internal knowledge base.
//!
//! Read this first. Every role assignment is made at the one shared knowledge
//! base scope, so the only relationship to learn is `User -> Role -> Permission`.

use rust_but_verify::{Access, Authorizer, Policy, TargetType, define_authorization};

define_authorization! {
    target KnowledgeBase = "knowledge_base" {
        action ReadArticle = "article.read";
        action CreateArticle = "article.create";
        action EditArticle = "article.edit";
        action DeleteArticle = "article.delete";
    };
}

/// An article is a domain object. This small example has one internal knowledge
/// base, so article permissions are checked at that one shared scope.
struct Article {
    title: &'static str,
}

struct User {
    name: &'static str,
    access: Access,
}

fn report(label: &str, allowed: bool) {
    println!("{label}: {}", if allowed { "ALLOWED" } else { "DENIED" });
}

fn main() -> Result<(), rust_but_verify::PolicyError> {
    let mut policy = Policy::builder();

    // A role grants permissions. The role names are runtime data, but typed
    // actions keep application call sites readable.
    policy.allow("viewer", ReadArticle);
    policy
        .allow("editor", ReadArticle)
        .allow("editor", CreateArticle)
        .allow("editor", EditArticle);
    policy
        .allow("admin", ReadArticle)
        .allow("admin", CreateArticle)
        .allow("admin", EditArticle)
        .allow("admin", DeleteArticle);
    let authz = Authorizer::new(policy.build()?);

    // Every role applies to this one shared knowledge base. Later examples
    // give role assignments distinct scope identifiers.
    let alice = User {
        name: "Alice",
        access: Access::with_role("editor", KnowledgeBase::ident("internal")),
    };
    let bob = User {
        name: "Bob",
        access: Access::with_role("viewer", KnowledgeBase::ident("internal")),
    };
    let carol = User {
        name: "Carol",
        access: Access::with_role("admin", KnowledgeBase::ident("internal")),
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
    // Requesting permission to a non registered resource also fails
    assert!(
        !authz
            .can(
                &bob.access,
                EditArticle,
                KnowledgeBase::ident("not_registered")
            )
            .is_allowed()
    );
    assert!(
        !authz
            .can(
                &alice.access,
                EditArticle,
                KnowledgeBase::ident("not_registered")
            )
            .is_allowed()
    );

    Ok(())
}
