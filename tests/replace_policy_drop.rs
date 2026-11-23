//! Regression test: `replace_policy` must free the outgoing policy only after
//! it has released the write lock, so concurrent checks never wait for the
//! destructor.
//!
//! This file is its own test binary because it installs a global allocator.
//! When armed on the writer thread, the allocator pauses at the first free
//! made inside `replace_policy` (the first free of the outgoing policy) and
//! waits for a reader thread to complete one check. If the free happens while
//! the write lock is held, the reader cannot finish and the wait times out.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::sync::atomic::{AtomicBool, Ordering::SeqCst};
use std::thread;
use std::time::{Duration, Instant};

use rust_but_verify::{Access, ActionId, Authorizer, Policy, RoleAssignment, TargetKey};

/// How long the paused free waits for the reader before concluding that the
/// reader is blocked on the lock. A reader that is not blocked finishes in
/// microseconds, so this only bounds how long a failing run takes.
const READER_TIMEOUT: Duration = Duration::from_secs(5);

thread_local! {
    static ARMED: Cell<bool> = const { Cell::new(false) };
}
static FREE_STARTED: AtomicBool = AtomicBool::new(false);
static READER_FINISHED: AtomicBool = AtomicBool::new(false);
static READER_BLOCKED: AtomicBool = AtomicBool::new(false);
static WRITER_DONE: AtomicBool = AtomicBool::new(false);

/// Disarms the allocator and releases the reader when the writer returns or
/// unwinds, so the scoped reader can never wait forever for a paused free.
struct WriterDone;

impl Drop for WriterDone {
    fn drop(&mut self) {
        ARMED.with(|armed| armed.set(false));
        WRITER_DONE.store(true, SeqCst);
    }
}

struct PauseOnFirstFree;

unsafe impl GlobalAlloc for PauseOnFirstFree {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // One-shot: disarm before waiting. Nothing below allocates.
        if ARMED
            .try_with(|armed| armed.replace(false))
            .unwrap_or(false)
        {
            FREE_STARTED.store(true, SeqCst);
            let deadline = Instant::now() + READER_TIMEOUT;
            while !READER_FINISHED.load(SeqCst) {
                if Instant::now() >= deadline {
                    READER_BLOCKED.store(true, SeqCst);
                    break;
                }
                thread::sleep(Duration::from_millis(1));
            }
        }
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static ALLOCATOR: PauseOnFirstFree = PauseOnFirstFree;

fn policy(roles: usize, actions: usize) -> Policy {
    let mut builder = Policy::builder();
    for role in 0..roles {
        for action in 0..actions {
            builder.allow(
                format!("role-{role}"),
                ActionId::new(format!("action-{action}")),
            );
        }
    }
    builder.build().expect("generated policy should be valid")
}

#[test]
fn replace_policy_frees_outgoing_policy_after_releasing_write_lock() {
    let authz = Authorizer::new(policy(100, 5));
    let target = TargetKey::new("document", 1);
    let access = Access::new([RoleAssignment::from_key("role-0", target.clone())]);
    let action = ActionId::new("action-0");
    let incoming = policy(100, 5);

    thread::scope(|scope| {
        scope.spawn(|| {
            while !FREE_STARTED.load(SeqCst) && !WRITER_DONE.load(SeqCst) {
                thread::yield_now();
            }
            // Both the outgoing and incoming policies allow this check.
            assert!(authz.can_runtime(&access, &action, &target));
            READER_FINISHED.store(true, SeqCst);
        });

        let _writer_done = WriterDone;
        ARMED.with(|armed| armed.set(true));
        authz.replace_policy(incoming);
    });

    assert!(
        FREE_STARTED.load(SeqCst),
        "replace_policy should free the outgoing policy before returning"
    );
    assert!(
        !READER_BLOCKED.load(SeqCst),
        "a concurrent check could not complete while the outgoing policy was being freed, \
         so the free happened while the write lock was held"
    );
}
