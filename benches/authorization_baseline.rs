//! Criterion timing baselines for the public RustButVerify API.
//!
//! Lookup groups deliberately keep `Policy`, `Authorizer`, `Access`, and (for
//! runtime calls) erased identifiers outside the timed closure. Typed groups
//! intentionally create the action and target in the closure because that is
//! the public `can` call consumers make.

use criterion::{
    BatchSize, BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main,
};
use rust_but_verify::{
    Access, Action, ActionId, ActionOn, Authorizer, Policy, RoleAssignment, TargetKey, TargetType,
    Targets, define_authorization,
};

define_authorization! {
    target Article = "article";
    target Wiki = "wiki";
    target Section = "section";
    target Organization = "organization";

    action EditArticle = "article.edit" for [Article, Wiki, Section, Organization];
}

const ACCESS_SIZES: [usize; 14] = [
    0, 1, 2, 5, 10, 25, 50, 100, 250, 500, 1_000, 2_500, 5_000, 10_000,
];
const REALISTIC_SIZES: [usize; 3] = [10, 100, 1_000];

#[derive(Clone, Copy)]
struct ShortAction;
impl Action for ShortAction {
    fn action_id(&self) -> ActionId {
        ActionId::new("edit.art")
    }
}
#[derive(Clone, Copy)]
struct NormalAction;
impl Action for NormalAction {
    fn action_id(&self) -> ActionId {
        ActionId::new("community.configure")
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
            "organization.workspace.project.document.discussion.comment.moderation.configure_permissions",
        )
    }
}
impl ActionOn<Article> for ShortAction {}
impl ActionOn<Article> for NormalAction {}
impl ActionOn<Article> for MediumAction {}
impl ActionOn<Article> for LongAction {}
#[derive(Clone, Copy)]
struct VeryLongAction;
impl Action for VeryLongAction {
    fn action_id(&self) -> ActionId {
        ActionId::new(
            "organization.workspace.project.document.discussion.comment.moderation.configure_permissions.and_a_long_suffix_to_reach_two_hundred_and_fifty_characters_for_measurement_purposes_only_abcdefghijklmnopqrstuvwxyz_0123456789_abcdefghijklmnopqrstuvwxyz_0123456789_abcdefghijklmnopqrstuvwxyz",
        )
    }
}
impl ActionOn<Article> for VeryLongAction {}

fn policy_with(grants: &[(&str, &str)]) -> Policy {
    let mut builder = Policy::builder();
    for (role, action) in grants {
        builder.allow(*role, ActionId::new(*action));
    }
    builder.build().expect("benchmark policy is valid")
}
fn authorizer() -> Authorizer {
    Authorizer::new(policy_with(&[("editor", "article.edit")]))
}
fn assignment(role: &str, identifier: impl ToString) -> RoleAssignment {
    RoleAssignment::new(role, Article::ident(identifier))
}
fn access_with_editor_at(size: usize, editor_at: Option<usize>) -> Access {
    Access::new((0..size).map(|index| {
        assignment(
            if editor_at == Some(index) {
                "editor"
            } else {
                "reader"
            },
            991,
        )
    }))
}
fn distinct_positions(size: usize) -> Vec<(&'static str, usize)> {
    let mut positions = vec![
        ("first", 0),
        ("quarter", size / 4),
        ("midpoint", size / 2),
        ("three_quarters", size * 3 / 4),
        ("last", size - 1),
    ];
    positions.dedup_by_key(|(_, index)| *index);
    positions
}

fn bench_can_lookup(criterion: &mut Criterion) {
    let authz = authorizer();
    let action = ActionId::new("article.edit");
    let target = Article::ident(991).into_target_key();
    let mut group = criterion.benchmark_group("can_lookup");
    let empty = Access::empty();
    group.bench_function("typed_empty_deny", |b| {
        b.iter(|| {
            black_box(authz.can(
                black_box(&empty),
                black_box(EditArticle),
                black_box(Article::ident(991)),
            ))
        })
    });
    group.bench_function("runtime_empty_deny", |b| {
        b.iter(|| {
            black_box(authz.can_runtime(black_box(&empty), black_box(&action), black_box(&target)))
        })
    });
    for size in ACCESS_SIZES.into_iter().filter(|size| *size > 0) {
        for (label, index) in distinct_positions(size) {
            let access = access_with_editor_at(size, Some(index));
            group.bench_with_input(
                BenchmarkId::new(format!("typed_allow_{label}"), size),
                &access,
                |b, access| {
                    b.iter(|| {
                        black_box(authz.can(
                            black_box(access),
                            black_box(EditArticle),
                            black_box(Article::ident(991)),
                        ))
                    })
                },
            );
            group.bench_with_input(
                BenchmarkId::new(format!("runtime_allow_{label}"), size),
                &access,
                |b, access| {
                    b.iter(|| {
                        black_box(authz.can_runtime(
                            black_box(access),
                            black_box(&action),
                            black_box(&target),
                        ))
                    })
                },
            );
        }
        let deny = access_with_editor_at(size, None);
        group.bench_with_input(
            BenchmarkId::new("typed_deny_role", size),
            &deny,
            |b, access| {
                b.iter(|| {
                    black_box(authz.can(
                        black_box(access),
                        black_box(EditArticle),
                        black_box(Article::ident(991)),
                    ))
                })
            },
        );
        group.bench_with_input(
            BenchmarkId::new("runtime_deny_role", size),
            &deny,
            |b, access| {
                b.iter(|| {
                    black_box(authz.can_runtime(
                        black_box(access),
                        black_box(&action),
                        black_box(&target),
                    ))
                })
            },
        );
    }
    group.finish();
}

fn bench_deny_shapes(criterion: &mut Criterion) {
    let authz = authorizer();
    let action = ActionId::new("article.edit");
    let target = Article::ident(991).into_target_key();
    let mut group = criterion.benchmark_group("deny_shapes");
    let empty = Access::empty();
    group.bench_function("empty_access", |b| {
        b.iter(|| {
            black_box(authz.can_runtime(black_box(&empty), black_box(&action), black_box(&target)))
        })
    });
    for size in [5, 25, 100, 1_000] {
        let no_target = Access::new((0..size).map(|index| assignment("editor", index + 10_000)));
        let target_no_grant = Access::new((0..size).map(|_| assignment("reader", 991)));
        let repeated_target_no_grant = Access::new(
            (0..size)
                .map(|index| assignment(if index % 2 == 0 { "reader" } else { "viewer" }, 991)),
        );
        for (label, access) in [
            ("no_target_match", no_target),
            ("target_no_grant", target_no_grant),
            ("repeated_target_no_grant", repeated_target_no_grant),
        ] {
            group.bench_with_input(BenchmarkId::new(label, size), &access, |b, access| {
                b.iter(|| {
                    black_box(authz.can_runtime(
                        black_box(access),
                        black_box(&action),
                        black_box(&target),
                    ))
                })
            });
        }
    }
    group.finish();
}

fn bench_access_shapes(criterion: &mut Criterion) {
    let authz = authorizer();
    let action = ActionId::new("article.edit");
    let mixed_policy = Authorizer::new(policy_with(&[
        ("editor", "article.edit"),
        ("wiki_editor", "article.edit"),
        ("organization_member", "article.edit"),
    ]));
    let mut group = criterion.benchmark_group("access_shapes");
    for size in REALISTIC_SIZES {
        let same_role_unique_targets =
            Access::new((0..size).map(|index| assignment("editor", index)));
        let unique_roles_same_target =
            Access::new((0..size).map(|index| assignment(&format!("role_{index}"), 991)));
        let unique_roles_unique_targets =
            Access::new((0..size).map(|index| assignment(&format!("role_{index}"), index)));
        let mixed = Access::new((0..size).map(|index| match index % 7 {
            0 => RoleAssignment::new("organization_member", Organization::ident(1)),
            1 => RoleAssignment::new("wiki_editor", Wiki::ident(42)),
            2 => assignment("editor", 991),
            _ => assignment(
                if index % 2 == 0 { "reader" } else { "viewer" },
                index + 10_000,
            ),
        }));
        for (label, access, local_authz, local_target) in [
            (
                "same_role_unique_targets",
                same_role_unique_targets,
                &authz,
                Article::ident(991).into_target_key(),
            ),
            (
                "unique_roles_same_target",
                unique_roles_same_target,
                &authz,
                Article::ident(991).into_target_key(),
            ),
            (
                "unique_roles_unique_targets",
                unique_roles_unique_targets,
                &authz,
                Article::ident(991).into_target_key(),
            ),
            (
                "realistic_mixed",
                mixed,
                &mixed_policy,
                Article::ident(991).into_target_key(),
            ),
        ] {
            group.bench_with_input(BenchmarkId::new(label, size), &access, |b, access| {
                b.iter(|| {
                    black_box(local_authz.can_runtime(
                        black_box(access),
                        black_box(&action),
                        black_box(&local_target),
                    ))
                })
            });
        }
    }
    group.finish();
}

fn bench_typed_vs_runtime(criterion: &mut Criterion) {
    let authz = authorizer();
    let action = ActionId::new("article.edit");
    let target = Article::ident(991).into_target_key();
    let mut group = criterion.benchmark_group("typed_vs_runtime");
    for (size, allow) in [
        (0, false),
        (1, true),
        (10, true),
        (10, false),
        (100, true),
        (100, false),
        (1_000, true),
        (1_000, false),
    ] {
        let access = access_with_editor_at(size, allow.then_some(size.saturating_sub(1)));
        for runtime in [false, true] {
            let label = if runtime { "runtime" } else { "typed" };
            group.bench_with_input(
                BenchmarkId::new(
                    format!("{label}_{}", if allow { "allow_last" } else { "deny" }),
                    size,
                ),
                &access,
                |b, access| {
                    if runtime {
                        b.iter(|| {
                            black_box(authz.can_runtime(
                                black_box(access),
                                black_box(&action),
                                black_box(&target),
                            ))
                        })
                    } else {
                        b.iter(|| {
                            black_box(authz.can(
                                black_box(access),
                                black_box(EditArticle),
                                black_box(Article::ident(991)),
                            ))
                        })
                    }
                },
            );
        }
    }
    group.finish();
}

fn bench_identifiers(criterion: &mut Criterion) {
    let uuid = "550e8400-e29b-41d4-a716-446655440000";
    let medium = "tenant:acme-corporation:workspace:docs:article:2026:991:published";
    let long = "tenant:acme-corporation:organization:engineering:workspace:knowledge-base:collection:internal:article:2026:991:published:revision:12345678";
    let very_long = "tenant:acme-corporation:organization:engineering:workspace:knowledge-base:collection:internal:article:2026:991:published:revision:12345678:abcdefghijklmnopqrstuvwxyz:abcdefghijklmnopqrstuvwxyz:abcdefghijklmnopqrstuvwxyz:abcdefghijklmnopqrstuvwxyz";
    let mut targets = criterion.benchmark_group("identifiers_target_construction");
    targets.bench_function("u64", |b| {
        b.iter(|| black_box(Article::ident(black_box(991_u64))))
    });
    for (label, value) in [
        ("short_str", "991"),
        ("uuid", uuid),
        ("medium_composite", medium),
        ("long_composite_128", long),
        ("very_long_composite_256", very_long),
    ] {
        targets.bench_function(label, |b| {
            b.iter(|| black_box(Article::ident(black_box(value))))
        });
    }
    targets.finish();
    let mut actions = criterion.benchmark_group("identifiers_action_construction");
    actions.bench_function("short_8", |b| b.iter(|| black_box(ShortAction.action_id())));
    actions.bench_function("normal_20", |b| {
        b.iter(|| black_box(NormalAction.action_id()))
    });
    actions.bench_function("medium_50", |b| {
        b.iter(|| black_box(MediumAction.action_id()))
    });
    actions.bench_function("long_100", |b| b.iter(|| black_box(LongAction.action_id())));
    actions.bench_function("very_long_250", |b| {
        b.iter(|| black_box(VeryLongAction.action_id()))
    });
    actions.finish();
    let mut assignments = criterion.benchmark_group("identifiers_assignment_construction");
    assignments.bench_function("numeric", |b| {
        b.iter(|| black_box(RoleAssignment::new("editor", Article::ident(991_u64))))
    });
    for (label, value) in [
        ("short", "991"),
        ("uuid", uuid),
        ("composite_64", medium),
        ("composite_128", long),
    ] {
        assignments.bench_function(label, |b| {
            b.iter(|| black_box(RoleAssignment::new("editor", Article::ident(value))))
        });
    }
    assignments.finish();
}

fn prepared_assignments(size: usize) -> Vec<RoleAssignment> {
    (0..size).map(|index| assignment("reader", index)).collect()
}
fn bench_access_build(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("access_build");
    for size in ACCESS_SIZES {
        group.throughput(Throughput::Elements(size as u64));
        let assignments = prepared_assignments(size);
        group.bench_with_input(
            BenchmarkId::new("collect_cloned_assignments", size),
            &assignments,
            |b, assignments| {
                b.iter(|| black_box(Access::new(black_box(assignments.iter().cloned()))))
            },
        );
        group.bench_with_input(
            BenchmarkId::new("construct_assignments_and_access", size),
            &size,
            |b, &size| {
                b.iter(|| {
                    black_box(Access::new((0..size).map(|index| {
                        RoleAssignment::new("reader", Article::ident(index))
                    })))
                })
            },
        );
    }
    group.finish();
    let mut incremental = criterion.benchmark_group("access_build_incremental");
    for size in [1, 5, 10, 25, 100, 1_000] {
        incremental.bench_with_input(
            BenchmarkId::new("empty_add_role", size),
            &size,
            |b, &size| {
                b.iter(|| {
                    let mut access = Access::empty();
                    for index in 0..size {
                        access.add_role("reader", Article::ident(index));
                    }
                    black_box(access)
                })
            },
        );
    }
    incremental.finish();
    let mut clones = criterion.benchmark_group("access_clone");
    for size in [1, 10, 100, 1_000] {
        let access = Access::new(prepared_assignments(size));
        clones.bench_with_input(BenchmarkId::new("clone", size), &access, |b, access| {
            b.iter(|| black_box(access.clone()))
        });
    }
    clones.finish();
}

fn bench_decision_cost(criterion: &mut Criterion) {
    let authz = authorizer();
    let action = ActionId::new("article.edit");
    let mut group = criterion.benchmark_group("decision_allow_vs_deny");
    for (label, id) in [
        ("short", "991"),
        ("uuid", "550e8400-e29b-41d4-a716-446655440000"),
        (
            "long",
            "tenant:acme-corporation:organization:engineering:workspace:knowledge-base:collection:internal:article:2026:991:published:revision:12345678",
        ),
    ] {
        let target = Article::ident(id).into_target_key();
        let allow = Access::new([assignment("editor", id)]);
        let deny = Access::new([assignment("reader", id)]);
        for runtime in [false, true] {
            for (outcome, access) in [("allow", &allow), ("deny", &deny)] {
                let path = if runtime { "runtime" } else { "typed" };
                group.bench_with_input(
                    BenchmarkId::new(format!("{path}_can_{outcome}"), label),
                    access,
                    |b, access| {
                        if runtime {
                            b.iter(|| {
                                black_box(authz.can_runtime(
                                    black_box(access),
                                    black_box(&action),
                                    black_box(&target),
                                ))
                            })
                        } else {
                            b.iter(|| {
                                black_box(authz.can(
                                    black_box(access),
                                    black_box(EditArticle),
                                    black_box(Article::ident(id)),
                                ))
                            })
                        }
                    },
                );
                group.bench_with_input(
                    BenchmarkId::new(format!("{path}_decide_{outcome}"), label),
                    access,
                    |b, access| {
                        if runtime {
                            b.iter(|| {
                                black_box(authz.decide_runtime(
                                    black_box(access),
                                    black_box(&action),
                                    black_box(&target),
                                ))
                            })
                        } else {
                            b.iter(|| {
                                black_box(authz.decide(
                                    black_box(access),
                                    black_box(EditArticle),
                                    black_box(Article::ident(id)),
                                ))
                            })
                        }
                    },
                );
            }
        }
    }
    group.finish();
}

fn run_can_any(authz: &Authorizer, access: &Access, candidates: usize) -> bool {
    match candidates {
        2 => authz.can_any(access, EditArticle, (Article::ident(991), Wiki::ident(42))),
        3 => authz.can_any(
            access,
            EditArticle,
            Targets::<EditArticle>::new()
                .add(Article::ident(991))
                .add(Wiki::ident(42))
                .add(Section::ident(8)),
        ),
        4 => authz.can_any(
            access,
            EditArticle,
            Targets::<EditArticle>::new()
                .add(Article::ident(991))
                .add(Wiki::ident(42))
                .add(Section::ident(8))
                .add(Organization::ident(1)),
        ),
        10 => authz.can_any(
            access,
            EditArticle,
            Targets::<EditArticle>::new()
                .add(Article::ident(991))
                .add(Wiki::ident(42))
                .add(Section::ident(8))
                .add(Article::ident(1_001))
                .add(Article::ident(1_002))
                .add(Article::ident(1_003))
                .add(Article::ident(1_004))
                .add(Article::ident(1_005))
                .add(Article::ident(1_006))
                .add(Organization::ident(1)),
        ),
        _ => unreachable!(),
    }
}
fn can_any_access(size: usize, allowing_index: Option<usize>) -> Access {
    if size == 0 {
        return Access::empty();
    }
    let mut assignments = vec![RoleAssignment::new("reader", Article::ident(10_000)); size];
    if let Some(index) = allowing_index {
        let key = match index {
            0 => Article::ident(991).into_target_key(),
            1 => Wiki::ident(42).into_target_key(),
            2 => Section::ident(8).into_target_key(),
            _ => Organization::ident(1).into_target_key(),
        };
        assignments[size - 1] = RoleAssignment::from_key("editor", key);
    }
    Access::new(assignments)
}
fn bench_can_any(criterion: &mut Criterion) {
    let authz = authorizer();
    let mut group = criterion.benchmark_group("can_any_matrix");
    for size in [0, 1, 5, 10, 25, 100, 1_000] {
        for candidates in [2, 3, 4] {
            let positions: Vec<(&str, Option<usize>)> = match candidates {
                2 => vec![
                    ("first_allow", Some(0)),
                    ("last_allow", Some(1)),
                    ("all_deny", None),
                ],
                3 => vec![
                    ("first_allow", Some(0)),
                    ("middle_allow", Some(1)),
                    ("last_allow", Some(2)),
                    ("all_deny", None),
                ],
                _ => vec![
                    ("first_allow", Some(0)),
                    ("middle_allow", Some(1)),
                    ("last_allow", Some(3)),
                    ("all_deny", None),
                ],
            };
            for (position, allowing_index) in positions {
                let access = can_any_access(size, allowing_index);
                group.bench_with_input(
                    BenchmarkId::new(format!("{candidates}_targets_{position}"), size),
                    &access,
                    |b, access| {
                        b.iter(|| black_box(run_can_any(&authz, black_box(access), candidates)))
                    },
                );
            }
        }
    }
    group.finish();
}

fn bench_locked_multi_target(criterion: &mut Criterion) {
    let authz = authorizer();
    let mut group = criterion.benchmark_group("locked_multi_target");
    for (label, candidates, allowing_index) in [
        ("tuple_two_all_deny", 2, None),
        ("tuple_two_second_allow", 2, Some(1)),
        ("targets_four_all_deny", 4, None),
        ("targets_ten_all_deny", 10, None),
    ] {
        let access = can_any_access(1, allowing_index);
        group.bench_function(label, |b| {
            b.iter(|| black_box(run_can_any(&authz, black_box(&access), candidates)))
        });
    }
    group.finish();
}
fn can_runtime_sequence(
    authz: &Authorizer,
    access: &Access,
    action: &ActionId,
    targets: &[TargetKey],
) -> bool {
    for target in targets {
        if authz.can_runtime(access, action, target) {
            return true;
        }
    }
    false
}
fn bench_can_any_overhead(criterion: &mut Criterion) {
    let authz = authorizer();
    let action = ActionId::new("article.edit");
    let mut group = criterion.benchmark_group("can_any_overhead_small_access");
    for size in [0, 1, 2] {
        let targets = [
            Article::ident(991).into_target_key(),
            Wiki::ident(42).into_target_key(),
            Section::ident(8).into_target_key(),
            Organization::ident(1).into_target_key(),
        ];
        let deny = Access::new((0..size).map(|index| assignment("reader", index + 10_000)));
        group.bench_with_input(BenchmarkId::new("single_deny", size), &deny, |b, access| {
            b.iter(|| {
                black_box(authz.can_runtime(
                    black_box(access),
                    black_box(&action),
                    black_box(&targets[0]),
                ))
            })
        });
        for n in [2, 3, 4] {
            group.bench_with_input(
                BenchmarkId::new(format!("can_any_{n}_deny"), size),
                &deny,
                |b, access| b.iter(|| black_box(run_can_any(&authz, black_box(access), n))),
            );
            group.bench_with_input(
                BenchmarkId::new(format!("can_runtime_sequence_{n}_deny"), size),
                &deny,
                |b, access| {
                    b.iter(|| {
                        black_box(can_runtime_sequence(
                            &authz,
                            black_box(access),
                            &action,
                            &targets[..n],
                        ))
                    })
                },
            );
            let allow_last = if size == 0 {
                Access::empty()
            } else {
                Access::new([RoleAssignment::from_key("editor", targets[n - 1].clone())])
            };
            group.bench_with_input(
                BenchmarkId::new(format!("can_any_{n}_last_allow"), size),
                &allow_last,
                |b, access| b.iter(|| black_box(run_can_any(&authz, black_box(access), n))),
            );
            group.bench_with_input(
                BenchmarkId::new(format!("can_runtime_sequence_{n}_last_allow"), size),
                &allow_last,
                |b, access| {
                    b.iter(|| {
                        black_box(can_runtime_sequence(
                            &authz,
                            black_box(access),
                            &action,
                            &targets[..n],
                        ))
                    })
                },
            );
        }
    }
    group.finish();
}

fn policy_for(role_count: usize, actions_per_role: usize) -> Policy {
    let mut builder = Policy::builder();
    for role in 0..role_count {
        for action in 0..actions_per_role {
            builder.allow(
                format!("role_{role}"),
                ActionId::new(format!("action_{action}")),
            );
        }
    }
    builder.build().expect("valid generated policy")
}
fn bench_policy(criterion: &mut Criterion) {
    let cases = [(1, 1), (100, 5), (1_000, 20), (10_000, 20)];
    let target = Article::ident(991).into_target_key();
    let access_grant = Access::new([assignment("role_0", 991)]);
    let access_no_grant = Access::new([assignment("role_0", 991)]);
    let access_absent = Access::new([assignment("absent", 991)]);
    let mut lookup = criterion.benchmark_group("policy_lookup");
    for (roles, actions) in cases {
        let authz = Authorizer::new(policy_for(roles, actions));
        for (label, access, action) in [
            ("grants", &access_grant, ActionId::new("action_0")),
            (
                "role_no_grant",
                &access_no_grant,
                ActionId::new("action_missing"),
            ),
            ("role_absent", &access_absent, ActionId::new("action_0")),
        ] {
            lookup.bench_with_input(
                BenchmarkId::new(format!("{label}_{roles}x{actions}"), roles),
                &access,
                |b, access| {
                    b.iter(|| {
                        black_box(authz.can_runtime(
                            black_box(access),
                            black_box(&action),
                            black_box(&target),
                        ))
                    })
                },
            );
        }
    }
    lookup.finish();
    let mut build = criterion.benchmark_group("policy_build");
    for (roles, actions) in cases {
        build.bench_function(format!("{roles}_roles_{actions}_actions"), |b| {
            b.iter(|| black_box(policy_for(roles, actions)))
        });
    }
    build.finish();
    let mut replacement = criterion.benchmark_group("policy_replacement");
    for (label, roles, actions) in [("small", 1, 1), ("medium", 100, 5), ("large", 1_000, 20)] {
        let authz = Authorizer::new(policy_for(roles, actions));
        let replacement_policy = policy_for(roles, actions);
        replacement.bench_function(label, |b| {
            b.iter_batched(
                || replacement_policy.clone(),
                |policy| authz.replace_policy(black_box(policy)),
                BatchSize::SmallInput,
            )
        });
    }
    replacement.finish();
    let mut validation = criterion.benchmark_group("policy_build_validation");
    for (roles, actions) in [(100, 5), (1_000, 20)] {
        let known: Vec<ActionId> = (0..actions)
            .map(|action| ActionId::new(format!("action_{action}")))
            .collect();
        validation.bench_function(format!("without_validation_{roles}x{actions}"), |b| {
            b.iter(|| black_box(policy_for(roles, actions)))
        });
        validation.bench_function(format!("with_validation_{roles}x{actions}"), |b| {
            b.iter(|| {
                let mut builder = Policy::builder();
                builder.validate_actions(known.clone());
                for role in 0..roles {
                    for action in 0..actions {
                        builder.allow(
                            format!("role_{role}"),
                            ActionId::new(format!("action_{action}")),
                        );
                    }
                }
                black_box(builder.build().expect("valid"))
            })
        });
    }
    validation.finish();
}

fn bench_identifier_sensitivity(criterion: &mut Criterion) {
    let authz = authorizer();
    let mut group = criterion.benchmark_group("authorization_identifier_sensitivity");
    for (label, id) in [
        ("short", "991"),
        ("uuid", "550e8400-e29b-41d4-a716-446655440000"),
        (
            "medium",
            "tenant:acme-corporation:workspace:docs:article:2026:991:published",
        ),
        (
            "long",
            "tenant:acme-corporation:organization:engineering:workspace:knowledge-base:collection:internal:article:2026:991:published:revision:12345678",
        ),
    ] {
        let target = Article::ident(id).into_target_key();
        let action = ActionId::new("article.edit");
        let allow = Access::new([assignment("editor", id)]);
        let deny = Access::new([assignment("reader", id)]);
        for (outcome, access) in [("allow", &allow), ("deny", &deny)] {
            group.bench_with_input(
                BenchmarkId::new(format!("typed_{outcome}"), label),
                access,
                |b, access| {
                    b.iter(|| {
                        black_box(authz.can(
                            black_box(access),
                            black_box(EditArticle),
                            black_box(Article::ident(id)),
                        ))
                    })
                },
            );
            group.bench_with_input(
                BenchmarkId::new(format!("runtime_{outcome}"), label),
                access,
                |b, access| {
                    b.iter(|| {
                        black_box(authz.can_runtime(
                            black_box(access),
                            black_box(&action),
                            black_box(&target),
                        ))
                    })
                },
            );
        }
    }
    group.finish();
    let mut strings = criterion.benchmark_group("authorization_role_action_length");
    for (label, role, action) in [
        ("short", "ed", "do.edit"),
        ("normal", "editor", "community.configure"),
        (
            "long",
            "organization_workspace_document_moderator",
            "organization.workspace.project.document.discussion.comment.moderation.configure_permissions",
        ),
    ] {
        let authz = Authorizer::new(policy_with(&[(role, action)]));
        let runtime_action = ActionId::new(action);
        let access = Access::new([assignment(role, 991)]);
        let target = Article::ident(991).into_target_key();
        strings.bench_function(label, |b| {
            b.iter(|| {
                black_box(authz.can_runtime(
                    black_box(&access),
                    black_box(&runtime_action),
                    black_box(&target),
                ))
            })
        });
    }
    strings.finish();
}

fn realistic_access(size: usize) -> Access {
    Access::new((0..size).map(|index| match index {
        0 => RoleAssignment::new("organization_member", Organization::ident(1)),
        1 => RoleAssignment::new("wiki_editor", Wiki::ident(42)),
        2 => assignment("editor", 991),
        _ => assignment(
            if index % 2 == 0 { "reader" } else { "viewer" },
            index + 10_000,
        ),
    }))
}
fn realistic_authorizer() -> Authorizer {
    Authorizer::new(policy_with(&[
        ("organization_member", "article.edit"),
        ("wiki_editor", "article.edit"),
        ("editor", "article.edit"),
    ]))
}
fn request_batch(authz: &Authorizer, access: &Access, checks: usize) -> usize {
    (0..checks)
        .filter(|index| match index % 4 {
            0 => authz.can(access, EditArticle, Article::ident(991)),
            1 => authz.can(access, EditArticle, Article::ident(999)),
            2 => authz.can_any(access, EditArticle, (Article::ident(999), Wiki::ident(42))),
            _ => authz.can_any(access, EditArticle, (Article::ident(999), Wiki::ident(999))),
        })
        .count()
}
fn bench_realistic(criterion: &mut Criterion) {
    let authz = realistic_authorizer();
    let mut group = criterion.benchmark_group("realistic_workloads");
    let simple = Access::new([assignment("editor", 991)]);
    group.bench_function("simple_article_ownership", |b| {
        b.iter(|| {
            black_box(authz.can(
                black_box(&simple),
                black_box(EditArticle),
                black_box(Article::ident(991)),
            ))
        })
    });
    let normal = realistic_access(8);
    group.bench_function("normal_user_allowed_leaf", |b| {
        b.iter(|| {
            black_box(authz.can(
                black_box(&normal),
                black_box(EditArticle),
                black_box(Article::ident(991)),
            ))
        })
    });
    group.bench_function("normal_user_can_any_wiki_allow", |b| {
        b.iter(|| {
            black_box(authz.can_any(
                black_box(&normal),
                black_box(EditArticle),
                black_box((Article::ident(999), Wiki::ident(42))),
            ))
        })
    });
    for (label, size, checks) in [
        ("moderate_25_10_checks", 25, 10),
        ("heavy_100_100_checks", 100, 100),
    ] {
        let access = realistic_access(size);
        group.bench_function(label, |b| {
            b.iter(|| black_box(request_batch(&authz, black_box(&access), checks)))
        });
    }
    group.finish();
    let mut batches = criterion.benchmark_group("request_batches");
    for (size, counts) in [
        (5, vec![1, 10, 100]),
        (25, vec![1, 10, 100]),
        (100, vec![1, 10, 100, 1_000]),
    ] {
        let access = realistic_access(size);
        for checks in counts {
            batches.bench_with_input(
                BenchmarkId::new(format!("access_{size}"), checks),
                &checks,
                |b, &checks| {
                    b.iter(|| black_box(request_batch(&authz, black_box(&access), checks)))
                },
            );
        }
    }
    batches.finish();
}

criterion_group!(
    benches,
    bench_can_lookup,
    bench_deny_shapes,
    bench_access_shapes,
    bench_typed_vs_runtime,
    bench_identifiers,
    bench_access_build,
    bench_decision_cost,
    bench_can_any,
    bench_locked_multi_target,
    bench_can_any_overhead,
    bench_policy,
    bench_identifier_sensitivity,
    bench_realistic
);
criterion_main!(benches);
