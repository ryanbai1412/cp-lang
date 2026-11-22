# debugger worklog

Newest first: what landed (commit), evidence, next, blocked, requests from other lanes.

## 2026-11-22
- Landed `cli: run the debugging demos and dap tests as v0.3` (from `devin/wip-v03-debug-demos`, rebased; `V.first!` spelling as on main, dropped the dead `CPL_EDITION` env removals). New test `every_debugging_demo_checks_clean`. Evidence: `cpl check examples/debugging/shortest_path.cp` no errors; `cargo test --release -p cpl-cli --test replay --test dap` 5 + 3 pass. The 10^7 budget test now gets past the demos and fails on overhead only: scalar 8.48x, array 9.69x untraced (1.09 s / 1.65 s recorded), shortest_path demo 1.74x.
- Review packet: README, PLAN, ARCHITECTURE, metrics.toml, `probes/prefix_trap.cp`; screenshots recaptured in `examples/debugging/screenshots/` (knapsack, shows, trap).
- Evidence: every metric in `metrics.toml` run on main `4b8f504f`. The 10^7 budget test fails: `examples/debugging/shortest_path.cp` E0301 on v0.3.
- Next (phase 2): land the in-flight branches, starting with `devin/wip-v03-debug-demos`.
- Lane opened.
