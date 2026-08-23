//! Step 4: ownership-derived and Wiki-wide access are independent paths.

use rust_but_verify::{Access, Authorizer, Policy, TargetType, define_authorization};

define_authorization! {
    // The target and action declarations are unchanged from Step 3.
    target Article = "article";
    target Wiki = "wiki";

    action EditArticle = "article.edit" for [Article, Wiki];
}

struct ArticleRecord {
    id: u64,
    wiki_id: u64,
    owner_id: u64,
}

struct ApplicationUser {
    id: u64,
    name: &'static str,
}

/// Add an Article-level role to an existing `Access` value.
///
/// Unlike Step 2, this function mutates access that may already contain other
/// roles. That is how one caller can carry multiple independent access facts.
fn add_article_owner_access(
    access: &mut Access,
    caller: &ApplicationUser,
    article: &ArticleRecord,
) {
    if caller.id == article.owner_id {
        println!(
            "Since {} is the owner of article {}, we add the role 'article_owner' to their specific access",
            caller.name, article.id,
        );
        access.add_role("article_owner", Article::ident(article.id));
    }
}

fn main() -> Result<(), rust_but_verify::PolicyError> {
    // NEW: two different roles allow the same action. Either policy rule may
    // authorize a request when its role matches one of the supplied targets.
    let mut policy_builder = Policy::builder();
    policy_builder
        .allow("article_owner", EditArticle)
        .allow("wiki_editor", EditArticle);
    let authorization_policy = policy_builder.build()?;
    let authorizer = Authorizer::new(authorization_policy);

    let alice = ApplicationUser {
        id: 1,
        name: "Alice",
    };
    let bob = ApplicationUser { id: 2, name: "Bob" };

    // The records deliberately exercise both access paths:
    //
    // - Alice owns Article 1, but it is outside the Wiki she edits.
    // - Bob owns Articles 2 and 3, but they are inside the Wiki Alice edits.
    // - Bob owns Article 4, and it is outside the Wiki Alice edits.
    let article_1 = ArticleRecord {
        id: 1,
        wiki_id: 2,
        owner_id: alice.id,
    };
    let article_2 = ArticleRecord {
        id: 2,
        wiki_id: 1,
        owner_id: bob.id,
    };
    let article_3 = ArticleRecord {
        id: 3,
        wiki_id: 1,
        owner_id: bob.id,
    };
    let article_4 = ArticleRecord {
        id: 4,
        wiki_id: 2,
        owner_id: bob.id,
    };

    // Start with the Wiki-level access introduced in Step 3.
    println!(
        "Since {} is an editor of Wiki 1, we add the role 'wiki_editor' to their specific access",
        alice.name,
    );
    let mut access_for_alice = Access::with_role("wiki_editor", Wiki::ident(1));

    // NEW: add the Article-level ownership fact from Step 2 to the same access
    // value. Alice's access now contains roles on two different target types:
    // `wiki_editor` on Wiki 1 and `article_owner` on Article 1.
    add_article_owner_access(&mut access_for_alice, &alice, &article_1);

    // The `can_any` calls are shaped exactly like Step 3. What changes is that
    // either candidate can now match a different role in the same access value.
    // Article 1 matches `article_owner`; its Wiki 2 candidate does not match.
    let can_alice_edit_article_1 = authorizer.can_any(
        &access_for_alice,
        EditArticle,
        (Article::ident(article_1.id), Wiki::ident(article_1.wiki_id)),
    );
    // Article 2 does not match `article_owner`; its Wiki 1 candidate matches
    // `wiki_editor`. This is a separate successful authorization path.
    let can_alice_edit_article_2 = authorizer.can_any(
        &access_for_alice,
        EditArticle,
        (Article::ident(article_2.id), Wiki::ident(article_2.wiki_id)),
    );
    // Article 3 demonstrates that the Wiki-level path still covers multiple
    // articles even after Article-level access has been added.
    let can_alice_edit_article_3 = authorizer.can_any(
        &access_for_alice,
        EditArticle,
        (Article::ident(article_3.id), Wiki::ident(article_3.wiki_id)),
    );
    // Article 4 matches neither path: Alice has no role on Article 4 or Wiki 2.
    let can_alice_edit_article_4 = authorizer.can_any(
        &access_for_alice,
        EditArticle,
        (Article::ident(article_4.id), Wiki::ident(article_4.wiki_id)),
    );

    println!("Can Alice edit her own article, with ID 1? {can_alice_edit_article_1}");
    println!("Can Alice edit Bob's article, with ID 2, in Wiki 1? {can_alice_edit_article_2}");
    println!("Can Alice edit Bob's article, with ID 3, in Wiki 1? {can_alice_edit_article_3}");
    println!("Can Alice edit Bob's article, with ID 4, in Wiki 2? {can_alice_edit_article_4}");

    assert!(can_alice_edit_article_1);
    assert!(can_alice_edit_article_2);
    assert!(can_alice_edit_article_3);
    assert!(!can_alice_edit_article_4);
    Ok(())
}
