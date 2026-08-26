//! One-shot allocator accounting for representation construction and cloning.

use std::alloc::System;

use rust_but_verify_baseline as baseline;
use rust_but_verify_indexed_experiment as indexed;
use stats_alloc::{INSTRUMENTED_SYSTEM, Region, StatsAlloc};

#[global_allocator]
static ALLOCATOR: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

fn baseline_items(size: usize) -> Vec<baseline::RoleAssignment> {
    (0..size)
        .map(|i| {
            baseline::RoleAssignment::from_key("editor", baseline::TargetKey::new("article", i))
        })
        .collect()
}

fn indexed_items(size: usize) -> Vec<indexed::RoleAssignment> {
    (0..size)
        .map(|i| indexed::RoleAssignment::from_key("editor", indexed::TargetKey::new("article", i)))
        .collect()
}

fn report<T>(representation: &str, operation: &str, size: usize, run: impl FnOnce() -> T) {
    let region = Region::new(&INSTRUMENTED_SYSTEM);
    let retained = run();
    std::hint::black_box(&retained);
    let change = region.change();
    println!(
        "{{\"representation\":\"{representation}\",\"operation\":\"{operation}\",\"size\":{size},\"allocations\":{},\"bytes_allocated\":{},\"deallocations\":{},\"bytes_deallocated\":{},\"net_bytes\":{}}}",
        change.allocations,
        change.bytes_allocated,
        change.deallocations,
        change.bytes_deallocated,
        change
            .bytes_allocated
            .saturating_sub(change.bytes_deallocated),
    );
    drop(retained);
}

fn main() {
    println!("# indexed Access allocation comparison; retained value is alive at measurement");
    println!(
        "# static_size vec={} indexed={}",
        std::mem::size_of::<baseline::Access>(),
        std::mem::size_of::<indexed::Access>()
    );
    for size in [100, 1_000, 10_000, 100_000, 500_000, 1_000_000] {
        report("vec", "assignments_plus_access", size, || {
            baseline::Access::new(baseline_items(size))
        });
        report("indexed", "assignments_plus_access", size, || {
            indexed::Access::new(indexed_items(size))
        });

        let vec_access = baseline::Access::new(baseline_items(size));
        let indexed_access = indexed::Access::new(indexed_items(size));
        report("vec", "clone", size, || vec_access.clone());
        report("indexed", "clone", size, || indexed_access.clone());
    }
}
