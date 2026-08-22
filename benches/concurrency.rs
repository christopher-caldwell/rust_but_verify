//! Fixed-work concurrent throughput baseline for `std::sync::RwLock` reads.
//!
//! Worker creation and fixture construction complete before each timed region.
//! The reported interval includes only the barrier release, authorization work,
//! and joining workers; it is therefore a throughput baseline, not a latency
//! percentile benchmark.

use std::sync::{Arc, Barrier};
use std::thread;
use std::time::{Duration, Instant};

use rust_but_verify::{
    Access, ActionId, Authorizer, Policy, RoleAssignment, TargetType, define_authorization,
};

define_authorization! {
    target Article = "article";
    action EditArticle = "article.edit" for [Article];
}

fn policy() -> Policy {
    let mut builder = Policy::builder();
    builder.allow("editor", EditArticle);
    builder.build().expect("valid")
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

fn throughput(threads: usize, access: Access, writer_every: Option<usize>) -> (u64, Duration) {
    const OPERATIONS_PER_THREAD: usize = 100_000;
    let authz = Arc::new(Authorizer::new(policy()));
    let access = Arc::new(access);
    let action = Arc::new(ActionId::new("article.edit"));
    let target = Arc::new(Article::ident(991).into_target_key());
    let start = Arc::new(Barrier::new(
        threads + usize::from(writer_every.is_some()) + 1,
    ));
    let mut handles = Vec::new();
    for _ in 0..threads {
        let authz = Arc::clone(&authz);
        let access = Arc::clone(&access);
        let action = Arc::clone(&action);
        let target = Arc::clone(&target);
        let start = Arc::clone(&start);
        handles.push(thread::spawn(move || {
            start.wait();
            let mut allowed = 0_u64;
            for _ in 0..OPERATIONS_PER_THREAD {
                if authz.can_runtime(&access, &action, &target).is_allowed() {
                    allowed += 1;
                }
            }
            std::hint::black_box(allowed);
            OPERATIONS_PER_THREAD as u64
        }));
    }
    let writer = writer_every.map(|every| {
        let authz = Arc::clone(&authz);
        let start = Arc::clone(&start);
        // Construct every replacement before the timed barrier. This isolates
        // reader throughput under write-lock contention from policy building.
        let replacements: Vec<_> = (0..(threads * OPERATIONS_PER_THREAD / every).max(1))
            .map(|_| policy())
            .collect();
        thread::spawn(move || {
            start.wait();
            for replacement in replacements {
                authz.replace_policy(replacement);
            }
        })
    });
    start.wait();
    let began = Instant::now();
    let total = handles
        .into_iter()
        .map(|handle| handle.join().expect("reader thread"))
        .sum();
    if let Some(writer) = writer {
        writer.join().expect("writer thread");
    }
    (total, began.elapsed())
}

fn main() {
    println!("# rust-but-verify concurrent throughput; fixed 100000 operations/read thread");
    println!("threads,scenario,writer_every,operations,seconds,ops_per_second");
    for threads in [1, 2, 4, 8, 16] {
        for (scenario, access) in [
            ("small_immediate_allow", access(1, Some(0))),
            ("access_5_deny", access(5, None)),
            ("access_25_last_allow", access(25, Some(24))),
            ("access_100_last_allow", access(100, Some(99))),
        ] {
            let (operations, elapsed) = throughput(threads, access, None);
            println!(
                "{threads},{scenario},none,{operations},{:.6},{:.0}",
                elapsed.as_secs_f64(),
                operations as f64 / elapsed.as_secs_f64()
            );
        }
    }
    for readers in [4, 8, 16] {
        for every in [10_000, 1_000, 100, 10] {
            let (operations, elapsed) = throughput(readers, access(1, Some(0)), Some(every));
            println!(
                "{readers},small_immediate_allow,{every},{operations},{:.6},{:.0}",
                elapsed.as_secs_f64(),
                operations as f64 / elapsed.as_secs_f64()
            );
        }
    }
}
