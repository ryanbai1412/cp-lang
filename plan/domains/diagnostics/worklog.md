# diagnostics worklog

Newest first: what landed (commit), evidence, next, blocked, requests from other lanes.

## 2026-11-22
- M1: both recovery scoreboards gate again (`#[ignore]` removed; they run in `scripts/check.sh --fast`). `cargo test -p cpl-cli --test recovery -- --nocapture`: broken ports 396 copies, 0 silent, 1 cascaded (0.25%), 100.0% first-right, 78.6% fix-it; common mistakes 102 copies, 1 cascaded, 99.0% fix-it, 96.1% restored. Bars unchanged (99.5% first-right, 5% cascaded, 88% fixable, 95% restored).
- Benchmark base: ports with errors before the edit are skipped (decisions.md 216). 5 of the 37 ports are unclean under the full check: egoi22p3, ioi15p6, ioi18p1, ioi19p5, ioi19p6 (request in the compiler worklog). On all 37 the broken-ports board also passes (483 copies, 1 cascaded, 100.0% first-right, 79.1% fix-it); restored is 88.5% (115/128) because no fix restores an unclean port.
- Landed devin/wip-diag-recovery, rebased on main, plus: `Diagnostic::suggest` (one did-you-mean fix, replacing 8 copies), E0203 spec-member typo fixes, a grader fn whose body lost its `:` is one E0103 (was E0006 plus cascades), cpl-fmt keeps `x !in xs` (note in the lint-fmt worklog), member_typo moved to `crates/cpl-ir/src/lower/typo.rs` so no ratcheted file grew. Rebased branch alone, before these: 98.6% first-right, 34 cascaded / 483, 73.7% fix-it; common mistakes 72.3% fix-it.
- Review packet (README, PLAN, ARCHITECTURE, metrics.toml). Measured on main 4b8f504f: recovery scoreboards fail and are ignored (98.6% first-right, 8.9% cascaded, 59.2% fixable). Next (phase 2): rebase and land devin/wip-diag-recovery, re-measure.
- Request to compiler: replace 11 raw `cpl_diag::Code("E…")` literals with `codes::` constants (see ARCHITECTURE.md debt 2).
- Lane opened.
