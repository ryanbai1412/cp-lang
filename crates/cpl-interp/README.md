# cpl-interp

The debug interpreter (`cpl run`): a register VM over `cpl_ir::Program` that checks what the submission build does not (indices, input ranges, overflow, `!` on `None`, use during mutation) and records deterministic traces.

## Invariants

- A failed check is a `Trap` with its source span, never an interpreter crash.
- Output matches the emitted C++ for every valid program (`cpl-conform diff`).

## Tests

`src/tests.rs`, `tests/trace_log.rs`; agreement with C++ in `cpl-conform diff`.

Where it sits among the crates: `plan/ARCHITECTURE.md`.
