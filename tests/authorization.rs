use std::sync::Arc;

use rust_but_verify::{
    Access, Action, ActionId, ActionOn, Authorizer, Decision, Policy, RoleAssignment, TargetKey,
    TargetType, Targets,
};

#[derive(Clone, Copy)]
struct Edit;
impl Action for Edit {
    fn action_id(&self) -> ActionId {
        ActionId::new("scheduling.edit")
    }
}
#[derive(Clone, Copy)]
struct Delete;
impl Action for Delete {
    fn action_id(&self) -> ActionId {
        ActionId::new("scheduling.delete")
    }
}
struct Child;
impl TargetType for Child {
    const KIND: &'static str = "child";
}
struct Article;
impl TargetType for Article {
    const KIND: &'static str = "article";
}
struct Wiki;
impl TargetType for Wiki {
    const KIND: &'static str = "wiki";
}
impl ActionOn<Child> for Edit {}
impl ActionOn<Child> for Delete {}
impl ActionOn<Article> for Edit {}
impl ActionOn<Wiki> for Edit {}

fn policy(roles: &[&str]) -> Policy {
    let mut builder = Policy::builder();
    for role in roles {
        builder.allow(*role, Edit);
    }
    builder.build().unwrap()
}

#[test]
fn typed_and_runtime_bool_paths_match_decisions() {
    let authz = Authorizer::new(policy(&["parent"]));
    let access = Access::new([RoleAssignment::new("parent", Child::ident("123"))]);
    let cases = [
        (Edit.action_id(), TargetKey::new("child", "123")),
        (Delete.action_id(), TargetKey::new("child", "123")),
        (Edit.action_id(), TargetKey::new("child", "456")),
    ];

    assert!(authz.can(&access, Edit, Child::ident("123")));
    assert_eq!(
        authz.decide(&access, Edit, Child::ident("123")),
        Decision::Allow {
            via_role: "parent".into(),
            via_target: TargetKey::new("child", "123"),
        }
    );
    for (action, target) in cases {
        assert_eq!(
            authz.can_runtime(&access, &action, &target),
            authz.decide_runtime(&access, &action, &target).is_allowed()
        );
    }
    assert!(!authz.can(&Access::empty(), Edit, Child::ident("123")));
    assert_eq!(
        authz.decide(&Access::empty(), Edit, Child::ident("123")),
        Decision::Deny
    );
}

#[test]
fn multi_target_bool_and_provenance_follow_target_then_assignment_order() {
    let authz = Authorizer::new(policy(&["article_first", "article_second", "wiki"]));
    let access = Access::new([
        RoleAssignment::new("article_first", Article::ident(991)),
        RoleAssignment::new("article_second", Article::ident(991)),
        RoleAssignment::new("wiki", Wiki::ident(42)),
    ]);
    assert!(authz.can_any(&access, Edit, (Article::ident(991), Wiki::ident(42))));
    assert_eq!(
        authz.decide_any(&access, Edit, (Article::ident(991), Wiki::ident(42))),
        Decision::Allow {
            via_role: "article_first".into(),
            via_target: TargetKey::new("article", 991),
        }
    );
    assert_eq!(
        authz.decide_any(&access, Edit, (Wiki::ident(42), Article::ident(991))),
        Decision::Allow {
            via_role: "wiki".into(),
            via_target: TargetKey::new("wiki", 42),
        }
    );
}

#[test]
fn targets_support_empty_single_and_arbitrary_counts_in_insertion_order() {
    let authz = Authorizer::new(policy(&["wiki"]));
    let access = Access::with_role("wiki", Wiki::ident(8));
    let empty = Targets::<Edit>::new();
    assert!(!authz.can_any(&access, Edit, empty));
    assert_eq!(
        authz.decide_any(&access, Edit, Targets::<Edit>::new()),
        Decision::Deny
    );

    let one = Targets::<Edit>::new().add(Wiki::ident(8));
    assert!(authz.can_any(&access, Edit, one));
    let many = Targets::<Edit>::new()
        .add(Article::ident(1))
        .add(Article::ident(2))
        .add(Article::ident(3))
        .add(Article::ident(4))
        .add(Article::ident(5))
        .add(Article::ident(6))
        .add(Article::ident(7))
        .add(Wiki::ident(8));
    assert_eq!(
        authz.decide_any(&access, Edit, many),
        Decision::Allow {
            via_role: "wiki".into(),
            via_target: TargetKey::new("wiki", 8),
        }
    );
}

#[test]
fn multi_target_checks_never_mix_alternating_policy_snapshots() {
    let first = policy(&["role_a"]);
    let second = policy(&["role_b"]);
    let authz = Arc::new(Authorizer::new(first.clone()));
    let access = Arc::new(Access::new([
        RoleAssignment::new("role_b", Article::ident(1)),
        RoleAssignment::new("role_a", Wiki::ident(2)),
    ]));
    std::thread::scope(|scope| {
        let writer = Arc::clone(&authz);
        scope.spawn(move || {
            for index in 0..20_000 {
                writer.replace_policy(if index % 2 == 0 {
                    second.clone()
                } else {
                    first.clone()
                });
            }
        });
        for _ in 0..4 {
            let reader = Arc::clone(&authz);
            let access = Arc::clone(&access);
            scope.spawn(move || {
                for _ in 0..20_000 {
                    assert!(reader.can_any(&access, Edit, (Article::ident(1), Wiki::ident(2))));
                }
            });
        }
    });
}
