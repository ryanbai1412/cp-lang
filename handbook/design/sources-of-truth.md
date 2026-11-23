# Sources of truth

| | |
|---|---|
| Kind | Design |
| Owner | tech-debt |
| Status | Active |
| Describes | `scripts/regen.sh`, `scripts/debt-ratchet.py`, `crates/cpl-wasm/tests/parity.rs`, `crates/cpl-cli/tests/lsp_parity.rs` |

Every fact in the code has one owner, and every job has one entry point.
Anything else is generated from the owner or calls the entry point. This
doc lists them and the detector that keeps each one single.

For example, the list of lints exists once, in the lint registry. The
editor's code actions, `cpl lint --list` and the docs page all read the
registry. A second hand-written list would drift, so none is allowed.

## One entry point per job

The CLI, the language server and the playground call the same function
for each job and add only presentation.

| Job | Entry point | Owner |
|---|---|---|
| check | `cpl_check::Session`, being built. Until it lands, each client builds its own database, counted by `own_databases` | compiler |
| lint | `cpl_lint::lint_analysis` on the shared analysis | lint-fmt |
| fix | `cpl_ir::fix`, over `cpl_diag::apply_fixes` for the text edits | diagnostics |
| run | `cpl_interp::run_with` | compiler |
| trace | `cpl_interp::observe_on_this_thread`, with `checkpoint::observe_on_this_thread` for runs that take or resume checkpoints | debugger |
| compare output | `cpl_stress::judge::first_difference` | tools |

Parity tests run each job through every client and compare:
`crates/cpl-wasm/tests/parity.rs` for the playground and
`crates/cpl-cli/tests/lsp_parity.rs` for the editor. A job with a second
entry point is a bug in whichever one is newer.

## Generated, never written

| Fact | Owner | Generated into | By |
|---|---|---|---|
| Conformance coverage and rule gaps | the cases in `spec/conformance/` | `spec/conformance/` reports | `scripts/conformance-coverage.py` |
| Agent skill pages | the spec and the std sources | `skills/` | `scripts/skill-gen.py` |
| Editor language data | the operator table and keyword lists | `editors/vscode/` | `editors/vscode/scripts/language-data.js` |
| Current numbers for each domain | the measuring commands | `plan/domains/<domain>/metrics.toml` | each metric's `command` |

`scripts/regen.sh` regenerates them all and prints what changed. A
committed generated file that differs from a fresh regeneration is stale.

## Never written twice

| Rule | Example of a break | Detector |
|---|---|---|
| A mechanism lives in one crate | Two crates each compare real-number output tokens with their own tolerance | `scripts/debt-ratchet.py` (`near_dup_fns`, `dup_fn_bodies`, `lint_dup_helpers`) |
| Clients read checker facts, never re-derive them | A lint re-computes a type the checker already knows | `scripts/debt-ratchet.py` (`consumer_rederived`) |
| One analysis per process | A tool builds its own query database | `scripts/debt-ratchet.py` (`own_databases`) |
| One engine runs programs | A statement runs on the tree-walker instead of the VM | `scripts/debt-ratchet.py` (`vm_fallbacks`, `tree_walker`), `crates/cpl-cli/tests/vm_fallbacks.rs` |
| The stdlib's behavior lives in the stdlib | The IR lowers a std method by its name | `scripts/debt-ratchet.py` (`ir_builtin_names`, `backend_std_names`, `lint_std_names`, `prelude_privilege`) |
| Docs cite code that exists | An architecture doc names a deleted file | `scripts/debt-ratchet.py` (`stale_arch_paths`) |

## Replacing a mechanism

1. The new code lands in its target crate, in its target shape.
2. A replacement deletes what it replaces in the same step, once the new
   one passes the old one's tests.
3. A bridge between old and new code is named, with the step that removes
   it.
4. Reuse existing code only when it already has the target shape.
5. No step leaves a ratchet higher.

## Changes

- 2026-11-23: first draft, from `plan/rearchitecture.md`.
