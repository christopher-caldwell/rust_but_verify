# Knowledge Base tutorial

Run these from `examples/` with `just example knowledge_base 01` (and so on).

1. **Direct article access** introduces one role on one exact Article target.
2. **Article ownership** shows application-verified ownership becoming an access fact.
3. **Wiki-wide access** adds a Wiki target and the exactly-two-target `can_any`
   “article OR wiki” check for an article in that wiki.
4. **Combined access** keeps the same Wiki-wide access establishment from Step 3
   and adds the ownership-derived pattern from Step 2 to the same `Access`.

The typed authorization vocabulary is compile-time structure: target types,
actions, and the target types on which each action is valid must be known when
the application is compiled. The macro generates ordinary Rust types and trait
implementations; it cannot be populated from runtime configuration. `Policy`
and `Access`, by contrast, are runtime values. The application decides how to
load its current role-to-action rules and caller-specific access facts.

Every step uses `bool` as the natural answer to an authorization question.
The application derives ownership and article-to-Wiki facts before passing
typed targets to RustButVerify; the library does not discover relationships.
For more than two candidates, use `Targets<A>` rather than extending the
beginner tuple form.
