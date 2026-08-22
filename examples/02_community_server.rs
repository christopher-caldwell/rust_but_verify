//! Example 2: roles assigned to one community.
//!
//! The new idea is that a role does not answer only "what can this user do?"
//! It answers "what can this user do in this particular community?"

use rust_but_verify::{
    Access, Authorizer, Policy, RoleAssignment, TargetType, define_authorization,
};

define_authorization! {
    target Community = "community" {
        action ViewCommunity = "community.view";
        action CreateMessage = "message.create";
        action DeleteMessage = "message.delete";
        action ModerateMember = "member.moderate";
        action ConfigureCommunity = "community.configure";
    };
}

struct User {
    name: &'static str,
    access: Access,
}

fn report(user: &User, operation: &str, community: &str, allowed: bool) {
    println!(
        "{} {operation} in {community}: {}",
        user.name,
        if allowed { "ALLOWED" } else { "DENIED" }
    );
}

fn main() -> Result<(), rust_but_verify::PolicyError> {
    let mut policy = Policy::builder();
    policy
        .allow("member", ViewCommunity)
        .allow("member", CreateMessage)
        .allow("moderator", ViewCommunity)
        .allow("moderator", CreateMessage)
        .allow("moderator", DeleteMessage)
        .allow("moderator", ModerateMember)
        .allow("admin", ViewCommunity)
        .allow("admin", CreateMessage)
        .allow("admin", DeleteMessage)
        .allow("admin", ModerateMember)
        .allow("admin", ConfigureCommunity);
    let authz = Authorizer::new(policy.build()?);

    // The target in each assignment is the community in which the role applies.
    let alice = User {
        name: "Alice",
        access: Access::new([
            RoleAssignment::new("moderator", Community::ident("rust-developers")),
            RoleAssignment::new("member", Community::ident("game-night")),
        ]),
    };
    let bob = User {
        name: "Bob",
        access: Access::with_role("member", Community::ident("rust-developers")),
    };
    let carol = User {
        name: "Carol",
        access: Access::with_role("admin", Community::ident("game-night")),
    };

    // Alice's moderator assignment matches only `community::rust-developers`.
    let alice_moderates_rust = authz
        .can(
            &alice.access,
            ModerateMember,
            Community::ident("rust-developers"),
        )
        .is_allowed();
    report(
        &alice,
        "moderates members",
        "Rust Developers",
        alice_moderates_rust,
    );

    let alice_moderates_games = authz
        .can(
            &alice.access,
            ModerateMember,
            Community::ident("game-night"),
        )
        .is_allowed();
    report(
        &alice,
        "moderates members",
        "Game Night",
        alice_moderates_games,
    );

    // Her separate member assignment still lets Alice participate normally.
    let alice_posts_in_games = authz
        .can(&alice.access, CreateMessage, Community::ident("game-night"))
        .is_allowed();
    report(
        &alice,
        "creates messages",
        "Game Night",
        alice_posts_in_games,
    );

    let bob_reads_rust = authz
        .can(
            &bob.access,
            ViewCommunity,
            Community::ident("rust-developers"),
        )
        .is_allowed();
    report(
        &bob,
        "views the community",
        "Rust Developers",
        bob_reads_rust,
    );

    let carol_configures_games = authz
        .can(
            &carol.access,
            ConfigureCommunity,
            Community::ident("game-night"),
        )
        .is_allowed();
    report(
        &carol,
        "configures the community",
        "Game Night",
        carol_configures_games,
    );

    let carol_configures_rust = authz
        .can(
            &carol.access,
            ConfigureCommunity,
            Community::ident("rust-developers"),
        )
        .is_allowed();
    report(
        &carol,
        "configures the community",
        "Rust Developers",
        carol_configures_rust,
    );

    assert!(
        alice_moderates_rust && alice_posts_in_games && bob_reads_rust && carol_configures_games
    );
    assert!(!alice_moderates_games && !carol_configures_rust);
    Ok(())
}
