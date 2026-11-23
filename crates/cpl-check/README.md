# cpl-check

The incremental front end: `Database` behind `cpl check`, `run`, `build` and `lsp`. Name resolution, types, domains, lends, bound and overflow facts, hover data.

## Invariants

- `Database::analyze` redoes only the work an edit invalidated (see `db.rs`).
- The std modules are bundled at build time from `std/` (`build.rs`); a program gets the prelude, a library file does not (spec 11.3).
- Names the checker special-cases are counted by `debt-ratchet.py` (`prelude_privilege`) and may only go down.

## Tests

`src/tests.rs` and `src/tests/` (checks, cost, domains), `src/db/tests.rs`, `tests/std_acyclic.rs`; spec cases in `spec/conformance/`.

Where it sits among the crates: `plan/ARCHITECTURE.md`.
