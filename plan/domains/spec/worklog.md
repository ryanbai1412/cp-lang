# spec worklog

Newest first: what landed (commit), evidence, next, blocked, requests from other lanes.

## 2026-11-22
- Request from tech-debt (cold review round 1 of the packet, e258a752; rubric right home 4, one way 3, local next change 4, legible 4, finished migration 3). Paths check out; `ls -d spec/conformance/*/ | wc -l` gives 55 groups.
  1. `crates/cpl-conform` has no owner: idioms and spec both flag it. Tech-debt recommends compiler owns the crate, spec owns the case format and its contract tests (`crates/cpl-conform/tests/ratchet.rs`), idioms and stdlib own the data they feed it; one ask to the coordinator.
  2. Two pass counts: `research/scoreboard.json` (1222 entries, 995 pass) vs 1711 cases / 1617 pass. Make `cpl-conform all --check` write the scoreboard it gates on, and have `COVERAGE.md` read that, so there is one count.
  3. Put `conformance-coverage.py --check` in `scripts/check.sh` through `uv run`. It also regex-reads `crates/cpl-diag/src/registry.rs` (see the diagnostics worklog); a machine-readable registry export would remove that parse.
  4. The 83 `python`-fenced CP Lang blocks in `spec/core`: retag them `cp` in one commit, so `spec_blocks.rs` and the site highlight them as CP Lang.
- Review packet: README, PLAN, metrics.toml, ARCHITECTURE. Evidence: every number in metrics.toml rerun at 4b8f504f (`cpl-conform conformance spec-blocks`, `--rules`, `conformance-coverage.py`).
- Next: phase 2 starts with landing `devin/wip-conformance-matrix`.
- Blocked: nothing. Open: owner for `crates/cpl-conform` (asked the coordinator).
- Lane opened.
