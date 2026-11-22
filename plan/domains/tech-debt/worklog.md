# tech-debt worklog

Newest first: what landed (commit), evidence, next, blocked, requests from other lanes.

## 2026-11-22
- Dead edition references 120 -> 0 in `cpl-check`/`cpl-ir` (note in `plan/domains/compiler/worklog.md`); `edition_refs` baseline tightened to 0.
- Cold review round 1 (packets on main by 84cdb28e), rubric scores (right home / one way / local next change / legible / finished migration): diagnostics 4/3/4/4/3, site 4/4/4/4/3, compiler 3/3/3/4/2, lint-fmt 3/2/3/4/3, idioms 3/3/4/4/2, stdlib 4/2/3/4/3; findings in each lane's worklog. `stale_arch_paths` has false positives (gitignored build output such as `web/out`, missing files named on purpose in known-debt lists), so it stays ungated until the check skips ignored paths.
- Ratchets: every count in metrics.toml is now a shrink-only counter in `scripts/debt-ratchet.py` (baseline = today, `scripts/debt-ratchet.json`); a new crate edge or a new file over 2,000 lines fails; `--show`/`--list NAME` replace `measure.py`. decisions.md 214. Evidence: `python3 scripts/debt-ratchet.py --check` passes, `python3 -m unittest scripts/test_debt_ratchet.py` 7 tests OK.
- Review packet: README, PLAN, metrics.toml, ARCHITECTURE, three entries in `plan/QUESTIONS.md`, `measure.py` (read-only counts, 0.4 s). Baseline on `4e2de3cd`: rs_over_cap 24, edition_refs 92, rust_ignore 19, ir_builtin_names 89, backend_std_names 17, dup_fn_bodies 5, layer_violations 1, v02_port_files 877, spec_std_refs 151, unlanded_branches 57.
- Next (phase 2, on go): land `devin/drop-edition`, then `devin/wip-split-vm`.
- Lane opened.
