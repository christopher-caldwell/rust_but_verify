//! Example 2: roles scoped to one community.
//!
//! The new idea is that a role does not answer only "what can this user do?"
//! It answers "what can this user do in this particular community?"

use rust_but_verify::{
    Access, Action, ActionKey, Authorizer, Model, RoleAssignment, Scope, ScopeKey, ScopedBy,
};

#[derive(Clone, Copy)]
struct ViewCommunity;
#[derive(Clone, Copy)]
struct CreateMessage;
#[derive(Clone, Copy)]
struct DeleteMessage;
#[derive(Clone, Copy)]
struct ModerateMember;
#[derive(Clone, Copy)]
struct ConfigureCommunity;

// Each application action has a stable runtime key. Keeping these declarations
// explicit makes the policy entries below easy to connect to their meaning.
impl Action for ViewCommunity {
    fn action_key(&self) -> ActionKey {
        ActionKey::new("community", "view")
    }
}

impl Action for CreateMessage {
    fn action_key(&self) -> ActionKey {
        ActionKey::new("message", "create")
    }
}

impl Action for DeleteMessage {
    fn action_key(&self) -> ActionKey {
        ActionKey::new("message", "delete")
    }
}

impl Action for ModerateMember {
    fn action_key(&self) -> ActionKey {
        ActionKey::new("member", "moderate")
    }
}

impl Action for ConfigureCommunity {
    fn action_key(&self) -> ActionKey {
        ActionKey::new("community", "configure")
    }
}

struct Community;

impl Scope for Community {
    const NAMESPACE: &'static str = "community";
}

impl ScopedBy<Community> for ViewCommunity {}
impl ScopedBy<Community> for CreateMessage {}
impl ScopedBy<Community> for DeleteMessage {}
impl ScopedBy<Community> for ModerateMember {}
impl ScopedBy<Community> for ConfigureCommunity {}

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

fn main() -> Result<(), rust_but_verify::ModelError> {
    let mut policy = Model::builder();
    policy
        .grant("member", ViewCommunity)
        .grant("member", CreateMessage)
        .grant("moderator", ViewCommunity)
        .grant("moderator", CreateMessage)
        .grant("moderator", DeleteMessage)
        .grant("moderator", ModerateMember)
        .grant("admin", ViewCommunity)
        .grant("admin", CreateMessage)
        .grant("admin", DeleteMessage)
        .grant("admin", ModerateMember)
        .grant("admin", ConfigureCommunity);
    let authz = Authorizer::new(policy.build()?);

    // The scope in each assignment is the community in which the role applies.
    let alice = User {
        name: "Alice",
        access: Access::new([
            RoleAssignment::new("moderator", ScopeKey::new("community", "rust-developers")),
            RoleAssignment::new("member", ScopeKey::new("community", "game-night")),
        ]),
    };
    let bob = User {
        name: "Bob",
        access: Access::new([RoleAssignment::new(
            "member",
            ScopeKey::new("community", "rust-developers"),
        )]),
    };
    let carol = User {
        name: "Carol",
        access: Access::new([RoleAssignment::new(
            "admin",
            ScopeKey::new("community", "game-night"),
        )]),
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
