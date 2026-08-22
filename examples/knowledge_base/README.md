# Knowledge Base tutorial

Run these from `examples/` with `just example knowledge_base 01` (and so on).

1. **Direct article access** introduces one role on one exact Article target.
2. **Article ownership** shows application-verified ownership becoming an access fact.
3. **Wiki-wide access** adds a Wiki target and the exactly-two-target `can_any`
   “article OR wiki” check for an article in that wiki.
4. **Combined access** uses both ownership-derived and Wiki-wide access paths.

Every step uses `bool` as the natural answer to an authorization question.
The application derives ownership and article-to-Wiki facts before passing
typed targets to RustButVerify; the library does not discover relationships.
For more than two candidates, use `Targets<A>` rather than extending the
beginner tuple form.
