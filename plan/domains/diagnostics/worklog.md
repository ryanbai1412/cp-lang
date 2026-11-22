# diagnostics worklog

Newest first: what landed (commit), evidence, next, blocked, requests from other lanes.

## 2026-11-22
- Review packet (README, PLAN, ARCHITECTURE, metrics.toml). Measured on main 4b8f504f: recovery scoreboards fail and are ignored (98.6% first-right, 8.9% cascaded, 59.2% fixable). Next (phase 2): rebase and land devin/wip-diag-recovery, re-measure.
- Request to compiler: replace 11 raw `cpl_diag::Code("E…")` literals with `codes::` constants (see ARCHITECTURE.md debt 2).
- Lane opened.
