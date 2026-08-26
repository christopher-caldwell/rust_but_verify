//! Production Vec versus behavior-equivalent target-index comparison.

use std::time::Duration;

use criterion::{BatchSize, BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use rust_but_verify_baseline as baseline;
use rust_but_verify_indexed_experiment as indexed;

const LOOKUP_SIZES: [usize; 23] = [
    1, 2, 3, 4, 5, 10, 25, 50, 75, 100, 125, 150, 175, 200, 250, 500, 1_000, 2_500, 5_000, 10_000,
    100_000, 500_000, 1_000_000,
];
const BUILD_SIZES: [usize; 12] = [
    25, 50, 100, 250, 500, 1_000, 5_000, 10_000, 100_000, 500_000, 1_000_000, 2_500,
];

fn baseline_authz(grant: &str) -> baseline::Authorizer {
    let mut policy = baseline::Policy::builder();
    policy.allow(grant, baseline::ActionId::new("edit"));
    baseline::Authorizer::new(policy.build().expect("valid policy"))
}

fn indexed_authz(grant: &str) -> indexed::Authorizer {
    let mut policy = indexed::Policy::builder();
    policy.allow(grant, indexed::ActionId::new("edit"));
    indexed::Authorizer::new(policy.build().expect("valid policy"))
}

fn baseline_assignment(
    role: impl Into<baseline::RoleId>,
    id: impl ToString,
) -> baseline::RoleAssignment {
    baseline::RoleAssignment::from_key(role, baseline::TargetKey::new("article", id))
}

fn indexed_assignment(
    role: impl Into<indexed::RoleId>,
    id: impl ToString,
) -> indexed::RoleAssignment {
    indexed::RoleAssignment::from_key(role, indexed::TargetKey::new("article", id))
}

fn baseline_unique(size: usize) -> Vec<baseline::RoleAssignment> {
    (0..size)
        .map(|i| baseline_assignment("editor", i))
        .collect()
}

fn indexed_unique(size: usize) -> Vec<indexed::RoleAssignment> {
    (0..size).map(|i| indexed_assignment("editor", i)).collect()
}

fn bench_unique_target_lookup(criterion: &mut Criterion) {
    let baseline_authz = baseline_authz("editor");
    let indexed_authz = indexed_authz("editor");
    let baseline_action = baseline::ActionId::new("edit");
    let indexed_action = indexed::ActionId::new("edit");
    let mut group = criterion.benchmark_group("lookup_unique_targets");
    group
        .sample_size(10)
        .measurement_time(Duration::from_millis(200));
    for size in LOOKUP_SIZES {
        let baseline_access = baseline::Access::new(baseline_unique(size));
        let indexed_access = indexed::Access::new(indexed_unique(size));
        for (label, id) in [
            ("first", "0".to_owned()),
            ("middle", (size / 2).to_string()),
            ("last", (size - 1).to_string()),
            ("missing", "missing".to_owned()),
        ] {
            let baseline_target = baseline::TargetKey::new("article", &id);
            let indexed_target = indexed::TargetKey::new("article", &id);
            group.bench_function(BenchmarkId::new(format!("vec_{label}"), size), |b| {
                b.iter(|| {
                    black_box(baseline_authz.can_runtime(
                        black_box(&baseline_access),
                        black_box(&baseline_action),
                        black_box(&baseline_target),
                    ))
                })
            });
            group.bench_function(BenchmarkId::new(format!("indexed_{label}"), size), |b| {
                b.iter(|| {
                    black_box(indexed_authz.can_runtime(
                        black_box(&indexed_access),
                        black_box(&indexed_action),
                        black_box(&indexed_target),
                    ))
                })
            });
        }
    }
    group.finish();
}

fn baseline_same_target(size: usize) -> Vec<baseline::RoleAssignment> {
    (0..size)
        .map(|i| baseline_assignment(format!("role_{i}"), 991))
        .collect()
}

fn indexed_same_target(size: usize) -> Vec<indexed::RoleAssignment> {
    (0..size)
        .map(|i| indexed_assignment(format!("role_{i}"), 991))
        .collect()
}

fn bench_same_target_many_roles(criterion: &mut Criterion) {
    let baseline_action = baseline::ActionId::new("edit");
    let indexed_action = indexed::ActionId::new("edit");
    let baseline_target = baseline::TargetKey::new("article", 991);
    let indexed_target = indexed::TargetKey::new("article", 991);
    let mut group = criterion.benchmark_group("lookup_same_target_many_roles");
    group
        .sample_size(10)
        .measurement_time(Duration::from_millis(200));
    for size in [25, 100, 250, 1_000, 10_000, 100_000] {
        let baseline_access = baseline::Access::new(baseline_same_target(size));
        let indexed_access = indexed::Access::new(indexed_same_target(size));
        for (label, grant) in [
            ("first_grant", "role_0".to_owned()),
            ("last_grant", format!("role_{}", size - 1)),
            ("all_deny", "absent".to_owned()),
        ] {
            let baseline_authz = baseline_authz(&grant);
            let indexed_authz = indexed_authz(&grant);
            group.bench_function(BenchmarkId::new(format!("vec_{label}"), size), |b| {
                b.iter(|| {
                    black_box(baseline_authz.can_runtime(
                        &baseline_access,
                        &baseline_action,
                        &baseline_target,
                    ))
                })
            });
            group.bench_function(BenchmarkId::new(format!("indexed_{label}"), size), |b| {
                b.iter(|| {
                    black_box(indexed_authz.can_runtime(
                        &indexed_access,
                        &indexed_action,
                        &indexed_target,
                    ))
                })
            });
        }
    }
    group.finish();
}

fn bench_other_shapes(criterion: &mut Criterion) {
    let baseline_action = baseline::ActionId::new("edit");
    let indexed_action = indexed::ActionId::new("edit");
    let mut group = criterion.benchmark_group("lookup_other_shapes");
    group
        .sample_size(10)
        .measurement_time(Duration::from_millis(200));
    for size in [100, 1_000, 10_000, 100_000] {
        // Unique roles and targets, with a deterministic granting role at the last target.
        let baseline_uu =
            baseline::Access::new((0..size).map(|i| baseline_assignment(format!("role_{i}"), i)));
        let indexed_uu =
            indexed::Access::new((0..size).map(|i| indexed_assignment(format!("role_{i}"), i)));
        let baseline_mixed = baseline::Access::new((0..size).map(|i| {
            if i % 7 == 2 {
                baseline_assignment("editor", i)
            } else {
                baseline_assignment(if i % 2 == 0 { "reader" } else { "viewer" }, 50_000 + i)
            }
        }));
        let indexed_mixed = indexed::Access::new((0..size).map(|i| {
            if i % 7 == 2 {
                indexed_assignment("editor", i)
            } else {
                indexed_assignment(if i % 2 == 0 { "reader" } else { "viewer" }, 50_000 + i)
            }
        }));
        let cases = [
            (
                "unique_allow",
                (size - 1).to_string(),
                format!("role_{}", size - 1),
                &baseline_uu,
                &indexed_uu,
            ),
            (
                "unique_deny",
                (size - 1).to_string(),
                "absent".to_owned(),
                &baseline_uu,
                &indexed_uu,
            ),
            (
                "mixed_early",
                "2".to_owned(),
                "editor".to_owned(),
                &baseline_mixed,
                &indexed_mixed,
            ),
            (
                "mixed_late",
                (2 + (size - 3) / 7 * 7).to_string(),
                "editor".to_owned(),
                &baseline_mixed,
                &indexed_mixed,
            ),
            (
                "mixed_target_miss",
                "missing".to_owned(),
                "editor".to_owned(),
                &baseline_mixed,
                &indexed_mixed,
            ),
        ];
        for (label, id, grant, baseline_access, indexed_access) in cases {
            let baseline_authz = baseline_authz(&grant);
            let indexed_authz = indexed_authz(&grant);
            let baseline_target = baseline::TargetKey::new("article", &id);
            let indexed_target = indexed::TargetKey::new("article", &id);
            group.bench_function(BenchmarkId::new(format!("vec_{label}"), size), |b| {
                b.iter(|| {
                    black_box(baseline_authz.can_runtime(
                        baseline_access,
                        &baseline_action,
                        &baseline_target,
                    ))
                })
            });
            group.bench_function(BenchmarkId::new(format!("indexed_{label}"), size), |b| {
                b.iter(|| {
                    black_box(indexed_authz.can_runtime(
                        indexed_access,
                        &indexed_action,
                        &indexed_target,
                    ))
                })
            });
        }
    }
    group.finish();
}

fn bench_build_and_clone(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("build_from_precreated_assignments");
    group
        .sample_size(10)
        .measurement_time(Duration::from_millis(200));
    for size in BUILD_SIZES {
        let baseline_items = baseline_unique(size);
        let indexed_items = indexed_unique(size);
        group.bench_function(BenchmarkId::new("vec_new", size), |b| {
            b.iter(|| black_box(baseline::Access::new(baseline_items.iter().cloned())))
        });
        group.bench_function(BenchmarkId::new("indexed_new", size), |b| {
            b.iter(|| black_box(indexed::Access::new(indexed_items.iter().cloned())))
        });
    }
    group.finish();

    let mut complete = criterion.benchmark_group("build_assignments_plus_access");
    complete
        .sample_size(10)
        .measurement_time(Duration::from_millis(200));
    for size in BUILD_SIZES {
        complete.bench_function(BenchmarkId::new("vec", size), |b| {
            b.iter(|| {
                black_box(baseline::Access::new(
                    (0..size).map(|i| baseline_assignment("editor", i)),
                ))
            })
        });
        complete.bench_function(BenchmarkId::new("indexed", size), |b| {
            b.iter(|| {
                black_box(indexed::Access::new(
                    (0..size).map(|i| indexed_assignment("editor", i)),
                ))
            })
        });
    }
    complete.finish();

    let mut incremental = criterion.benchmark_group("build_incremental_add_role");
    incremental
        .sample_size(10)
        .measurement_time(Duration::from_millis(200));
    for size in [25, 100, 1_000, 10_000, 100_000, 500_000, 1_000_000] {
        incremental.bench_function(BenchmarkId::new("vec", size), |b| {
            b.iter(|| {
                let mut access = baseline::Access::empty();
                for i in 0..size {
                    access.push(baseline_assignment("editor", i));
                }
                black_box(access)
            })
        });
        incremental.bench_function(BenchmarkId::new("indexed", size), |b| {
            b.iter(|| {
                let mut access = indexed::Access::empty();
                for i in 0..size {
                    access.push(indexed_assignment("editor", i));
                }
                black_box(access)
            })
        });
    }
    incremental.finish();

    let mut clone_group = criterion.benchmark_group("clone_access");
    clone_group
        .sample_size(10)
        .measurement_time(Duration::from_millis(200));
    for size in BUILD_SIZES {
        let baseline_access = baseline::Access::new(baseline_unique(size));
        let indexed_access = indexed::Access::new(indexed_unique(size));
        clone_group.bench_function(BenchmarkId::new("vec", size), |b| {
            b.iter(|| black_box(baseline_access.clone()))
        });
        clone_group.bench_function(BenchmarkId::new("indexed", size), |b| {
            b.iter(|| black_box(indexed_access.clone()))
        });
    }
    clone_group.finish();
}

fn bench_end_to_end(criterion: &mut Criterion) {
    let baseline_authz = baseline_authz("editor");
    let indexed_authz = indexed_authz("editor");
    let baseline_action = baseline::ActionId::new("edit");
    let indexed_action = indexed::ActionId::new("edit");
    let mut group = criterion.benchmark_group("end_to_end_build_plus_checks");
    group
        .sample_size(10)
        .measurement_time(Duration::from_millis(200));
    for size in [
        25, 100, 250, 1_000, 5_000, 10_000, 100_000, 500_000, 1_000_000,
    ] {
        let baseline_items = baseline_unique(size);
        let indexed_items = indexed_unique(size);
        for checks in [
            1, 2, 5, 10, 15, 18, 20, 22, 25, 30, 35, 40, 50, 60, 75, 100, 250, 500, 1_000,
        ] {
            for (shape, id) in [
                ("index_favorable_last", size - 1),
                ("vec_friendly_first", 0),
            ] {
                let baseline_target = baseline::TargetKey::new("article", id);
                let indexed_target = indexed::TargetKey::new("article", id);
                group.bench_function(
                    BenchmarkId::new(format!("vec_{shape}_{checks}"), size),
                    |b| {
                        b.iter_batched(
                            || baseline_items.clone(),
                            |items| {
                                let access = baseline::Access::new(items);
                                for _ in 0..checks {
                                    black_box(baseline_authz.can_runtime(
                                        &access,
                                        &baseline_action,
                                        &baseline_target,
                                    ));
                                }
                            },
                            BatchSize::PerIteration,
                        )
                    },
                );
                group.bench_function(
                    BenchmarkId::new(format!("indexed_{shape}_{checks}"), size),
                    |b| {
                        b.iter_batched(
                            || indexed_items.clone(),
                            |items| {
                                let access = indexed::Access::new(items);
                                for _ in 0..checks {
                                    black_box(indexed_authz.can_runtime(
                                        &access,
                                        &indexed_action,
                                        &indexed_target,
                                    ));
                                }
                            },
                            BatchSize::PerIteration,
                        )
                    },
                );
            }
        }
    }
    group.finish();
}

mod multi {
    use super::{baseline, indexed};
    baseline::define_authorization! {
        target BaselineArticle = "article";
        action BaselineEdit = "edit" for [BaselineArticle];
    }
    indexed::define_authorization! {
        target IndexedArticle = "article";
        action IndexedEdit = "edit" for [IndexedArticle];
    }
}

fn bench_can_any(criterion: &mut Criterion) {
    use baseline::TargetType as _;
    use indexed::TargetType as _;
    let baseline_authz = baseline_authz("editor");
    let indexed_authz = indexed_authz("editor");
    let mut group = criterion.benchmark_group("can_any_representation");
    for size in [100, 1_000, 10_000] {
        let baseline_access = baseline::Access::new(baseline_unique(size));
        let indexed_access = indexed::Access::new(indexed_unique(size));
        for count in [4, 16, 64] {
            for (label, allow_at) in [
                ("first_allows", Some(0)),
                ("last_allows", Some(count - 1)),
                ("all_deny", None),
            ] {
                group.bench_function(
                    BenchmarkId::new(format!("vec_{count}_{label}"), size),
                    |b| {
                        b.iter(|| {
                            let targets =
                                (0..count).fold(baseline::Targets::new(), |targets, i| {
                                    targets.add(multi::BaselineArticle::ident(
                                        if allow_at == Some(i) {
                                            size - 1
                                        } else {
                                            size + i
                                        },
                                    ))
                                });
                            black_box(baseline_authz.can_any(
                                &baseline_access,
                                multi::BaselineEdit,
                                targets,
                            ))
                        })
                    },
                );
                group.bench_function(
                    BenchmarkId::new(format!("indexed_{count}_{label}"), size),
                    |b| {
                        b.iter(|| {
                            let targets = (0..count).fold(indexed::Targets::new(), |targets, i| {
                                targets.add(multi::IndexedArticle::ident(if allow_at == Some(i) {
                                    size - 1
                                } else {
                                    size + i
                                }))
                            });
                            black_box(indexed_authz.can_any(
                                &indexed_access,
                                multi::IndexedEdit,
                                targets,
                            ))
                        })
                    },
                );
            }
        }
        group.bench_function(BenchmarkId::new("vec_tuple_two_all_deny", size), |b| {
            b.iter(|| {
                black_box(baseline_authz.can_any(
                    &baseline_access,
                    multi::BaselineEdit,
                    (
                        multi::BaselineArticle::ident(size),
                        multi::BaselineArticle::ident(size + 1),
                    ),
                ))
            })
        });
        group.bench_function(BenchmarkId::new("indexed_tuple_two_all_deny", size), |b| {
            b.iter(|| {
                black_box(indexed_authz.can_any(
                    &indexed_access,
                    multi::IndexedEdit,
                    (
                        multi::IndexedArticle::ident(size),
                        multi::IndexedArticle::ident(size + 1),
                    ),
                ))
            })
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_unique_target_lookup,
    bench_same_target_many_roles,
    bench_other_shapes,
    bench_build_and_clone,
    bench_end_to_end,
    bench_can_any,
);
criterion_main!(benches);
