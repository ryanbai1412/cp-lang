# Two gate tiers (2026-11-22, built)

`spec/` does not describe the gate; this note does, until it is built.

## Problem
`scripts/check.sh --fast` is both the pre-commit hook and the land gate. Its
cache skips a stage only when the whole tree minus a few prose paths is
unchanged (`scripts/check_cache.py` `STAGES`), so any edit under `crates/`
reruns all 1116 nextest tests: 3.7 s with nothing changed, 77 s after
one line in `cpl-fmt` (measured, see `metrics.toml`). Ryan's target is about
10 s.

## Design
| Tier | Runs on | What | Budget |
|---|---|---|---|
| commit | pre-commit, `just check` | fmt and clippy of touched crates; unit + integration tests of the crates the change can reach in the cargo graph (reverse dependencies included), minus tests tagged `e2e`; quick script checks on touched paths | <= 10 s warm |
| land | `land.sh` on the rebased tip, CI on push | today's `--fast`, unchanged | <= 60 s warm, 8 cores |
| full | `just check-full`, nightly CI | today's `--full` | none |

- **Selection is computed, not listed.** `cargo metadata` gives the reverse
  dependency closure of crates owning changed files; a change outside
  `crates/` maps through the existing `STAGES` excludes. Unknown paths select
  everything (safe default).
- **`e2e` is a nextest test group** (`.config/nextest.toml`), so moving a
  test between tiers is one line, and the land tier still runs it.
- **Nothing is dropped**: the land tier is today's gate; the test count is a
  metric that may not fall.

## Why not
- *Only cache harder (per-test hashes)*: needs file-level dependency
  tracking for Rust tests that read corpus files at runtime; fragile.
- *Only move slow tests to `--full`*: 52 s of fast tests remain and nobody
  runs `--full`; the land tier keeps them on every landing instead.
- *sccache / mold*: compile is not the bottleneck on a warm tree (clippy 5 s).

## As built
`scripts/check.sh --commit` (the pre-commit hook, `just check-commit`) runs
the staged-secrets guard, `cargo fmt --all --check`, clippy of the touched
crates, and `cargo nextest --profile commit -E 'rdeps(<touched>)'`; the
`commit` profile in `.config/nextest.toml` leaves out the integration tests
of `cpl-cli`, `cpl-lsp` and `cpl-conform`. `scripts/affected.py` maps paths
(tested in `scripts/test_affected.py`). Measured: 6 s after a `cpl-fmt`
edit, 11 s after a `cpl-check` edit (every crate), 2 s with nothing staged.
`just check` stays the land tier, so an agent can run what `land.sh` will.

## Reverse
Point `.githooks/pre-commit` back at `check.sh --fast`.
