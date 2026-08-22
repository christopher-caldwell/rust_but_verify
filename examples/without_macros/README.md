# Without macros

These short reference examples use the same authorization API as the Knowledge
Base tutorial, but write `TargetType`, `Action`, and `ActionOn` implementations
by hand. `define_authorization!` is convenience syntax, not a separate system.

`can` and `can_any` return `bool`; use `decide` and `decide_any` only when the
first successful role/target provenance is useful. The multiple-target example
uses the exactly-two-target tuple form. For arbitrary numbers of candidates,
build `Targets::<ActionType>::new()` and add only action-compatible targets.
