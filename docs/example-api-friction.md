# API friction found while building the examples

These examples use the current public API without adding a relationship or
policy layer around it. The resulting friction is useful evidence for the next
iteration, not a request to redesign v0 before that evidence is reviewed.

- **Relationships are application-side facts.** `Authorizer::can` compares one
  supplied assignment to one exact target. Applications must verify and flatten
  `User -> Team -> Repository`, `Environment -> Project`, and pull-request
  authorship into effective `RoleAssignment` values before calling it. This is
  consistent with the documented boundary, but callers need careful, visible
  code to avoid accidentally treating an unverified relationship as permission.
- **Broad and leaf-level facts scale differently.** Wiki-wide authority never
  materializes every Article. After verifying ownership for one request, the
  application can supply that exact leaf assignment for that request only.
- **There is no native group, parent, or ownership representation.** A team
  permission, a repository-parent check, and an author-only permission all look like
  direct role assignments to the authorizer. The source-control example shows
  the minimum application glue required to retain that distinction.
- **Action and target declarations scale linearly.** Each action needs a small
  `Action` implementation and every valid action/target pair needs an explicit
  `ActionOn` implementation. The repetition is easy to understand, but the
  kitchen-sink example makes the declaration volume noticeable.
- **Target identifiers are application conventions.** Composite IDs such as
  `acme:payments-api:production` are straightforward, but construction is
  repeated and unvalidated by the library. Small domain wrappers help, while
  preserving the library's opaque-identifier boundary.
- **An allow decision identifies a role and target.** With overlapping permissions, the
  first matching assignment determines `Decision::Allow { via_role, via_target }`.
  That is enough for a yes/no check, but not a complete explanation of whether access
  came from a team, direct permission, or another effective-assignment source.
