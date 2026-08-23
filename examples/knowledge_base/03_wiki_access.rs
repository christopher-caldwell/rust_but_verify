//! Step 3: one Wiki-level role can grant access to each article in that Wiki.

use rust_but_verify::{Access, Authorizer, Policy, TargetType, define_authorization};

define_authorization! {
    // NEW: the authorization model now has two target types. A role may apply
    // directly to an Article, as before, or to the Wiki containing that article.
    target Article = "article";
    target Wiki = "wiki";

    // NEW: list every target type that may authorize this action. This lets one
    // edit request consider both the exact Article and its containing Wiki.
    action EditArticle = "article.edit" for [Article, Wiki];
}

// The application owns this relationship. `wiki_id` records which Wiki contains
// the Article; RustButVerify does not infer or look up that relationship.
struct ArticleRecord {
    id: u64,
    wiki_id: u64,
}

struct ApplicationUser {
    id: u64,
    name: &'static str,
}

struct WikiRecord {
    id: u64,
    owner_id: u64,
}

// This follows Step 2's ownership pattern, but the resulting role is attached
// to a Wiki target instead of an Article target.
fn establish_wiki_editor_access(caller: &ApplicationUser, wiki: &WikiRecord) -> Access {
    let mut access = Access::empty();

    if caller.id == wiki.owner_id {
        println!(
            "Since {} is the owner of Wiki {}, we add the role 'wiki_editor' to their specific access",
            caller.name, wiki.id,
        );
        access.add_role("wiki_editor", Wiki::ident(wiki.id));
    }
    access
}

fn main() -> Result<(), rust_but_verify::PolicyError> {
    let mut policy_builder = Policy::builder();
    policy_builder.allow("wiki_editor", EditArticle);
    let authorization_policy = policy_builder.build()?;
    let authorizer = Authorizer::new(authorization_policy);

    let alice = ApplicationUser {
        id: 1,
        name: "Alice",
    };

    // Alice owns Wiki 1, while another user owns Wiki 2.
    let wiki_1 = WikiRecord {
        id: 1,
        owner_id: alice.id,
    };
    let wiki_2 = WikiRecord { id: 2, owner_id: 2 };

    // Articles 1 and 2 belong to Wiki 1. Article 3 belongs to Wiki 2. These
    // records give the application the parent target to include in each check.
    let article_1 = ArticleRecord {
        id: 1,
        wiki_id: wiki_1.id,
    };
    let article_2 = ArticleRecord {
        id: 2,
        wiki_id: wiki_1.id,
    };
    let article_3 = ArticleRecord {
        id: 3,
        wiki_id: wiki_2.id,
    };

    // Alice receives one role on Wiki 1. She does not need a separate role on
    // every Article in that Wiki.
    let access_for_alice = establish_wiki_editor_access(&alice, &wiki_1);

    // NEW: `can_any` checks more than one valid target for the same action. Here
    // the tuple means "authorize this exact Article OR its containing Wiki."
    // The application supplies both trusted identities from `article_1`.
    //
    // Alice has no role on Article 1, but her Wiki 1 role matches the second
    // candidate, so the result is true.
    let can_alice_edit_article_1 = authorizer.can_any(
        &access_for_alice,
        EditArticle,
        (Article::ident(article_1.id), Wiki::ident(article_1.wiki_id)),
    );
    // Article 2 supplies the same Wiki target. The one Wiki-level role therefore
    // covers another article without adding another access fact.
    let can_alice_edit_article_2 = authorizer.can_any(
        &access_for_alice,
        EditArticle,
        (Article::ident(article_2.id), Wiki::ident(article_2.wiki_id)),
    );
    // Article 3 supplies Wiki 2 instead. Neither candidate matches Alice's role,
    // demonstrating that Wiki-wide access remains scoped to one exact Wiki.
    let can_alice_edit_article_3 = authorizer.can_any(
        &access_for_alice,
        EditArticle,
        (Article::ident(article_3.id), Wiki::ident(article_3.wiki_id)),
    );

    println!("Can Alice edit article 1 in Wiki 1? {can_alice_edit_article_1}");
    println!("Can Alice edit article 2 in Wiki 1? {can_alice_edit_article_2}");
    println!("Can Alice edit article 3 in Wiki 2? {can_alice_edit_article_3}");

    assert!(can_alice_edit_article_1);
    assert!(can_alice_edit_article_2);
    assert!(!can_alice_edit_article_3);
    Ok(())
}
