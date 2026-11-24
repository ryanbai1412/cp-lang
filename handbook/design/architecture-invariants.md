# Architecture invariants

| | |
|---|---|
| Kind | Design |
| Owner | coordinator, kept by Content |
| Status | Active |
| Describes | `scripts/debt-ratchet.py`, `scripts/check.sh` |

Each surface of the toolchain has exactly one mechanism. This page names that
mechanism, its owner and the gate that rejects a second one. A surface with no
mechanism yet says what will trigger its design. Doing so keeps the design
choice from being made by accident, in whichever tool happens to need it first.

[Sources of truth](sources-of-truth.md) explains the three kinds of single
source (one entry point per job, generated never written, never written
twice). This page is the full list of surfaces. A rule exists here only if a
gate enforces it. A row whose gate column says "to build" is a debt, and
Factory owns it.

## The rules behind every row

- **One mechanism per surface.** A second one is a bug in the newer of the two.
- **A second implementation carries its own deletion plan.** A fallback, shim,
  cache of another tool's data, allowlist or skip list lands with a ratchet
  counter, an owner and a deletion step, all in the same commit. The gate:
  a new list file or allowlist with no `LISTS` row (counter, owner, deletion
  plan) in `scripts/debt-ratchet.py` fails `unregistered_lists`, a hard zero.
- **Interface before consumers.** The owner publishes a surface's API shape
  before anyone builds on it. Shared analysis is per-revision and
  query-shaped, so incremental editor analysis can extend it later without
  forking it.
- **Rules are lints, not prose.** Each row's gate is a deterministic check in
  `scripts/debt-ratchet.py` or `scripts/check.sh`. If a rule can't be checked
  that way, it doesn't belong on this page.
- **Hunt for what no counter sees.** Each lane spends about 5% of its time
  looking for debt that no counter tracks. When it finds some, it adds a
  counter on the day it finds it.

# Reference

## Surfaces

| Surface | The one mechanism | Owner | Gate (counter or check) |
|---|---|---|---|
| Analysis | `cpl_check::Session` is the only thing that builds an analysis; it is per-revision and query-shaped | Core, with Consumers | `own_databases`, `consumer_rederived` |
| Std names in the compiler | Only the intrinsics table (decision 266) may name a std item | Core | `ir_builtin_names`, `backend_std_names`, `lint_std_names`, `prelude_privilege` |
| Std metrics | Each `plan/domains/*/measure.py` has a self-test that fails on a `0 of 0` reading | Content | `measure_broken`, `scripts/test_stdlib_measure.py` |
| Std bodies | Every pub std item runs its CP Lang body; the compiler never bypasses it | Content | `std_shadowed` (std pub names the compiler also matches by string, intrinsics excluded) |
| Running programs | The VM, also built to wasm; no interpreter fallback path; compile-time growth is refused (E0319) | Core | `vm_fallbacks`, `tree_walker`, `mono_unguarded` |
| Crate dependencies | The layer map in `design/architecture.md` | Factory | `layer_violations`, `crate_edges` |
| Tool data on the site | `web/` reads tool output; nothing is copied in by hand | Factory | to build: `web_hand_copied` |
| Machine output | One versioned schema crate, `cpl-json`, for `--format json`, LSP, wasm and trace files; no `serde_json` or hand-built JSON anywhere else | Consumers | `json_outside_cpl_json` |
| Diagnostics | One registry (code, message key, fix, docs) read by checker, lint, diag and docs; no diagnostic text outside it | Consumers, with Core | to build: `diag_text_outside_registry` |
| Configuration | One schema for `cp.toml`, judge profiles, CLI flags and editor settings | Consumers | `config_schemas` |
| Std documentation | One doc model, produced by the checker, for `cpl doc`, hover, site and completion | Consumers; the site reads it (Factory) | `std_doc_parsers` |
| Source edits | One lossless tree and edit engine for fmt, `cpl fix`, codemods and migrations; every codemod is a `cpl fix` | Consumers | `codemods_outside_fix`, `edit_engines` |
| Execution events | One event model on the VM for trace, profiler, debugger and replay | Core, with Consumers | to build: `event_models` |
| Integer widths and overflow | One width table, read by both the interpreter and codegen | Core | to build: `width_tables` |
| Input and output | One I/O contract, tested across the interpreter, the C++ runtime and the browser | Core, with Factory for the browser | to build: an I/O parity suite |
| Problems | One problem package (`corpus/problems/<id>/`) and one walker, `scripts/corpus.py` `problems()`, used by ports, bench, stress and judge | Content, with Factory for bench | `corpus_walkers` (any other glob of `corpus/problems`), `dream_unverified` (a `dream.cp` without a current `dream_stress` record) |
| Editors | Editors stay thin, with all logic in the LSP | Consumers | to build: `editor_logic_lines` |
| Modules | One module loader and resolver for CLI, LSP, wasm and judge | Core | `module_loaders` |
| Judges and targets | One judge/target table that drives the emitter | Core | to build: `judge_tables` |
| Codegen optimizations | Every optimization reads the effects/Uses table | Core, with Factory | to build: `opt_without_uses` |
| Site | Generated from the tools | Factory | `gen.py --check` |
| Known gaps | A gap is a `// gap:` line in its case file, with `spec/SUBSET.md` generated from them | Content | `skip_lists`; to build: a gap-age gate (48 h) |
| Decisions | One number each, reserved with `next-decision.py --reserve` | Content | `decision_dup_numbers` |

## Designed later

These surfaces are only shapes for now. Each has a stub interface so that the
first real need extends it instead of forking it.

| Surface | Stub | Triggered by |
|---|---|---|
| Incremental analysis | `Session` queries take a revision | the first editor feature that re-checks on every keystroke |
| Package versions and editions | an interface stub in the module loader | the first published library, or the first edition-breaking change |

## Changes

- 2026-11-24: first version, from the coordinator's prevention addenda 1 and 2.
