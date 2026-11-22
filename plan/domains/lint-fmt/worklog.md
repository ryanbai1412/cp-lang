# lint-fmt worklog

Newest first: what landed (commit), evidence, next, blocked, requests from other lanes.

## 2026-11-22
- Phase 1 review packet: README, PLAN, metrics.toml, ARCHITECTURE,
  design/overview. Baseline (metrics.toml): 38 lints, 0 records on main;
  corpus `spec.cp` 50 warnings, `spec/std` 293; fmt changes 552 of 4,443
  tracked `.cp` files, 0 not idempotent, 1 meaning change. Next: phase 2
  starts by landing `devin/wip-fmt` (PLAN.md, "Phase 2, first").
- Requests filed: compiler (fact table, checker in wasm), idioms (who lands `spec/idioms/` from `devin/idioms-docs`).
- Lane opened.
