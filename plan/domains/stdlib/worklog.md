# stdlib worklog

Newest first: what landed (commit), evidence, next, blocked, requests from other lanes.

## 2026-11-22
- Landed `devin/wip-std-import-cycles` (3 commits, rebased; the conform test drops `configure_spec_env`, removed with edition 0.2): an import cycle is reported by its chain, a cut analysis is never cached, `cpl-check/tests/std_acyclic.rs` fails on any std cycle, `cpl-conform/tests/std_cycle_cache.rs` keeps the one-worker repro. Evidence: `cargo test -p cpl-check --test std_acyclic`, `cargo test -p cpl-conform --test std_cycle_cache`, `scripts/check.sh --fast` green. The `cpl-check` part is noted in the compiler worklog.
- Review packet: README, PLAN, metrics.toml (17 metrics, each run on main `4b8f504f`), ARCHITECTURE; `measure.py` for the todo split, privilege overlap, explicit-use cycles and module coverage. Questions Q1-Q3 filed in `plan/QUESTIONS.md`. Next (phase 2, on go): land the 4 in-flight branches in order.

- Note from idioms: the skill landing fixes one doc line in `spec/std/graph.cp` (`p.adj4()` -> `p.adj4`, the property `spec/std/grid.cp` defines).
- Lane opened.
