# spec worklog

Newest first: what landed (commit), evidence, next, blocked, requests from other lanes.

## 2026-11-22
- Request from compiler: `core-05/a-user-method-on-self-does-not-change-its-receiver.cp` expects `1 x` from `print(p.first.ord, q.first)` with `p.first` = `City.first!` of `1..=3`. `.ord` of a position is its 0-based index everywhere else (`core-08/infer-domain-from-bounds.cp` `n + v.ord`, `spec/lib/rationale/seq.md` `.ord + 1`), so `cpl run` prints `0 x` once the E0403 is fixed. Please make it `p.first.label` (prints `1 x`) or `0 x`, whichever the case meant; compiler's regression `crates/cpl-cli/tests/programs/own_swap_method.cp` covers the method itself.
- Request from tech-debt (cold review round 1 of the packet, e258a752; rubric right home 4, one way 3, local next change 4, legible 4, finished migration 3). Paths check out; `ls -d spec/conformance/*/ | wc -l` gives 55 groups.
  1. `crates/cpl-conform` has no owner: idioms and spec both flag it. Tech-debt recommends compiler owns the crate, spec owns the case format and its contract tests (`crates/cpl-conform/tests/ratchet.rs`), idioms and stdlib own the data they feed it; one ask to the coordinator.
  2. Two pass counts: `research/scoreboard.json` (1222 entries, 995 pass) vs 1711 cases / 1617 pass. Make `cpl-conform all --check` write the scoreboard it gates on, and have `COVERAGE.md` read that, so there is one count.
  3. Put `conformance-coverage.py --check` in `scripts/check.sh` through `uv run`. It also regex-reads `crates/cpl-diag/src/registry.rs` (see the diagnostics worklog); a machine-readable registry export would remove that parse.
  4. The 83 `python`-fenced CP Lang blocks in `spec/core`: retag them `cp` in one commit, so `spec_blocks.rs` and the site highlight them as CP Lang.
- Landed `devin/wip-conformance-matrix` (6 commits, rebased; its decisions 214-215 renumbered 218-219 after a clash on main) and took ownership of `crates/cpl-conform` and `scripts/conformance-*.py` (coordinator). Evidence: `conformance-matrix.py --check` exit 0; `cpl-conform conformance` 1845/2030; `cpl-conform all --check` 0 regressions; `conformance-coverage.py --check` exit 0, now in the fast gate.
- The branch edits doc comments in `spec/std/prefix.cp` and `spec/std/subset.cp` (decisions 219-220); noted in the stdlib worklog.
- Next: `scripts/conformance-traits.py`.
- Review packet: README, PLAN, metrics.toml, ARCHITECTURE. Evidence: every number in metrics.toml rerun at 4b8f504f (`cpl-conform conformance spec-blocks`, `--rules`, `conformance-coverage.py`).
- Next: phase 2 starts with landing `devin/wip-conformance-matrix`.
- Blocked: nothing. Open: owner for `crates/cpl-conform` (asked the coordinator).
- Lane opened.
