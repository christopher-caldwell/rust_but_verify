//! Allocation accounting for one public operation at a time.
//!
//! This is intentionally a separate release-mode executable rather than a
//! Criterion group: allocator instrumentation changes timing substantially.

use std::alloc::System;

use rust_but_verify::{
    Access, ActionId, Authorizer, Decision, Policy, RoleAssignment, RoleId, TargetKey, TargetType,
    define_authorization,
};
use stats_alloc::{INSTRUMENTED_SYSTEM, Region, StatsAlloc};

#[global_allocator]
static ALLOCATOR: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

define_authorization! {
    target Article = "article";
    action EditArticle = "article.edit" for [Article];
}

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
    let allow_1 = access(1, Some(0));
    let deny_1 = access(1, None);
    for (label, access) in [
        ("typed_can_allow_access_1", &allow_1),
        ("typed_can_deny_access_1", &deny_1),
    ] {
        report(label, || {
            std::hint::black_box(authz.can(access, EditArticle, Article::ident(991)));
        });
    }
    for (label, access) in [
        ("typed_decide_allow_access_1", &allow_1),
        ("typed_decide_deny_access_1", &deny_1),
    ] {
        report(label, || {
            std::hint::black_box(authz.decide(access, EditArticle, Article::ident(991)));
        });
    }
    for (label, access) in [
        ("runtime_can_allow_access_1", &allow_1),
        ("runtime_can_deny_access_1", &deny_1),
    ] {
        report(label, || {
            std::hint::black_box(authz.can_runtime(access, &action, &target));
        });
    }
    for (label, access) in [
        ("runtime_decide_allow_access_1", &allow_1),
        ("runtime_decide_deny_access_1", &deny_1),
    ] {
        report(label, || {
            std::hint::black_box(authz.decide_runtime(access, &action, &target));
        });
    }
}
