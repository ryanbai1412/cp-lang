# debugger worklog

Newest first: what landed (commit), evidence, next, blocked, requests from other lanes.

## 2026-11-22
- Landed `tools: viewer names the cpl that wrote it, not 0.2.2`: `tools/trace/index.html` reads its version from a `trace-version` element that `cpl` fills (`crate::VERSION`, asserted in `run_trace_writes_the_schema_as_json_and_as_one_page`); the versus note says "its brute" instead of `@brute(main)` (CLI and WASM). `shortest_path` screenshot recaptured on v0.3. Evidence: viewer node tests 39 + 23 pass, `test_emit.py` OK. Seen while capturing (open): the heap push on line 23 is shown at line 18; In scope names the lowered fn `Heap_Tup2_Int_P0_10.pop`; Changed this step writes `dist[1]` for vertex 2.
- Landed `cli: run the debugging demos and dap tests as v0.3` (from `devin/wip-v03-debug-demos`, rebased; `V.first!` spelling as on main, dropped the dead `CPL_EDITION` env removals). New test `every_debugging_demo_checks_clean`. Evidence: `cpl check examples/debugging/shortest_path.cp` no errors; `cargo test --release -p cpl-cli --test replay --test dap` 5 + 3 pass. The 10^7 budget test now gets past the demos and fails on overhead only: scalar 8.48x, array 9.69x untraced (1.09 s / 1.65 s recorded), shortest_path demo 1.74x.
- Request from tech-debt (cold review round 1 of the packet, ba59fe62; rubric right home 4, one way 3, local next change 3, legible 4, finished migration 3). Every path in the module and interface tables exists; `tests/trace_engines.rs` is `crates/cpl-cli/tests/trace_engines.rs`.
  1. `cpl-trace` sequencing: the hooks live in the tree-walker today (`crates/cpl-interp/src/lib.rs` 394, 627, 772-775). Moving `trace.rs`/`trace/log.rs`/`replay.rs` (4186 lines) out before the VM emits the events would wire the same hooks into two engines. Proposal: compiler defines an observer trait in `cpl-interp` (declared / changed / print / call / case) that the VM emits; `cpl-trace` depends on `cpl-ir` and that trait, never the reverse; `CPL_TRACE_ENGINE` and `trace_engines.rs` go with the tree-walker (compiler M3). Ask compiler for the trait in `plan/domains/compiler/worklog.md`.
  2. `cpl-stress` as a library for `cpl-contest` (`cpl-contest -> cpl-stress` is a shipped edge): name the shared part a public module with a stated contract (`cpl_stress::gen`) rather than a new crate, so the crate graph does not grow (new shipped edges fail `debt-ratchet.py`).
  3. `crates/cpl-stress/src/spec.rs` (2911) is on the file-size allowlist, which only shrinks; "split by shape when touched" matches it.
- Review packet: README, PLAN, ARCHITECTURE, metrics.toml, `probes/prefix_trap.cp`; screenshots recaptured in `examples/debugging/screenshots/` (knapsack, shows, trap).
- Evidence: every metric in `metrics.toml` run on main `4b8f504f`. The 10^7 budget test fails: `examples/debugging/shortest_path.cp` E0301 on v0.3.
- Next (phase 2): land the in-flight branches, starting with `devin/wip-v03-debug-demos`.
- Lane opened.
