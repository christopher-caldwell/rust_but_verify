# RustButVerify vNext ergonomic API

## Outcome

Refine the exploratory v0 API from observed consumer friction while preserving
the library's narrow authorization model: runtime roles and grants, exact
target matching, and deny by default.

## Runtime identities

An action identifier is one opaque, consumer-owned string. The authorization
engine compares it for equality and does not parse or impose a namespace/name
taxonomy. Examples include `"article.edit"`, `"read_article"`, and a
consumer-defined stable identifier.

An authorization target remains collision-safe through two values: a target
kind and an opaque identifier. `Article::ident(1)` and `Wiki::ident(1)` must
remain distinct, even though their identifiers are equal. Target identifiers
accept ordinary consumer values such as numbers, slugs, UUIDs, and composite
identifiers.

## Typed declarations

Consumers can continue to write ordinary Rust declarations for targets,
actions, and compatible action/target pairs. The library also provides a
declarative exported macro that generates the equivalent routine declarations.

The declaration surface uses terms that state the relationship plainly rather
than library jargon such as `scoped_by`. It supports an action associated with
one target by default and an action explicitly valid for multiple target
types. Generated action types retain descriptive full names such as
`ReadArticle`; consumers are not forced to use ambiguous names such as `Read`.

## Role assignments

Common access construction expresses the application fact that a role applies
to a typed target, without requiring consumers to construct a raw runtime
target key. Dynamic policy and assignment loaders retain a low-level runtime
escape hatch.

## Observable acceptance

- Policy grants accept descriptive typed actions and opaque runtime action IDs.
- A single action can be checked only on declared compatible target types,
  including more than one target type when explicitly declared.
- Equal identifiers in distinct target kinds cannot authorize one another.
- Common role assignment construction requires neither a raw target kind
  string nor a raw target-key type.
- Documentation and examples show equivalent macro and non-macro action/
  target declarations, plus numeric, slug, composite, and UUID identifiers.
- The library continues to do no relationship traversal, domain-object
  validation, policy-language evaluation, persistence, or framework work.
