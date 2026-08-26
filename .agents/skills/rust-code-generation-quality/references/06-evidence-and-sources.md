# Evidence and Sources

This skill intentionally distinguishes *AI authorship* from *low-quality generated code*. Style alone is weak evidence. The rules focus on concrete engineering quality.

## Rust project LLM policy

Rust's project policy emphasizes reviewer burden, contributor understanding, complete testing, and explicitly warns that style is not evidence of LLM use.

- Rust Forge — LLM usage policy: https://forge.rust-lang.org/policies/llm-usage.html

The policy is particularly relevant to this skill's requirements for self-review, understanding design choices, edge-case testing, and heightened scrutiny around safety-critical generated code.

## Rust Clippy

Clippy's lint categories cover correctness, suspicious code, style, complexity, and performance problems. These provide deterministic checks for many patterns that should not be delegated to subjective “AI detection.”

- Clippy repository/documentation: https://github.com/rust-lang/rust-clippy
- Clippy lint index: https://rust-lang.github.io/rust-clippy/master/index.html

Clippy also documents lints relevant to redundant cloning, borrowed containers, mutex/async misuse, and safety comments.

## Rust API Guidelines

The Rust API Guidelines provide concrete conventions for caller-controlled copying/borrowing, conversions, error types, naming, type safety, and public API dependability.

- Checklist: https://rust-lang.github.io/api-guidelines/checklist.html
- Dependability: https://rust-lang.github.io/api-guidelines/dependability.html

## Rust Design Patterns — clone anti-pattern

The community-maintained Rust Design Patterns guide explicitly identifies “Clone to satisfy the borrow checker” as an anti-pattern. The skill's ownership section is deliberately built around this failure mode.

- https://rust-unofficial.github.io/patterns/anti_patterns/borrow_clone.html

## The Rust Book — panic versus Result

The Rust Book recommends `Result` for recoverable errors and discusses appropriate versus inappropriate use of panic.

- https://doc.rust-lang.org/stable/book/ch09-03-to-panic-or-not-to-panic.html

## Rustonomicon

The Rustonomicon documents the invariant-based reasoning required around unsafe Rust. A safety comment is not proof; the surrounding safe API must actually maintain the invariant relied upon by unsafe operations.

- https://doc.rust-lang.org/stable/nomicon/working-with-unsafe.html

## Rust-specific LLM crate hallucination research

A 2026 study evaluating generated Rust snippets across multiple LLM families found hallucinated crate references across every model family evaluated. This supports the skill's strict rule that dependency names, features, versions, and APIs must be verified rather than trusted because they look plausible.

- arXiv HTML: https://arxiv.org/html/2606.08444

Treat the paper's exact aggregate percentage cautiously: the paper text and displayed counts appear not to agree exactly. The robust conclusion is that hallucinated Rust dependency references occurred across all evaluated model families.

## AI assistance and maintainability research

A 2026 controlled study with 151 participants (mostly professional developers) did not find a significant downstream maintainability difference between AI-assisted and non-AI-assisted implementations, while observing tendencies such as more comments in AI-assisted code.

This is why this skill does **not** treat verbosity, comments, syntax, or formatting as proof of bad code or AI authorship.

- Empirical Software Engineering: https://link.springer.com/article/10.1007/s10664-026-10889-1

## Interpretation

These sources support a quality-oriented standard:

- do not attempt to “detect AI” from style;
- use deterministic tooling where possible;
- evaluate ownership, APIs, errors, unsafe invariants, testing, and actual behavior;
- verify external crate/API knowledge rather than trusting generated plausibility;
- treat unexplained compiler-appeasement patterns as design-review triggers, not automatic violations.
