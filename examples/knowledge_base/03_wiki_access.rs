//! Step 3: one Wiki-level role can grant access to each article in that Wiki.

use rust_but_verify::{Access, Authorizer, Policy, TargetType, define_authorization};

define_authorization! {
    target Article = "article";
    target Wiki = "wiki";

    action EditArticle = "article.edit" for [Article, Wiki];
}

struct ArticleRecord {
    id: u64,
    wiki_id: u64,
}

/// The application already knows the article belongs to this Wiki. It supplies
/// both exact targets; RustButVerify does not discover that relationship.
fn may_edit_article(authz: &Authorizer, access: &Access, article: &ArticleRecord) -> bool {
    authz
        .can_any(
            access,
            EditArticle,
            (Article::ident(article.id), Wiki::ident(article.wiki_id)),
        )
        .is_allowed()
}

fn main() -> Result<(), rust_but_verify::PolicyError> {
    let mut policy = Policy::builder();
    policy.allow("wiki_editor", EditArticle);
    let authz = Authorizer::new(policy.build()?);

    let article_991 = ArticleRecord { id: 991, wiki_id: 42 };
    let article_992 = ArticleRecord { id: 992, wiki_id: 42 };
    let unrelated_article = ArticleRecord { id: 500, wiki_id: 7 };
    let alice = Access::with_role("wiki_editor", Wiki::ident(42));

    let edits_991 = may_edit_article(&authz, &alice, &article_991);
    let edits_992 = may_edit_article(&authz, &alice, &article_992);
    let edits_unrelated = may_edit_article(&authz, &alice, &unrelated_article);

    println!("Alice edits Article 991 in Wiki 42: {edits_991}");
    println!("Alice edits Article 992 in Wiki 42: {edits_992}");
    println!("Alice edits Article 500 in Wiki 7: {edits_unrelated}");

    assert!(edits_991 && edits_992);
    assert!(!edits_unrelated);
    Ok(())
}
