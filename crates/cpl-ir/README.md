# cpl-ir

The typed program `cpl run` and `cpl build` share: `lower` turns the checker's `Analysis` into a `Program` with resolved slots, types, and sugar spelled out as loops and tests.

## Invariants

- One place decides what the source means; the interpreter and the C++ back end both read this IR, never syntax or checker internals.
- Strings are bytes; a non-ASCII character literal is an error here.

## Tests

No unit tests of its own: it is tested through `cpl-interp`, `cpl-codegen` and `cpl-conform diff` (interpreter vs C++ on every case).

Where it sits among the crates: `handbook/design/architecture.md`.
