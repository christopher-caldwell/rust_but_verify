//! Step 2: the application turns verified article ownership into an access fact.

use rust_but_verify::{Access, Authorizer, Policy, TargetType, define_authorization};

define_authorization! {
    target Article = "article" {
        action EditArticle = "article.edit";
    };
}

struct ArticleRecord {
    id: u64,
    owner: &'static str,
}

/// The application has already loaded `article` and verified its owner. The
/// authorization library receives only the resulting exact access fact.
fn ownership_access(caller: &str, article: &ArticleRecord) -> Access {
    let mut access = Access::empty();
    if caller == article.owner {
        access.add_role("article_owner", Article::ident(article.id));
    }
    access
}

fn main() -> Result<(), rust_but_verify::PolicyError> {
    let mut policy = Policy::builder();
    policy.allow("article_owner", EditArticle);
    let authz = Authorizer::new(policy.build()?);

    let article_991 = ArticleRecord {
        id: 991,
        owner: "Bob",
    };
    let article_992 = ArticleRecord {
        id: 992,
        owner: "Alice",
    };

    let bob_on_991 = ownership_access("Bob", &article_991);
    let bob_on_992 = ownership_access("Bob", &article_992);

    let edits_991 = authz.can(&bob_on_991, EditArticle, Article::ident(article_991.id));
    let edits_992 = authz.can(&bob_on_992, EditArticle, Article::ident(article_992.id));

    println!("Bob owns and edits Article 991: {edits_991}");
    println!("Bob edits Article 992, owned by Alice: {edits_992}");

    assert!(edits_991);
    assert!(!edits_992);
    Ok(())
}
