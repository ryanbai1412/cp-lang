# tech-debt worklog

Newest first: what landed (commit), evidence, next, blocked, requests from other lanes.

## 2026-11-22
- Ratchet `prelude_privilege` = 27: public `spec/std` names that `crates/cpl-check/src/prelude.rs` TYPE/VALUE/TRAIT tables also hold (same definition as the stdlib packet's `measure.py privilege`, which reports 27); kept apart from `ir_builtin_names` (89, lowering arms) as the stdlib review proposed. `--list prelude_privilege` names them.
- `plan/ARCHITECTURE.md` B1 refreshed to measured values: 201k Rust lines in 19 crates, one language (0.2 row closed), 89 call.rs names (was 63), harness and interpreter rows updated; B3 step 4 now only the `std/` move and the 0.2 ports.
- Ratchet: `edition_refs` also counts `spec_edition` (the compiler packet's 120 was right; 0 now); `stale_arch_paths` skips gitignored build output and `## Known debt` lists, and stays informational: 10 on main (`--list stale_arch_paths`: 1 compiler, 1 diagnostics, 5 idioms citing `devin/idioms-docs` files, 1 site, and 2 debugger paths that are planned on purpose, `crates/cpl-trace` and `web/packages/trace-viewer`). Gating it would fail every lane that names a planned path; it gates once docs mark planned paths.
- Dead edition references 120 -> 0 in `cpl-check`/`cpl-ir` (note in `plan/domains/compiler/worklog.md`); `edition_refs` baseline tightened to 0.
- Cold review round 1 (packets on main by 84cdb28e), rubric scores (right home / one way / local next change / legible / finished migration): diagnostics 4/3/4/4/3, site 4/4/4/4/3, compiler 3/3/3/4/2, lint-fmt 3/2/3/4/3, idioms 3/3/4/4/2, stdlib 4/2/3/4/3, debugger 4/3/3/4/3, spec 4/3/4/4/3, perf 3/2/3/5/3, tools 3/3/3/4/3, infra 3/3/3/4/4; findings in each lane's worklog.
- Ratchets: every count in metrics.toml is now a shrink-only counter in `scripts/debt-ratchet.py` (baseline = today, `scripts/debt-ratchet.json`); a new crate edge or a new file over 2,000 lines fails; `--show`/`--list NAME` replace `measure.py`. decisions.md 214. Evidence: `python3 scripts/debt-ratchet.py --check` passes, `python3 -m unittest scripts/test_debt_ratchet.py` 7 tests OK.
- Review packet: README, PLAN, metrics.toml, ARCHITECTURE, three entries in `plan/QUESTIONS.md`, `measure.py` (read-only counts, 0.4 s). Baseline on `4e2de3cd`: rs_over_cap 24, edition_refs 92, rust_ignore 19, ir_builtin_names 89, backend_std_names 17, dup_fn_bodies 5, layer_violations 1, v02_port_files 877, spec_std_refs 151, unlanded_branches 57.
- Next (phase 2, on go): land `devin/drop-edition`, then `devin/wip-split-vm`.
- Lane opened.
