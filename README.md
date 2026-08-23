# Rust But Verify

*get it?*

Small, typed RBAC for exact application resources.

Rust But Verify answers one question:

> Can any of this caller's role assignments perform this action on this target?

It keeps authorization deliberately narrow. Your application owns users, persistence, relationships, and business rules. Rust But Verify evaluates the authorization facts you give it.

> **Status:** early development. The public API is still subject to breaking changes.

## Quick start

```toml
[dependencies]
rust-but-verify = "0.1.0"
```

Define the targets and actions your application understands:

```rust
use rust_but_verify::{
    Access, Authorizer, Policy, TargetType, define_authorization,
};

define_authorization! {
    target Article = "article" {
        action EditArticle = "article.edit";
    };
}
```

Build a policy describing what each role may do:

```rust
let mut policy = Policy::builder();
policy.allow("article_editor", EditArticle);

let authorizer = Authorizer::new(policy.build()?);
```

Supply the caller's access and ask the authorization question:

```rust
let bob = Access::with_role(
    "article_editor",
    Article::ident(1),
);

assert!(authorizer.can(
    &bob,
    EditArticle,
    Article::ident(1),
));

assert!(!authorizer.can(
    &bob,
    EditArticle,
    Article::ident(2),
));
```

Bob is an editor of **Article 1**, not every article.

## The model

Rust But Verify separates three things:


| Type         | Meaning                                                    |
| ------------ | ---------------------------------------------------------- |
| `Policy`     | What actions a role may perform                            |
| `Access`     | Which roles apply to a caller on exact targets             |
| `Authorizer` | Evaluates an action against the policy and supplied access |


A policy might say:

```text
article_editor -> article.edit
```

Access supplies the missing context:

```text
article_editor @ article::1
```

Together they answer:

```text
Can this caller edit article::1? -> true
Can this caller edit article::2? -> false
```

Checks use exact target identity and default to deny.

## Compile-time vocabulary, runtime policy

The typed authorization vocabulary is part of your Rust program and must be known at compile time:

- target types
- action types
- which actions are valid on which target types

`define_authorization!` generates ordinary Rust types and trait implementations for that vocabulary. It cannot be populated dynamically from a database or configuration file.

`Policy` and `Access` are runtime values.

That means your application may load or update:

- which actions a role currently allows
- which role assignments apply to a caller
- request-specific facts such as verified ownership

without recompiling the application.

## Application relationships stay in the application

Suppose Article 1 belongs to Wiki 42 and a caller is an editor of that Wiki.

Your application establishes the `Article -> Wiki` relationship. Rust But Verify does not discover it.

An action can be valid against both target types:

```rust
define_authorization! {
    target Article = "article";
    target Wiki = "wiki";

    action EditArticle = "article.edit" for [Article, Wiki];
}
```

Then check the exact article **or** its already-verified Wiki:

```rust
let allowed = authorizer.can_any(
    &access,
    EditArticle,
    (
        Article::ident(article.id),
        Wiki::ident(article.wiki_id),
    ),
);
```

The two-target tuple is convenience syntax for a common "this or that" check.

For an arbitrary number of relevant targets, use `Targets<A>`:

```rust
use rust_but_verify::Targets;

let targets = Targets::<EditArticle>::new()
    .add(Article::ident(article.id))
    .add(Wiki::ident(article.wiki_id));

let allowed = authorizer.can_any(
    &access,
    EditArticle,
    targets,
);
```

`Targets<A>` preserves insertion order, has no arbitrary target-count limit, and checks every added target for `ActionOn` compatibility at compile time.

## Decisions and provenance

Most callers only need a boolean:

```rust
authorizer.can(&access, EditArticle, Article::ident(1))
```

When you also need the first successful authorization path, use `decide`:

```rust
match authorizer.decide(
    &access,
    EditArticle,
    Article::ident(1),
) {
    rust_but_verify::Decision::Allow {
        via_role,
        via_target,
    } => {
        println!("allowed by {via_role} on {via_target}");
    }
    rust_but_verify::Decision::Deny => {
        println!("denied");
    }
}
```

`Decision` is provenance, not a complete explanation engine. If multiple assignments could authorize an action, it reports the first successful one in evaluation order.

## Runtime policy updates

`Authorizer` does not need to be rebuilt for every policy change.

Build and validate a replacement policy, then install it:

```rust
authorizer.replace_policy(new_policy);
```

Or build and replace in one operation:

```rust
authorizer.try_replace(policy_builder)?;
```

Multi-target checks evaluate against one complete policy snapshot.

## What Rust But Verify does not do

Rust But Verify intentionally does not provide:

- identity or authentication
- persistence
- relationship discovery or traversal
- ownership inference
- wildcard matching
- ABAC predicates
- framework integrations
- a policy language

The consuming application establishes its domain facts. Rust But Verify evaluates the resulting role, action, and target combinations.

## Examples

The `examples/` directory is a standalone Cargo package containing runnable tutorial domains.

The Knowledge Base tutorial builds the model gradually:

1. direct access to one Article
2. application-verified ownership
3. Wiki-wide access
4. multiple independent authorization paths

Run a step with:

```bash
cd examples
just example knowledge_base 01
```

Each file is a complete standalone binary.

### Without macros

The macro is optional.

`examples/without_macros/` demonstrates the same API using ordinary implementations of:

- `TargetType`
- `Action`
- `ActionOn`

```bash
cd examples
just example without_macros 01
```



## Development

Common repository commands are available through `just`:

```bash
just check
just test
just lint
just doc
just bench
```

The benchmark suite and historical measurements live under `benches/` and `benchmark-results.md`.

## Minimum supported Rust version

Rust But Verify requires Rust **1.85** or newer.

## License

Rust But Verify is licensed under the [MIT License](LICENSE).