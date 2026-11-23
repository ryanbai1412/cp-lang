# cpl-codegen

Lowers the IR to one deterministic, self-contained C++17 file: layout, widths, fast I/O, size budget.

## Invariants

- Deterministic: the same program gives byte-identical C++.
- No undefined behaviour in emitted code; `CPL_CONFORM_CXXFLAGS="-O1 -g -fsanitize=address,undefined" cpl-conform diff` checks it.
- Reads the IR only; library type names here are counted by `backend_std_names`.

## Tests

`src/tests.rs`, `src/copy_tests.rs`, `src/devirt_tests.rs`; end-to-end in `cpl-cli/tests/` and `cpl-conform diff`.

Where it sits among the crates: `handbook/design/architecture.md`.
