# Restructure plan

The agreed plan for merging branches, finishing the v0.3.0 spec and cleaning up
the repository. Delete this file once every step is done.

## Order

1. **Merge.** Merge `main` into the `spec` branch, resolve conflicts, pass
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

| Old | Goes to | Status |
|---|---|---|
| `design/GOALS` | `AGENTS.md`; preserved verbatim in `research/design-history/GOALS.md` | Done |
| `design/ACCEPT`, `00-grounding`, `EXEMPLARS*`, `design/sketches/ioi2025/*.cp` | `corpus/problems/ioi25_*/dream.cp` + `research/design-history/` | Done |
| `design/COST` | cost table in `spec/core/13-lowering.md` (§13.8) | Done |
| `design/IDIOMS` | `site/content/cpp-guide.md` + `research/design-history/IDIOMS.md` | Done |
| `design/ALIGN`, `CORE-ALIGN` | `research/design-history/` | Done |
| `design/spec`, `DELTA.md`, `plan/1x` | superseded by `spec/` (`DELTA.md` preserved in `research/design-history/DELTA.md`) | Done |
| `plan/30`–`70` | preserved in `research/design-history/plan-v0.2/`; normative rules live in `spec/tools/` | Done |
| `plan/80-validation.md` | `corpus/VALIDATION.md` | Done |
| `plan/90-docs-and-playground.md` | `site/README.md` + `research/design-history/plan-v0.2/90-docs-and-playground.md` | Done |
| `plan/STDLIB.md` | `spec/std/README.md` | Done |
| `proto/ports/*` | `corpus/problems/<id>/port.cp` (`dp*` renamed to `atcoder_dp_<letter>`) | Done |
| `proto/` | deleted; all paths recorded in `research/MIGRATION.md` | Done |
| `subs/` | deleted | Done |
| `archive/` | moved to `research/design-history/`; `archive/` deleted | Done |

- Before deleting, repoint live references: `crates/cpl-check/src/tests.rs`
  and `scripts/scoreboard.py` (`plan/STDLIB.md`), `site/exemplars.py`
  (`design/EXEMPLARS-PORTS.md`), `crates/cpl-syntax/src/planned.rs`
  (`design/spec`), prover and lends files (`plan/19`). `grep -rn` for each path
  before its file goes.
- `AGENTS.md` reading list points at `spec/`, not `plan/1x`.

## Toolchain (during cleanup)

| Area | Tool | Status |
|---|---|---|
| Versions | `mise.toml` pins Rust, Python, Node, uv, pnpm, just | Done |
| Rust | Cargo workspace, `rust-toolchain.toml`, nextest, clippy, rustfmt | Done |
| Python | root `pyproject.toml`, `uv.lock`, Python 3.12, ruff, pytest | Done |
| JS/TS | root `package.json`, `pnpm-workspace.yaml` (`web/`, `editors/vscode/`) | Done |
| Tasks | `justfile`: `just check`, `just check-full`, `just test`, `just fmt`, `just bench`, `just docs`, `just site`, `just conformance`, `just worktree` | Done |
| Hooks | lefthook (optional; `scripts/install-hooks.sh` still active) | Open |
| CI | none for now (no GitHub Actions); the local hook is the gate. Add later | Done |

The site generator stays in Python.

mise and just do different jobs: mise installs and pins tool versions (run
once per machine); just runs project commands (run daily). Commands stay in
the `justfile`, not mise tasks, so dropping mise never breaks a command.

## Tooling improvements (during cleanup)

Tests:
- One home per kind: `spec/conformance/` for language meaning (rule IDs),
  per-crate golden dirs for tool behaviour. Fold `crates/cpl-cli/tests/cases/*` and
  `crates/cpl-cli/tests/programs` into those.
- Expectations live in the test file (`// error:` header), never in a Rust
  `match` beside it. One `just bless` updates every golden kind.
- Timing tests (budget, latency, profile, benchmark) move to `just bench`
  with a quiet-machine check, out of the commit gate; under load they fail
  without a real bug.
- One language mode: v0.3 behaviour without the `CPL_SPEC` env switch;
  old behaviour, if needed, behind an explicit edition flag.
- The random program generator moves into `crates/` as an interpreter vs
  C++ differential test with shrinking. It also runs the bytecode VM
  against the tree-walker: `cpl-interp`'s `vm.rs` serves plain runs and
  falls back to `lib.rs` for what it does not lower, while traced and
  profiled runs use only the tree-walker, so a bug can live in one engine.
  This parity test lands before the interpreter consolidation below.
- Timing tests measure the release `cpl`, never the dev-profile binary
  `cargo test` builds (the budget test does this now).
- Conformance ratchet: the met count never drops; newly met cases are
  recorded automatically.

Docs that code reads:
- A link checker over code and markdown fails the fast gate when a cited
  path or section disappears (planned-feature links, `plan/STDLIB.md`,
  `design/EXEMPLARS`).
- A script assigns the next decision number, so branches cannot collide.

Scripts:
- The ~60 loose Python scripts become one tested package with subcommands
  (conformance, stub audit, scoreboard); scripts nothing calls are deleted.
- The `justfile` is the only entry point; `check.sh` logic moves behind it.
  `just check` runs only tests of changed crates (target under a minute);
  `just test` runs everything, before push.

Agent ergonomics:
- Generate the agent half of `DOMAINS.md`, `STATUS.md` counts and the
  `REPO.md` tree instead of hand-maintaining them.
- `just worktree <name>`: an isolated worktree with its own `target/`.
- `mise install && just check` is the whole onboarding.

Language tools on our own code:
- `cpl fmt --check` and `cpl lint` over corpus, stdlib and conformance in
  the gate; LSP golden snapshots cover every diagnostic code.

## Other cleanup items

- Conformance gate: the number of met cases never drops.
- Done: one perf harness. `bench/rust_latency_audit.py` and
  `bench/rust_max_n_audit.py` stay; `bench/max_n_audit.py`, `tools/bench/`
  and `proto/bench.py` are deleted.
- Done: `crates/cpl-verify` and the Python prototype are deleted. Its
  generator was not kept: it generated the prototype's subset, and the
  type-directed generator in `plan/ARCHITECTURE.md` replaces it.
- Done: `proto/` deleted (`ports/` moved into `corpus/problems/<id>/`, `tests/` moved to `crates/cpl-cli/tests/cases/`).
- Done: `bench/corpus_registry.py` deleted; `auto_strip` folded into `scripts/scoreboard.py` over `corpus/problems/`.
- Open: Rename `stdlib/spec.cp` to `stdlib/brute.cp` (owned by stdlib session).
- Open: Delete fully merged remote branches; stale unmerged ones need Ryan's yes.
- Deferred (other sessions): `crates/cpl-conform` (session 71e45566), interpreter consolidation, removing stdlib privilege, splitting monolith files (`plan/ARCHITECTURE.md`).

## Duplicated-helper audit

All hand-rolled zero-dependency helpers audited across `crates/` and consolidated into single owners (zero third-party crates):

| Helper | Copies before | Lines before | Lines after | New home / decision |
| --- | --- | --- | --- | --- |
| JSON value, parser, writer | 5 (`cpl-lsp/src/json.rs` 387, `cpl-conform/src/json.rs` 359, `cpl-contest/src/json.rs` 197, `cpl-doc/src/json.rs` 71, inline `enum Json` in `cpl-cli/src/dap.rs` ~240) | ~1,254 | 752 (incl. unit tests) | `crates/cpl-json` (`BTreeMap<String, Value>` for deterministic sorted-key output, O(log n) lookup, and structural equality; `cpl-doc` retains thin renderer delegating escaping to `cpl-json`) |
| JSON string escaping & mini-decoders | 6 (`cpl-interp/src/trace.rs`, `cpl-cli/src/trace.rs`, `cpl-cli/src/profile.rs`, `cpl-stress/src/cli.rs`, `cpl-cli/tests/seeded_bugs.rs`, `cpl-cli/tests/stress_corpus.rs`) | ~95 | 17 (thin wrappers + `cpl_json::write_script_safe_string` / `cpl_json::parse`) | `crates/cpl-json` (`escape_string`, `write_script_safe_string`, `parse`) |
| `Content-Length` stdio framing | 3 (`cpl-lsp/src/transport.rs`, `cpl-cli/src/dap.rs`, `cpl-cli/tests/dap.rs`) | ~138 | 84 (single impl in `transport.rs`) | `crates/cpl-lsp/src/transport.rs` (`read_message`, `write_raw_message`, `write_message`) |
| CLI flag/arg parser (`Args`) | 2 (`cpl-contest/src/args.rs` 68, `cpl-stress/src/cli.rs` ~55) | ~123 | 105 (incl. unit tests) | `crates/cpl-stress/src/args.rs` (reused by `cpl-contest` and `cpl-stress`) |
| Token/line "first difference" | 1 full diff (`cpl-stress/src/judge.rs::first_difference`, reused by `cpl-contest`); `cpl-conform` uses 1-line `split_whitespace().eq(...)` | 1 copy | unchanged | Kept in `cpl-stress/src/judge.rs` (no duplicate diff engine to merge) |
| Subprocess with timeout | 1 single-process (`cpl-stress/src/program.rs`) + 1 dual-process interactor loop (`cpl-stress/src/judge.rs`) | 1 crate (`cpl-stress`) | unchanged | Kept in `cpl-stress` (single owner; `cpl-conform` uses plain `wait_with_output`) |
| Scratch/temp dirs in tests | ~10 three-line `fn scratch(name)` test helpers across independent crates (`cpl-fmt`, `cpl-doc`, `cpl-lint`, `cpl-contest`, `cpl-stress`, `cpl-cli/tests`) | ~3 lines each | unchanged | Not consolidated — independent crates would require a new shared test-util crate for a 3-line `std::env::temp_dir().join(...)` |
| HTML escaping / URI percent-decode / FNV-1a hex / hex literal | 1 each (`cpl-doc/src/highlight.rs`, `cpl-lsp/src/server.rs`, `cpl-conform/src/diff.rs`, `cpl-syntax/src/literal.rs`) | 1 copy each | unchanged | Already single-owner; no duplication |

