//! Reproduces whether `can_any` can observe two policy snapshots in one call.

use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU64, Ordering},
};
use std::thread;
use std::time::{Duration, Instant};

use rust_but_verify::{
    Access, Authorizer, Policy, RoleAssignment, TargetType, define_authorization,
};

define_authorization! {
    target Article = "article";
    target Wiki = "wiki";
    action Edit = "article.edit" for [Article, Wiki];
}

fn policy(granted_role: &str) -> Policy {
    let mut builder = Policy::builder();
    builder.allow(granted_role, Edit);
    builder.build().expect("valid policy")
}

fn run(readers: usize, duration: Duration) -> (u64, u64, Duration) {
    let policy_a = policy("role_a");
    let policy_b = policy("role_b");
    let authz = Arc::new(Authorizer::new(policy_a.clone()));
    let access = Arc::new(Access::new([
        RoleAssignment::new("role_b", Article::ident(1)),
        RoleAssignment::new("role_a", Wiki::ident(2)),
    ]));
    let stop = Arc::new(AtomicBool::new(false));
    let allows = Arc::new(AtomicU64::new(0));
    let denies = Arc::new(AtomicU64::new(0));
    let mut workers = Vec::with_capacity(readers);
    for _ in 0..readers {
        let authz = Arc::clone(&authz);
        let access = Arc::clone(&access);
        let stop = Arc::clone(&stop);
        let allows = Arc::clone(&allows);
        let denies = Arc::clone(&denies);
        workers.push(thread::spawn(move || {
            while !stop.load(Ordering::Relaxed) {
                if authz.can_any(&access, Edit, (Article::ident(1), Wiki::ident(2))) {
                    allows.fetch_add(1, Ordering::Relaxed);
                } else {
                    denies.fetch_add(1, Ordering::Relaxed);
                }
            }
        }));
    }
    let writer_authz = Arc::clone(&authz);
    let writer_stop = Arc::clone(&stop);
    let writer = thread::spawn(move || {
        while !writer_stop.load(Ordering::Relaxed) {
            writer_authz.replace_policy(policy_b.clone());
            writer_authz.replace_policy(policy_a.clone());
        }
    });
    let started = Instant::now();
    thread::sleep(duration);
    stop.store(true, Ordering::Relaxed);
    writer.join().expect("writer");
    for worker in workers {
        worker.join().expect("reader");
    }
    (
        allows.load(Ordering::Relaxed),
        denies.load(Ordering::Relaxed),
        started.elapsed(),
    )
}

fn main() {
    println!("readers,total_calls,allow_count,deny_count,impossible_deny_rate,duration_seconds");
    for readers in [1, 4, 8] {
        let (allow, deny, elapsed) = run(readers, Duration::from_secs(3));
        let total = allow + deny;
        println!(
            "{readers},{total},{allow},{deny},{:.8},{:.3}",
            deny as f64 / total as f64,
            elapsed.as_secs_f64()
        );
    }
}
