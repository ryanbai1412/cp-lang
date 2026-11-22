# debugger worklog

Newest first: what landed (commit), evidence, next, blocked, requests from other lanes.

## 2026-11-22
- Review packet: README, PLAN, ARCHITECTURE, metrics.toml, `probes/prefix_trap.cp`; screenshots recaptured in `examples/debugging/screenshots/` (knapsack, shows, trap).
- Evidence: every metric in `metrics.toml` run on main `4b8f504f`. The 10^7 budget test fails: `examples/debugging/shortest_path.cp` E0301 on v0.3.
- Next (phase 2): land the in-flight branches, starting with `devin/wip-v03-debug-demos`.
- Lane opened.
