# cpl-lsp

The language server (`cpl lsp`): diagnostics as you type, hover, go-to-definition, outline, quick fixes, formatting, completion, signature help, inlay hints, references and rename, over stdio.

## Invariants

- Answers from the same `cpl_check::Database` as the command line, so the editor and `cpl check` never disagree.
- Latency budgets are tested (`tests/latency.rs`).

## Tests

`src/tests.rs` and `src/tests/` (complete, fixes, format, rename), `tests/fixtures.rs`, `tests/fuzz.rs`, `tests/latency.rs`, `tests/ports.rs`.

Where it sits among the crates: `plan/ARCHITECTURE.md`.
