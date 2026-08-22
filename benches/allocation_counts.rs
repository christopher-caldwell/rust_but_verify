//! Allocation accounting for one public operation at a time.
//!
//! This is intentionally a separate release-mode executable rather than a
//! Criterion group: allocator instrumentation changes timing substantially.

use std::alloc::System;

use rust_but_verify::{
    Access, Action, ActionId, ActionOn, Authorizer, Decision, Policy, RoleAssignment, RoleId,
    TargetKey, TargetType, define_authorization,
};
use stats_alloc::{INSTRUMENTED_SYSTEM, Region, StatsAlloc};

#[global_allocator]
static ALLOCATOR: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

define_authorization! {
    target Article = "article";
    action EditArticle = "article.edit" for [Article];
}

#[derive(Clone, Copy)]
struct NormalAction;
impl Action for NormalAction {
    fn action_id(&self) -> ActionId {
        ActionId::new("community.configure")
    }
}
impl ActionOn<Article> for NormalAction {}

#[derive(Clone, Copy)]
struct ShortAction;
impl Action for ShortAction {
    fn action_id(&self) -> ActionId {
        ActionId::new("edit.art")
    }
}
#[derive(Clone, Copy)]
struct MediumAction;
impl Action for MediumAction {
    fn action_id(&self) -> ActionId {
        ActionId::new("organization.manage_members_and_content_permissions")
    }
}
#[derive(Clone, Copy)]
struct LongAction;
impl Action for LongAction {
    fn action_id(&self) -> ActionId {
        ActionId::new(
            "organization.workspace.project.document.discussion.comment.moderation.configure_permissions.and_a_long_suffix_to_reach_two_hundred_and_fifty_characters_for_measurement_purposes_only_abcdefghijklmnopqrstuvwxyz_0123456789_abcdefghijklmnopqrstuvwxyz_0123456789_abcdefghijklmnopqrstuvwxyz",
        )
    }
}
impl ActionOn<Article> for ShortAction {}
impl ActionOn<Article> for MediumAction {}
impl ActionOn<Article> for LongAction {}

fn authorizer() -> Authorizer {
    let mut policy = Policy::builder();
    policy.allow("editor", EditArticle);
    Authorizer::new(policy.build().expect("valid benchmark policy"))
}
fn access(size: usize, editor_at: Option<usize>) -> Access {
    Access::new((0..size).map(|index| {
        RoleAssignment::new(
            if editor_at == Some(index) {
                "editor"
            } else {
                "reader"
            },
            Article::ident(991),
        )
    }))
}
fn generated_policy(roles: usize, actions: usize) -> Policy {
    let mut builder = Policy::builder();
    for role in 0..roles {
        for action in 0..actions {
            builder.allow(
                format!("role_{role}"),
                ActionId::new(format!("action_{action}")),
            );
        }
    }
    builder.build().expect("valid generated policy")
}
fn report(label: &str, operation: impl FnOnce()) {
    let region = Region::new(&INSTRUMENTED_SYSTEM);
    operation();
    let change = region.change();
    println!(
        "{{\"benchmark\":\"{label}\",\"allocations\":{},\"bytes_allocated\":{},\"deallocations\":{},\"bytes_deallocated\":{}}}",
        change.allocations, change.bytes_allocated, change.deallocations, change.bytes_deallocated
    );
}

fn main() {
    println!("# rust-but-verify allocation counts (one release-mode operation per line)");
    println!(
        "# static_sizes RoleId={} ActionId={} TargetKey={} RoleAssignment={} Access={} Decision={} Policy={} Authorizer={}",
        std::mem::size_of::<RoleId>(),
        std::mem::size_of::<ActionId>(),
        std::mem::size_of::<TargetKey>(),
        std::mem::size_of::<RoleAssignment>(),
        std::mem::size_of::<Access>(),
        std::mem::size_of::<Decision>(),
        std::mem::size_of::<Policy>(),
        std::mem::size_of::<Authorizer>(),
    );
    let authz = authorizer();
    let action = ActionId::new("article.edit");
    let target = Article::ident(991).into_target_key();
    let empty = access(0, None);
    let allow_1 = access(1, Some(0));
    let deny_1 = access(1, None);
    let allow_10 = access(10, Some(9));
    let deny_10 = access(10, None);
    let allow_100 = access(100, Some(99));
    let deny_100 = access(100, None);
    for (label, access) in [
        ("typed_can_deny_access_0", &empty),
        ("typed_can_allow_access_1", &allow_1),
        ("typed_can_deny_access_1", &deny_1),
        ("typed_can_allow_last_access_10", &allow_10),
        ("typed_can_deny_access_10", &deny_10),
        ("typed_can_allow_last_access_100", &allow_100),
        ("typed_can_deny_access_100", &deny_100),
    ] {
        report(label, || {
            std::hint::black_box(authz.can(access, EditArticle, Article::ident(991)));
        });
    }
    for (label, access) in [
        ("runtime_can_deny_access_0", &empty),
        ("runtime_can_allow_access_1", &allow_1),
        ("runtime_can_deny_access_1", &deny_1),
        ("runtime_can_allow_last_access_10", &allow_10),
        ("runtime_can_deny_access_10", &deny_10),
        ("runtime_can_allow_last_access_100", &allow_100),
        ("runtime_can_deny_access_100", &deny_100),
    ] {
        report(label, || {
            std::hint::black_box(authz.can_runtime(access, &action, &target));
        });
    }
    report("target_u64", || {
        std::hint::black_box(Article::ident(991_u64));
    });
    report("target_short_str", || {
        std::hint::black_box(Article::ident("991"));
    });
    report("target_uuid", || {
        std::hint::black_box(Article::ident("550e8400-e29b-41d4-a716-446655440000"));
    });
    report("target_long", || {
        std::hint::black_box(Article::ident(
            "tenant:acme-corporation:organization:engineering:workspace:knowledge-base:collection:internal:article:2026:991:published:revision:12345678",
        ));
    });
    report("target_composite_64", || {
        std::hint::black_box(Article::ident(
            "tenant:acme:workspace:knowledge:article:991:published:revision:01",
        ));
    });
    report("target_composite_128", || {
        std::hint::black_box(Article::ident(
            "tenant:acme-corporation:organization:engineering:workspace:knowledge-base:collection:internal:article:2026:991:published:revision:12345678",
        ));
    });
    report("target_composite_256", || {
        std::hint::black_box(Article::ident(
            "tenant:acme-corporation:organization:engineering:workspace:knowledge-base:collection:internal:article:2026:991:published:revision:12345678:abcdefghijklmnopqrstuvwxyz:abcdefghijklmnopqrstuvwxyz:abcdefghijklmnopqrstuvwxyz:abcdefghijklmnopqrstuvwxyz",
        ));
    });
    report("action_id_normal", || {
        std::hint::black_box(NormalAction.action_id());
    });
    report("action_id_short", || {
        std::hint::black_box(ShortAction.action_id());
    });
    report("action_id_medium", || {
        std::hint::black_box(MediumAction.action_id());
    });
    report("action_id_long", || {
        std::hint::black_box(LongAction.action_id());
    });
    report("decision_allow_construct", || {
        std::hint::black_box(Decision::Allow {
            via_role: RoleId::new("editor"),
            via_target: TargetKey::new("article", 991),
        });
    });
    report("decision_deny_construct", || {
        std::hint::black_box(Decision::Deny);
    });
    for (label, identifier) in [
        ("role_assignment_numeric", "991"),
        ("role_assignment_short", "id"),
        (
            "role_assignment_uuid",
            "550e8400-e29b-41d4-a716-446655440000",
        ),
        (
            "role_assignment_composite_64",
            "tenant:acme:workspace:knowledge:article:991:published:revision:01",
        ),
        (
            "role_assignment_composite_128",
            "tenant:acme-corporation:organization:engineering:workspace:knowledge-base:collection:internal:article:2026:991:published:revision:12345678",
        ),
    ] {
        report(label, || {
            std::hint::black_box(RoleAssignment::new("editor", Article::ident(identifier)));
        });
    }
    for size in [1, 10, 100, 1_000] {
        report(&format!("access_construct_{size}"), || {
            std::hint::black_box(access(size, None));
        });
    }
    for size in [1, 10, 100, 1_000] {
        let prebuilt = access(size, None);
        report(&format!("access_clone_{size}"), || {
            std::hint::black_box(prebuilt.clone());
        });
    }
    for (label, roles, actions) in [
        ("policy_build_1x1", 1, 1),
        ("policy_build_100x5", 100, 5),
        ("policy_build_1000x20", 1_000, 20),
    ] {
        report(label, || {
            std::hint::black_box(generated_policy(roles, actions));
        });
    }
}
