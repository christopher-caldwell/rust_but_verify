//! Opt-in scaling envelope. This is intentionally excluded from routine benchmarks and CI.

use std::time::Duration;

use criterion::{BatchSize, BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use rust_but_verify::{
    Access, ActionId, Authorizer, Policy, RoleAssignment, TargetType, Targets, define_authorization,
};

define_authorization! {
    target Article = "article";
    action EditArticle = "article.edit" for [Article];
}

const LARGE_ACCESS_SIZES: [usize; 7] =
    [10_000, 25_000, 50_000, 100_000, 250_000, 500_000, 1_000_000];
const TARGET_COUNTS: [usize; 8] = [1, 2, 4, 8, 16, 32, 64, 128];

fn authorizer() -> Authorizer {
    let mut builder = Policy::builder();
    builder.allow("editor", EditArticle);
    Authorizer::new(builder.build().expect("valid policy"))
}

fn assignment(role: &str, id: usize) -> RoleAssignment {
    RoleAssignment::new(role, Article::ident(id))
}

fn same_target_access(size: usize, editor_at: Option<usize>) -> Access {
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

fn bench_large_lookup(criterion: &mut Criterion) {
    let authz = authorizer();
    let action = ActionId::new("article.edit");
    let target = Article::ident(991).into_target_key();
    let missing = Article::ident("missing").into_target_key();
    let mut group = criterion.benchmark_group("access_scaling_envelope_lookup");
    group
        .sample_size(10)
        .measurement_time(Duration::from_secs(2));
    for size in LARGE_ACCESS_SIZES {
        for (label, editor_at, lookup_target) in [
            ("first_allow", Some(0), &target),
            ("midpoint_allow", Some(size / 2), &target),
            ("last_allow", Some(size - 1), &target),
            ("full_deny_target_miss", None, &missing),
            ("full_deny_role_miss", None, &target),
        ] {
            let access = same_target_access(size, editor_at);
            group.bench_with_input(BenchmarkId::new(label, size), &access, |b, access| {
                b.iter(|| {
                    black_box(authz.can_runtime(
                        black_box(access),
                        black_box(&action),
                        black_box(lookup_target),
                    ))
                })
            });
        }
    }
    group.finish();
}

fn prepared_assignments(size: usize) -> Vec<RoleAssignment> {
    (0..size).map(|index| assignment("reader", index)).collect()
}

fn bench_large_construction(criterion: &mut Criterion) {
    let mut build = criterion.benchmark_group("access_scaling_envelope_build");
    build
        .sample_size(10)
        .measurement_time(Duration::from_secs(2));
    for size in LARGE_ACCESS_SIZES {
        let assignments = prepared_assignments(size);
        build.bench_with_input(
            BenchmarkId::new("new_precreated", size),
            &assignments,
            |b, assignments| b.iter(|| black_box(Access::new(assignments.iter().cloned()))),
        );
        build.bench_with_input(
            BenchmarkId::new("assignments_plus_access", size),
            &size,
            |b, &size| {
                b.iter(|| {
                    black_box(Access::new(
                        (0..size).map(|index| assignment("reader", index)),
                    ))
                })
            },
        );
        build.bench_with_input(BenchmarkId::new("clone", size), &assignments, |b, items| {
            let access = Access::new(items.iter().cloned());
            b.iter(|| black_box(access.clone()))
        });
        if size <= 250_000 || matches!(size, 500_000 | 1_000_000) {
            build.bench_with_input(
                BenchmarkId::new("incremental_add_role", size),
                &size,
                |b, &size| {
                    b.iter_batched(
                        Access::empty,
                        |mut access| {
                            for index in 0..size {
                                access.add_role("reader", Article::ident(index));
                            }
                            black_box(access)
                        },
                        BatchSize::PerIteration,
                    )
                },
            );
        }
    }
    build.finish();
}

fn candidates(count: usize, allowing_position: Option<usize>) -> Targets<EditArticle> {
    (0..count).fold(Targets::new(), |targets, index| {
        let id = if allowing_position == Some(index) {
            991
        } else {
            10_000 + index
        };
        targets.add(Article::ident(id))
    })
}

fn bench_targets_curve(criterion: &mut Criterion) {
    let authz = authorizer();
    let mut group = criterion.benchmark_group("targets_scaling_envelope");
    for access_size in [1, 100, 1_000] {
        let allow = Access::new(
            std::iter::once(assignment("editor", 991))
                .chain((1..access_size).map(|index| assignment("reader", 20_000 + index))),
        );
        let deny = Access::new((0..access_size).map(|index| assignment("reader", 20_000 + index)));
        for count in TARGET_COUNTS {
            for (label, allowing_position, access) in [
                ("all_deny", None, &deny),
                ("first_target_allows", Some(0), &allow),
                ("last_target_allows", Some(count - 1), &allow),
            ] {
                group.bench_with_input(
                    BenchmarkId::new(format!("access_{access_size}_{label}"), count),
                    &(count, allowing_position, access),
                    |b, &(count, allowing_position, access)| {
                        b.iter(|| {
                            black_box(authz.can_any(
                                black_box(access),
                                EditArticle,
                                black_box(candidates(count, allowing_position)),
                            ))
                        })
                    },
                );
            }
        }
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_large_lookup,
    bench_large_construction,
    bench_targets_curve
);
criterion_main!(benches);
