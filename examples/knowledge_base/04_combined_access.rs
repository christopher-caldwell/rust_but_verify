//! Step 4: ownership-derived and Wiki-wide access are independent paths.

use rust_but_verify::{Access, Authorizer, Policy, TargetType, define_authorization};

define_authorization! {
    target Article = "article";
    target Wiki = "wiki";

    action EditArticle = "article.edit" for [Article, Wiki];
}

struct ArticleRecord {
    id: u64,
    wiki_id: u64,
    owner: &'static str,
}

fn may_edit_article(authz: &Authorizer, access: &Access, article: &ArticleRecord) -> bool {
    // The application supplies the article and its already-verified Wiki target.
    authz.can_any(
        access,
        EditArticle,
        (Article::ident(article.id), Wiki::ident(article.wiki_id)),
    )
}

fn main() -> Result<(), rust_but_verify::PolicyError> {
    let mut policy = Policy::builder();
    policy
        .allow("article_owner", EditArticle)
        .allow("wiki_editor", EditArticle);
    let authz = Authorizer::new(policy.build()?);

    let article_700 = ArticleRecord {
        id: 700,
        wiki_id: 7,
        owner: "Alice",
    };
    let article_991 = ArticleRecord {
        id: 991,
        wiki_id: 42,
        owner: "Bob",
    };
    let article_992 = ArticleRecord {
        id: 992,
        wiki_id: 42,
        owner: "Carol",
    };
    let article_500 = ArticleRecord {
        id: 500,
        wiki_id: 7,
        owner: "Dana",
    };

    // The application has verified both facts: Alice owns Article 700 and is
    // a Wiki 42 editor. RustButVerify does not infer either relationship.
    let mut alice = Access::with_role("wiki_editor", Wiki::ident(42));
    if article_700.owner == "Alice" {
        alice.add_role("article_owner", Article::ident(article_700.id));
    }

    let edits_700 = may_edit_article(&authz, &alice, &article_700);
    let edits_991 = may_edit_article(&authz, &alice, &article_991);
    let edits_992 = may_edit_article(&authz, &alice, &article_992);
    let edits_500 = may_edit_article(&authz, &alice, &article_500);

    println!("Alice edits owned Article 700: {edits_700}");
    println!("Alice edits Article 991 in Wiki 42: {edits_991}");
    println!("Alice edits Article 992 in Wiki 42: {edits_992}");
    println!("Alice edits unrelated Article 500: {edits_500}");

    assert!(edits_700 && edits_991 && edits_992);
    assert!(!edits_500);
    Ok(())
}
