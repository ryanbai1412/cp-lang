# stdlib

Segment: D. Taste and co-design. Owns: `spec/std` (-> `std/`). Worklog: [worklog.md](worklog.md).

## Covers
Every std module, written in CP Lang (no compiler special cases), its docs and doc-tests.

## Improve
Write the remaining `todo` bodies; break import cycles; move the 77 compiler builtins that are library-shaped into std; build the trait-has-every-method generator; move `spec/std` -> `std/` (with repo).

## Success criteria
0 `todo` bodies (`spec-stub.py`); every doc example runs as a test; trait generator green; builtins count shrinking to an agreed floor.

## Ryan's role
Co-designer: API shape, naming and which facilities belong in std, decided on real ports.
