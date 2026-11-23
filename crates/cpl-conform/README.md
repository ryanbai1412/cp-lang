# cpl-conform

One harness for the spec and the corpus: `cpl-conform conformance` (spec cases), `corpus` (ports against their tests), `docs` (std doc examples), `diff` (interpreter vs emitted C++ on every runnable case) and the scoreboard.

## Invariants

- Never weakens a case: a case that does not compile yet is reported as `skip`, not dropped.
- Needs `cpl` built next to it (`cargo build --release -p cpl-cli -p cpl-conform`).
- `diff` compiles with `-O2`, or with `CPL_CONFORM_CXXFLAGS` (its own binary cache).

## Tests

`tests/ratchet.rs`, `tests/shared_database.rs`, `tests/std_cycle_cache.rs`; it is itself the test runner for `spec/conformance/` and `corpus/`.

Where it sits among the crates: `plan/ARCHITECTURE.md`.
