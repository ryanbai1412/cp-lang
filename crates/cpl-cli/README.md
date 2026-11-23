# cpl-cli

The `cpl` binary: wires every subcommand (`check`, `run`, `build`, `test`, `stress`, `fmt`, `lint`, `lsp`, `trace`, ...) over the other crates.

## Invariants

- Allocates with mimalloc on native targets (decision 249).
- Holds wiring, not language logic: a subcommand's behaviour lives in its crate.

## Tests

`tests/` (end-to-end: `cli.rs`, `corpus.rs`, `catalogue.rs`, `differential.rs`, `debugger.rs`, `cases/` and more), `src/unit_tests.rs`.

Where it sits among the crates: `plan/ARCHITECTURE.md`.
