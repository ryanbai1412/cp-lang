# Restructure plan

The agreed plan for merging branches, finishing the v0.3.0 spec and cleaning up
the repository. Delete this file once every step is done.

## Order

1. **Merge.** Merge `main` into `devin/spec`, resolve conflicts, pass
   `scripts/check.sh --fast`, then fast-forward `main` to the result. One
   branch from then on: work on `main` or short-lived branches merged daily.
   - `spec/`, `decisions.md` entries 157+ and the spec checker changes keep the
     spec side; ports, factory and LSP work keep `main`'s.
   - `decisions.md`: `main`'s colliding entries are renumbered after the
     spec's; nothing is dropped. Cross-references are updated.
   - `plan/DOMAINS.md` keeps the readable map on top and the agent audit below.
2. **Finish the spec.** It is finished when:
   - every normative rule has a conformance case (`scripts/conformance.py
     --rules` reports none uncovered);
   - no contradiction in `spec/delta/doc-notes/` is unresolved, checked by
     reading, not by a heuristic count;
   - an agent that has read only `spec/` agrees with us on the rebuild
     questions, and every hole it finds is fixed.
3. **Cleanup.** Delete as much as possible, after moving every unique piece
   of information into the layout below. No `archive/`: git history keeps the
   old files.

## Target layout

| Path | Holds |
|---|---|
| `AGENTS.md` | mission, goals with their tests, design rules, how agents work |
| `decisions.md` | numbered design decisions, each with its reason and how to reverse it |
| `spec/core`, `spec/lib`, `spec/tools` | the only normative source: language, library design, tool behaviour, with rule IDs |
| `spec/std/` | documented signatures for every library function |
| `spec/conformance/` | rule-linked cases: compiles or not, expected output or diagnostic |
| `corpus/problems/<id>/` | `meta.json`, `dream.cp`, `spec.cp`, `port.cp` (until `spec.cp` runs), `notes.md`, `tests/` |
| `corpus/submissions/` | every raw submission, kept; `meta.json` links to its originals here |
| `corpus/bugs/` | real-bug replay set by class |
| `crates/` | compiler, interpreter, LSP, CLI, with tests beside each crate |
| `stdlib/` | library implementation |
| `bench/` | the one perf harness and its results |
| `tools/trace/`, `editors/`, `site/`, `examples/` | trace viewer, editor plugins, docs site and playground, sample library |
| `plan/` | `DOMAINS.md`, `OPERATING.md`, `agent-context.md`, `PORTING.md` only |
| `research/` | dated, non-normative history: reports, reviews, retros, design debates |
| `scripts/` | gates: `check.sh`, conformance, spec-stub, scoreboard |

## Migration rules

- Every deleted file gets a row in `research/MIGRATION.md`: each section and
  where its content went, so lossiness can be checked.
- Nothing is summarised away. Content goes to the home that matches what it is:

| Old | Goes to |
|---|---|
| `design/GOALS` | `AGENTS.md`; any line missing there is added verbatim |
| `design/ACCEPT`, `00-grounding`, `EXEMPLARS*` | `corpus/problems/<id>/` dreams plus `notes.md` |
| `design/COST` | cost table in `spec/core/13-lowering.md` |
| `design/IDIOMS` | `site/content/cpp-guide.md` |
| `design/ALIGN`, `CORE-ALIGN` | resolved answers become `decisions.md` entries; debate text moves verbatim to `research/design-history/` |
| `design/spec`, `DELTA.md`, `plan/1x` | superseded by `spec/`; unique rules added there first |
| `plan/30`–`70` | rules with IDs in `spec/tools/` |
| `plan/80-validation.md` | `corpus/VALIDATION.md` |
| `plan/90-docs-and-playground.md` | `site/README.md` |
| `plan/STDLIB.md` | `spec/std/README.md` |
| `proto/ports/*` | `corpus/problems/<id>/port.cp` |
| `proto/` | deleted, except the random program generator, which moves into `crates/` for interpreter-vs-C++ parity |
| `subs/` | deleted (exact copy of `corpus/submissions/`) |

- Before deleting, repoint live references: `crates/cpl-check/src/tests.rs`
  and `scripts/scoreboard.py` (`plan/STDLIB.md`), `site/exemplars.py`
  (`design/EXEMPLARS-PORTS.md`), `crates/cpl-syntax/src/planned.rs`
  (`design/spec`), prover and lends files (`plan/19`). `grep -rn` for each path
  before its file goes.
- `AGENTS.md` reading list points at `spec/`, not `plan/1x`.

## Toolchain (during cleanup)

| Area | Tool |
|---|---|
| Versions | `mise.toml` pins Rust, Python, Node, uv, pnpm, just |
| Rust | Cargo workspace, `rust-toolchain.toml`, nextest, clippy, rustfmt |
| Python | root `pyproject.toml`, uv workspace and `uv.lock`, Python 3.12, ruff, pytest |
| JS/TS | root pnpm workspace, biome, vitest; replaces `package-lock.json` |
| Tasks | `justfile`: `just check`, `just test`, `just site`, `just conformance` |
| Hooks | lefthook, replacing `scripts/install-hooks.sh` |
| CI | none for now (no GitHub Actions); the local hook is the gate. Add later |

The site generator stays in Python.

## Other cleanup items

- Conformance gate: the number of met cases never drops.
- Rename `stdlib/spec.cp` to `stdlib/brute.cp`.
- Delete fully merged remote branches; stale unmerged ones need Ryan's yes.
- Deferred until after the merge: interpreter consolidation, removing stdlib
  privilege, splitting monolith files.
