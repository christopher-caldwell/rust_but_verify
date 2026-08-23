//! Step 2: the application turns verified article ownership into an access fact.

use rust_but_verify::{Access, Authorizer, Policy, TargetType, define_authorization};

// It's important to note that this CAN be static or dynamic. You are free to load your policy, and create rules however you need.
define_authorization! {
    // For the entity type `article`
    target Article = "article" {
        // Our our personal use case defines one action people can do to an `article`, which is edit.
        // The value here is up to you, however you want to structure
        action EditArticle = "article.edit";
    };
}

/// Application data for articles and users, that you are completely responsible for.
struct ArticleRecord {
    id: u64,
    owner_id: u64,
}

struct ApplicationUser {
    id: u64,
    name: &'static str,
}

/// Create an `Access` value based on your own use case and rules. The access value is later evaluated by the authorizer.
/// 
/// You have the option to build your access at request time, or at start time, depending on your needs. 
/// Request time is better for a larger set of dynamic info. Build time can be used if the set is fixed size, 
/// and can be loaded entirely, all at once.
fn establish_ownership_access(caller: &ApplicationUser, article: &ArticleRecord) -> Access {
    // Begin with no roles. The caller only receives a role if this application-level ownership check succeeds.
    let mut access = Access::empty();

    // If the requestor is the owner, they are given the role `article_owner`. 
    if caller.id == article.owner_id {
        println!(
            "Since {:?} is the owner of article {:?}, we add the role 'article_owner' to their specific access",
            caller.name, article.id,
        );
        access.add_role("article_owner", Article::ident(article.id));
    }
    access
}

fn main() -> Result<(), rust_but_verify::PolicyError> {
    // The policy says what an ownership role is allowed to do. 
    // It does not however, inform which resource the role can do that action on.
    let mut policy_builder = Policy::builder();
    policy_builder.allow("article_owner", EditArticle);
    let authorization_policy = policy_builder.build()?;
    let authorizer = Authorizer::new(authorization_policy);


    // The users of our application
    let bob = ApplicationUser {
        id: 1,
        name: "Bob"
    };
    let alice = ApplicationUser {
        id: 2,
        name: "Alice"
    };

    // These are application records that would normally come from a database.
    // Bob owns Article 1, while Alice owns Article 2.
    let article_1 = ArticleRecord {
        id: 1,
        owner_id: bob.id,
    };
    let article_2 = ArticleRecord {
        id: 2,
        owner_id: alice.id,
    };

    // Establish access separately for each resource.
    let access_for_bob_on_article_1 = establish_ownership_access(&bob, &article_1);

    // Since Bob is NOT the owner of article 2, this will result in empty access, effectively saying 
    // "Bob can do nothing for article 2."
    let access_for_bob_on_article_2 = establish_ownership_access(&bob, &article_2);

    // We take the access created, then ask the authorizer to give a true/false answer.
    // Since Bob IS the owner of article 1, this is true.
    let can_bob_edit_article_1 = authorizer.can(
        &access_for_bob_on_article_1,
        EditArticle,
        Article::ident(article_1.id),
    );
    // Since Bob is NOT the owner of article 2, this is false.
    let can_bob_edit_article_2 = authorizer.can(
        &access_for_bob_on_article_2,
        EditArticle,
        Article::ident(article_2.id),
    );

    println!("Can Bob edit his own article, with ID 1? {can_bob_edit_article_1}");
    println!("Can Bob edit Alice's article, with ID 2? {can_bob_edit_article_2}");

    assert!(can_bob_edit_article_1);
    assert!(!can_bob_edit_article_2);
    Ok(())
}
