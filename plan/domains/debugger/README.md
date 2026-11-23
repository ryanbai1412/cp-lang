# debugger

Segment: E. Experience. One page; detail in [PLAN.md](PLAN.md), code in [ARCHITECTURE.md](ARCHITECTURE.md), numbers in [metrics.toml](metrics.toml), log in [worklog.md](worklog.md).
Owns: `crates/cpl-stress`, the trace (today inside `crates/cpl-interp/src/trace*`, to become `crates/cpl-trace`), the viewer (today `tools/trace/`, planned `web/packages/trace-viewer`), `examples/debugging/`.
Status (2026-11-22): stress is fast and green; the trace works end to end but is 6.7-8.3x slower than an untraced run, the trap output in the terminal shows no locals, and one of the three v0.3 demos no longer compiles.

## Mission

Doubt to evidence: from "WA, and I don't know why" to a minimal counterexample, the first state that went wrong, and a trace you can scrub, in seconds, deterministically.
The loop Ryan reviews: **WA -> `cpl stress` -> minimal input -> trap or first divergence -> HTML trace / step back.**

## Covers / not covered

| Covers (debugger) | Not covered (owner) |
|---|---|
| `cpl stress`: typed-input generation, `@alt`/`@gen`/`@check`/`@interactor`, shrinking, seeds | The language rules for those decorators: `spec/lib/random-stress.md` (spec) |
| Trace recording, step back, replay of an edited call (`cpl run --trace`, `--live`, `cpl replay`) | The interpreter and VM the trace hooks into: `crates/cpl-interp` (compiler) |
| DAP time-travel debugger (`cpl dap`) and its behaviour | The VS Code extension that hosts it: `editors/` (tools) |
| The trace viewer (`tools/trace/index.html`) and its tests | Embedding it in the site and the playground UI: `web/` (site) |
| Trap evidence: which locals and call path a trap shows | Diagnostic wording and rendering: diagnostics |
| WASM trace/stress entry points in `crates/cpl-wasm/src/lib.rs` | The wasm build, size cap, deploy: site / infra |
| Seeded-bug and stress-budget benchmarks (`bench/stress/`) | `cpl profile`, `cpl bench`: perf / tools |

## Success criteria for v0.3

1. Every seeded bug, at least one per P0 class, is found and shrunk to its minimal input; median < 1 s, asserted in the gate.
2. Every trap prints the rule, the place, the offending locals and the call path in source terms (`spec/tools/trace-debug.md`).
3. Tracing a 10^7-step run stays inside the budget test: overhead bar (see decision 1), < 500 MB, step back < 50 ms.
4. The three `examples/debugging/` demos compile on v0.3 and run the whole loop; their screenshots are current.
5. The viewer has a browser test in `scripts/check.sh`; the playground's Trace works on every runnable docs example.
6. One trace engine: `CPL_TRACE_ENGINE=tree` and its parity test are gone, replaced by recorded goldens.

## Milestones

- **M1 Green loop on v0.3.** Exit: `ten_million_budget_test` no longer fails on compile; `debug_demos_compile` 3/3 with `@alt(program)`; in-flight branches landed or deleted (`git cherry` notes in the worklog); seeded median asserted; viewer unit tests in `scripts/check.sh --fast`.
- **M2 Traps show values.** Exit: `trap_terminal_locals` prints `i = 3` and the call path for `probes/prefix_trap.cp`; a `runtime_errors` case per trap code (R01xx) asserts the locals; the viewer header names a trapped run "Trapped at step N"; seeded bugs grown to cover every P0 class.
- **M3 Fast trace, own crate.** Exit: `crates/cpl-trace` exists with the recorder, log and replay, depending on a narrow observer interface from `cpl-interp` (contract test next to the core); the tree trace engine deleted; overhead at the decision-1 bar; viewer browser test in the gate.

## Metrics

All run 2026-11-22 on main `4b8f504f`, release build; commands in [metrics.toml](metrics.toml).

| Metric | Today | Target | M |
|---|---|---|---|
| Seeded bugs found + minimal | 21/21 | all, ≥ 1 per P0 class | M2 |
| Seeded median time to minimal | 0.72 s (max 1.29 s), not asserted | < 1 s, asserted | M1 |
| Stress budget problems | 3/3, max 0.60 s | all < 10 s | M1 |
| 10^7-step budget test | **FAIL** (demo E0301) | pass | M1 |
| Trace overhead vs untraced | 6.65x scalar, 8.28x array, 1.18x knapsack | ≤ 3x (decision 1) | M3 |
| Recording vs observed-only run | 1.67x | ≤ 1.5x | M3 |
| Trace memory at 10^7 steps | 230 / 361 MB | < 500 MB | M1 |
| Step back, worst | 20.5 / 74.7 µs | < 50 ms | M1 |
| VM vs tree trace parity | pass, 169 programs | replaced by goldens | M3 |
| Debugger suites (8 binaries) | pass, 47 tests | pass | all |
| `cpl-stress` unit | pass, 83 tests | pass | all |
| Viewer unit tests | 62/62 in gate; browser test of 3 demos, skipped without Chrome | in gate + browser test | M1/M3 |
| Debug demos compile | 2/3, all spell `@brute` | 3/3, `@alt(program)` | M1 |
| Locals printed with a trap | 0 | offending locals + call path | M2 |
| wasm gzip (trace+stress inside) | 1,761,196 B | ≤ 2,100,000 B | all |

## Judgment rubric

Numbers can't say whether the evidence is the evidence a contestant needed. Each question has a probe a cold agent attempts with only the docs and the CLI.

1. **Is the first screen the answer?** After `cpl stress --trace`, does the first view show the divergence and the line responsible, without scrolling? Probe: given `examples/debugging/knapsack.cp`, name the buggy line from the trace alone in ≤ 3 actions.
2. **Is the minimal input actually explanatory?** Probe: for 5 seeded bugs, the agent states the bug from the shrunk input plus trap/trace only, never reading the brute.
3. **Does a trap need a rerun?** Probe: `probes/prefix_trap.cp`; can the agent fix it from the terminal output alone, with no trace and no `debug(...)`?
4. **Is the loop guessable from the docs?** Probe: a fresh WA solution and no instructions beyond the site docs; does the agent reach `cpl stress` and a minimal input without help?
5. **Does it stay out of the way?** Probe: the agent adds a dimension to a DP that has an `@alt(program)` brute; does stress keep working with zero edits to the generator or the brute?

## Improve (highest impact first)

1. Fix the broken demo and land in-flight branches (`devin/wip-v03-debug-demos`, `devin/wip-stress-gen-alt`, `devin/wip-trace-raw-cells`); the budget test fails on main today.
2. Traps print locals and call path in the terminal; the data is already recorded (the trace shows `i 3` at the trap step).
3. Viewer: label trapped runs, drop the hard-coded `0.2.2` (`tools/trace/index.html` lines 7, 337, 855), say `@alt` not `@brute(main)`; put its tests and a browser test in the gate.
4. Trace overhead: raw cell writes and VM-native recording; observed-only hooks alone cost 3.98x today.
5. Split `crates/cpl-trace` out of `cpl-interp` behind an observer interface; delete the tree trace engine.
6. Generators for shapes a row can't say: graphs, trees, grids, anti-hash; shrinking in the browser.
7. Seeded bugs per P0 class, with the class recorded per bug.

## Decisions for Ryan

Filed in [plan/QUESTIONS.md](../../QUESTIONS.md):
1. What the trace overhead bar is measured against (untraced VM, observed run, or an absolute budget).
2. The command surface: the spec's `cpl trace` / `cpl debug` / `cpl snap` vs today's `cpl run --trace` / `cpl dap` / `cpl replay`.
3. Where the viewer lives: one self-contained `tools/trace/index.html`, or a built `web/packages/trace-viewer`.

## Ryan's role

Align on the loop above and on what generators must describe; answer the three questions; review rounds on screenshots and recordings of the loop (worklog). Current screens, recaptured 2026-11-22 from main with `cpl stress --trace` and `cpl run --trace`:

| ![knapsack](../../../examples/debugging/screenshots/knapsack-trace.png) | ![shows](../../../examples/debugging/screenshots/shows-trace.png) |
|---|---|
| ![trap](../../../examples/debugging/screenshots/trap-trace.png) | `shortest_path.cp`: not capturable (E0301 on v0.3); `vscode-step-back.png` not recaptured |
