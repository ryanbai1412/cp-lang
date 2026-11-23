# Architecture & tech-debt (segment B, metric)

**Mission.** The codebase gets simpler every week: each concept lives in one
place, the stdlib has no privilege the compiler grants by name, the 0.2 era is
gone, and an everyday change (a std method, a diagnostic, a lint, an LSP
feature) touches one obvious place. This lane is the steward of
`domain-ontology` section 7: it measures the shape, gates it with ratchets,
and cold-reviews every lane's landed diffs.

Detail: [PLAN.md](PLAN.md) · [metrics.toml](metrics.toml) ·
[ARCHITECTURE.md](ARCHITECTURE.md) · [worklog.md](worklog.md) ·
cross-domain map: [plan/ARCHITECTURE.md](../../ARCHITECTURE.md) ·
ratchet and counts: `scripts/debt-ratchet.py` (`--show`, `--list NAME`; 0.4 s, in the fast gate)

## Covers / not covered

| Covers (this lane) | Not covered (owner) |
|---|---|
| `plan/ARCHITECTURE.md`: crates, edges, shared core | Shared-core code itself (compiler) |
| Crate/file layout; pure-move splits of oversized files | Behaviour changes in those files (owning lane) |
| Dead code, duplicated helpers, 0.2 leftovers | Stdlib bodies and their TODOs (stdlib) |
| Stdlib privilege: measuring it, proposing the floor | Moving builtins into std (compiler + stdlib) |
| `scripts/debt-ratchet.py` and its baseline | The gate, land queue, rest of `scripts/` (infra) |
| Cold reviews of every lane's diffs; change-cost probes | Fixing findings (owning lane, before its features) |
| Repo restructure: `std` -> `std/`, `cpl-prover` rename | Normative wording in `spec/` (spec) |

## Success criteria for v0.3

1. No 0.2 code path, test or corpus file remains (`edition_refs`, `v02_port_files` = 0).
2. One interpreter engine; back ends read the IR only (`tree_walker`, `layer_violations` = 0).
3. Library type names never appear in IR, interpreter or emitter (`backend_std_names` = 0); compiler builtins at or under the floor (decision 266).
4. Every ratchet below is in `scripts/check.sh` and can only go down.
5. Every lane has had two cold reviews; rubric median >= 4/5; the four change-cost probes each touch <= 3 files.

## Milestones

**M1. The 0.2 era and the in-flight restructure are landed** (phase 2, first).
- [ ] `edition_refs` 0 (branch `devin/drop-edition`), `v02_port_files` 0
- [ ] `vm.rs` split landed (`devin/wip-split-vm`); `rs_over_cap` <= 23
- [ ] `std/` at the root, `spec_std_refs` 0; `crates/cpl-prover` -> `crates/cpl-prover`
- [ ] `direct_python` 0 (uv everywhere); `just gen --check` in the gate
- [ ] `unlanded_branches` 0: each landed or deleted with a `git cherry` note

**M2. The gates hold the shape.**
- [x] Ratchets in `scripts/debt-ratchet.py` for every metric below (one script, one baseline)
- [x] A new crate edge fails the gate until the baseline and `plan/ARCHITECTURE.md` list it
- [x] Round 1 cold review of the 11 other lanes, scores in the worklog, findings in owners' worklogs (2026-11-22)
- [x] Probe baseline P1-P4 recorded (files, crates, minutes, where stuck) (2026-11-22)
- [x] `plan/ARCHITECTURE.md` part B1 rewritten to measured current state (2026-11-22)

**M3. v0.3 shape** (exit = success criteria above).
- [ ] `tree_walker` 0, `layer_violations` 0, `backend_std_names` 0, `dup_fn_bodies` 0
- [ ] `ir_builtin_names` <= signed floor; `rs_over_cap` <= 8; `rust_ignore` <= 3
- [ ] Round 2 cold reviews: median >= 4; probes <= 3 files each

## Metrics (all run 2026-11-22 on `4e2de3cd`; commands in [metrics.toml](metrics.toml))

`M` = `python3 scripts/debt-ratchet.py --show`; `--list NAME` prints the offenders

| Metric | Command | Today (2026-11-22, after phase 2 lands) | v0.3 target |
|---|---|---|---|
| Rust files over 2,000 lines | `M rs_over_cap` | 23 | <= 8, none new |
| Largest Rust file (lines) | `find crates -path '*/src/*' -name '*.rs' \| xargs wc -l \| sort -n \| tail -2 \| head -1` | 15,367 (`cpl-interp/src/vm.rs`) | <= 4,000 |
| Dead edition references | `M edition_refs` | 0 (was 92) | 0 |
| `#[ignore]` Rust tests | `M rust_ignore` | 17 | <= 3 |
| Builtin names string-matched in lowering | `M ir_builtin_names` | 86 (was 89) | <= 66, the floor (decision 266) |
| Std type names in IR/interp/codegen | `M backend_std_names` | 17 | 0 |
| Duplicated fn bodies across files | `M dup_fn_bodies` | 4 | 0 |
| Shipped crate edges | `M crate_edges` | 75 | no unreviewed new edge |
| Layer violations (shipped) | `M layer_violations` | 1 (`cpl-lint -> cpl-prover`) | 0 |
| Second interpreter engine | `M tree_walker` | 1 | 0 |
| v0.2 `port.cp` files | `M v02_port_files` | 877 | 0 |
| `std` references | `M spec_std_refs` | 300 (counts docs too; was 149 on the narrower pattern) | 0 |
| Direct `python3` in gate/just | `M direct_python` | 13 | 0 |
| Script tests the gate never runs | `M unrun_script_tests` | 3 | 0 |
| Stale paths in architecture docs (not gated yet) | `M stale_arch_paths` | 9 (`--list stale_arch_paths`) | 0 |
| Std TODO bodies (ratchet; stdlib drives) | `python3 scripts/debt-ratchet.py --check` | 162 | never up |
| `devin/*` branches with unlanded commits | `git branch -r \| grep -c origin/devin/` | 47 remote, triage in worklog.md | 0 |
| Cold-review rubric median | review log in worklog.md | 3 (round 1, 11 lanes x 5 questions) | >= 4 |
| Change-cost probes, files touched | probe log in worklog.md | cold run 2026-11-22: P1 8, P2 7, P3 11, P4 5; P5 3/5 | <= 3 each |

## Judgment rubric (cold review of every lane's landed diffs)

Each scored 1-5 by a reader who has not seen the work, on the last ~2 weeks
of `git log --grep 'Domain: <lane>'`.

1. **Right home.** Does each change live in the crate that owns the concept, with no new special case for a stdlib name?
2. **One way.** Does it reuse the existing mechanism or helper instead of adding a second walker, analysis, catalog or copy?
3. **Local next change.** Could the next everyday change in this area be made in one place?
4. **Legible cold.** Can a cold agent explain the module in 5 minutes from names, `ARCHITECTURE.md` and doc comments, and are they current?
5. **Finished migration.** Does it delete what it replaces, with any bridge listed as debt?

## Probes (a cold agent attempts; we log files, crates, minutes, stuck points)

| Probe | Tests rubric | Owner of fixes |
|---|---|---|
| P1 Add a std method with a doc example (`List.rotate`) | 1, 3 | stdlib, compiler |
| P2 Add a diagnostic code with a fix-it | 2, 3 | diagnostics |
| P3 Add a lint with its test | 2, 3, 4 | lint-fmt |
| P4 Add an LSP hover field | 2, 3 | tools |
| P5 Explain a random module from `ARCHITECTURE.md` only | 4 | owning lane |

## Top 10 architectural debt today (ranked; detail in [ARCHITECTURE.md](ARCHITECTURE.md#known-debt))

| # | Debt | Where | Owner |
|---|---|---|---|
| 1 | Stdlib privilege: 89 names matched in lowering, 74-variant `Builtin` enum | `crates/cpl-ir/src/lower/call.rs` (13,357 lines), `crates/cpl-ir/src/lib.rs` | compiler + stdlib |
| 2 | Two interpreter engines; VM falls back to the tree-walker | `crates/cpl-interp/src/lib.rs` (`fn eval`), `vm.rs` | compiler |
| 3 | Dead 0.2 edition switch, 92 refs | `crates/cpl-check/src/edition.rs`, `check/expr.rs` (32 refs) | compiler (branch ready) |
| 4 | Two range/bound analyses; lint calls the prover directly | `crates/cpl-lint/src/rules/bounds.rs`, `crates/cpl-prover/` | lint-fmt + compiler |
| 5 | 24 files over 2,000 lines | `vm.rs` 15,367; `lower/{call,stmt,mod,expr}.rs` 6-13k | many; this lane moves |
| 6 | Library type names known to IR and emitter | `lower/call.rs`, `lower/mod.rs:24`, `cpl-codegen/src/expr.rs:2780` | compiler + perf |
| 7 | Std member catalog kept twice (checker list vs lowering match), stale rows | `crates/cpl-check/src/builtins.rs`, ignored test `crates/cpl-cli/tests/builtin_catalog.rs:32` | compiler + tools |
| 8 | Browser lints compiled out, web and CLI disagree | `crates/cpl-lint/src/lib.rs:954`, `rules/bounds.rs:680` | lint-fmt |
| 9 | Library source under `spec/`, 877 v0.2 ports, 57 stale branches | `std/`, `corpus/problems/*/port.cp` | this lane + stdlib/idioms |
| 10 | Copied helpers across crates | `cpl-{fmt,lint}/src/cli.rs` `collect` = `cpl-doc/src/cli.rs:163`; `refutable`, `prefix_shape_struct` in check and ir | tools, lint-fmt, compiler |

## Improve list (highest impact first)

1. Land M1: the restructure and 0.2 deletion are written; they only need rebasing and the gate.
2. Done: every count above is a shrink-only ratchet in `scripts/debt-ratchet.py`.
3. Move the 23 derived builtins into std behind the perf gate (decision 266).
4. One VM: file the tree-walker removal plan with compiler; split `vm.rs` first.
5. One bound analysis: lint reads checker facts (proposal to lint-fmt + compiler).
6. Run cold-review round 1 and the probe baseline; findings go to owners' worklogs.
7. Rewrite `plan/ARCHITECTURE.md` B1 from measurements, not memory.

## Ryan's role

Sign off the targets above once (segment B), then read the
weekly numbers in `worklog.md`. Push back on any rubric question or probe that
does not match what "clean" means to you.

## Decisions

Q1-Q3 are decided by their recommendations: decisions 266 (builtin floor),
267 (steward pure moves) and 268 (hard-fail ratchets) in `decisions.md`.
Nothing from this lane is waiting on Ryan.
