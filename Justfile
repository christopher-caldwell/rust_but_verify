default: check

# Check that the library and its test targets compile.
check:
    cargo check --all-targets

build:
    cargo build

fmt:
    cargo fmt --all

fmt-check:
    cargo fmt --all -- --check

clippy:
    cargo clippy --all-targets -- -D warnings

test:
    cargo test

# Check the standalone tutorial package.
check-examples:
    cargo check --manifest-path examples/Cargo.toml --bins

doc:
    RUSTDOCFLAGS="-D warnings" cargo doc --no-deps

# Run the focused Criterion timing suite. Pass a Criterion filter, e.g. `just bench can_any`.
bench filter="":
    cargo bench --bench authorization_baseline -- {{filter}}

# Run every timing benchmark plus allocation and concurrent-read baselines.
bench-full:
    cargo bench --bench authorization_baseline
    cargo bench --bench allocation_counts
    cargo bench --bench concurrency

# Verify crate contents and release metadata without requiring a clean worktree.
package-check:
    ./scripts/publish-crate.sh --check-only --allow-dirty

# Validate the public API surface and tutorial binaries from the repository root.
check-public: test check-examples doc

lint: fmt-check clippy

# Prepare, validate, commit, publish, and tag a release from a clean branch.
release:
    ./scripts/publish-crate.sh --release

# Prepare a local release by generating version and changelog updates.
release-prepare:
    ./scripts/publish-crate.sh

# Run publish validation without generating release changes.
publish-dry:
    ./scripts/publish-crate.sh --check-only

# Publish the package from the committed release changes.
publish:
    ./scripts/publish-crate.sh --publish
