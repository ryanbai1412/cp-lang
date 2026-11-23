# debugger plan

The detail behind [README.md](README.md). Numbers are from [metrics.toml](metrics.toml) (2026-11-22, main `4b8f504f`).

## Milestone 1 order

Milestone 1 ([README](README.md#milestone-1)) sets the order, in hours: due about 09:30 PST 11/23.
1. **The stress loop, CLI.** Done: `stress_loop` tests in `crates/cpl-cli/tests/debugger.rs` for `@alt(program)`, `@check` and `@interactor`.
2. **Three platforms.** The loop tests and seeded bugs on macOS and Windows CI (request to infra for the runners); fix in `cpl-stress` what breaks: paths, CRLF in saved cases, the `sh` used to precompile `g++` headers (`crates/cpl-stress/src/program.rs`), which a same-file brute must not need.
3. **Neovim/Vim.** `:Cpl stress` / the LSP Stress lens, then `:Cpl test`, from the documented setup; `:!cpl stress %` / `:!cpl test %` for Vim. Tools owns `editors/` and the LSP: requests in their worklog, stress-side fixes here.
4. **VS Code and JetBrains.** JetBrains: the `cpl` run configuration with `stress` then `test`. VS Code: Stress lens, failing-case overlay, Run All Samples over `<file>.tests/`, v0.3 wording (requested in tools' worklog).
5. **The guide.** `stress.mdx` gains the rerun step (site's file: request); its examples stay runnable.
6. **Keep it green.** Seeded bugs and trap evidence stay in the gate.
7. **Milestone 1.5: replay.** Starts as soon as items 2 to 4 are done: record lines on the untraced build with snapshots, rebuild a window by restore and replay (workstream 4, `design/trace-replay.md`); the +20% target is its bar. Needs compiler's Interp snapshot/restore.
Deferred past both: the rest of workstream 4 (the split, the VM on wasm) and workstream 6 (generators beyond a row). Workstream 5 only for regressions.

## Workstreams

1. **Land and repair (M1).** Rebase and land the in-flight branches; delete the superseded ones with a `git cherry` note in the worklog.
   - `devin/wip-v03-debug-demos` (1 commit): the demos and DAP tests as v0.3; fixes the E0301 that fails the 10^7 budget test.
   - `devin/wip-stress-gen-alt` (5 commits): `gen(...)`/`gen<T>(...)`, `@alt(program)`/`@check` spellings with fix-its, inner `@alt(f)`, the corpus migration off `@brute`.
   - `devin/wip-trace-raw-cells` (2 commits): VM records quiet Int cell writes from registers; touches `crates/cpl-interp`, so it lands with a note in the compiler worklog.
   - `devin/wip-trace-notes`: 2 of its 5 commits are the raw-cell ones; the 3 plan commits are folded into the worklog, then the branch is deleted.
   - `devin/vm-trace` (2 commits, 366 behind): main already traces on the VM; expected superseded, checked with `git cherry` and a diff before deleting.
   Then: assert the seeded median (< 1 s) in `seeded_bugs.rs`; add the viewer node tests and a demo compile check to `scripts/check.sh --fast` (infra note).
2. **Trap evidence (M2).** The recording already holds the locals at the trap (the trace shows `i 3`). Print them in the terminal: the offending locals named in the failing expression, then the call path, in source terms. The diagnostics domain owns rendering; this lane supplies the values through a small `TrapContext` and asks diagnostics for the layout. One `runtime_errors` case per R01xx code. The viewer says "Trapped at step N" and jumps there.
3. **Seeded bugs by P0 class (M2).** Tag each of the 21 bugs with its P0 class; add bugs for the empty classes (I/O flushing, floating point, mutation surprises are the likely gaps); keep median < 1 s.
4. **Trace speed and the split (M3).** Ask the compiler domain for a narrow observer interface (declared / changed / call / return / print / case events, with a value view); move `trace.rs`, `trace/log.rs`, `replay.rs` into `crates/cpl-trace` behind it, with a contract test next to the core. Then delete `CPL_TRACE_ENGINE=tree`, swap `trace_engines.rs` for recorded goldens, and lift the `cfg(not(wasm32))` so the browser traces on the VM too.
5. **Viewer quality (M1 small, M3 browser test).** Version from the build, not `0.2.2`; v0.3 vocabulary; a headless-Chrome browser test of the three demos (first divergence visible, step back, trap step) in the gate.
6. **Generators beyond a row (after M2).** Graph, tree, grid and anti-hash generators through `gen<T>`; shrinking in the browser through `InterpEngine`.

## Order and why

- **M1 first** because main is not green for this domain: the budget test fails and a flagship demo doesn't compile. Landing the branches is also the cheapest progress: five reviewed-but-unlanded changes.
- **M2 before M3** because a trap that names its locals is felt on every wrong run, while trace overhead is felt only on long runs: the knapsack demo traces at 1.18x today. The spec already promises it (`spec/tools/trace-debug.md`, "Traps").
- **M3 last** because the split needs an interface from the compiler domain and the raw-cell work to settle; doing it first would move code twice.

## Alternatives considered

- **Split the crate first, then optimize.** Rejected for now: the hot hooks are inside the VM's dispatch; moving them before the observer interface exists either copies VM internals or adds a dynamic call per step.
- **Keep two engines (tree for trace, VM for run).** Rejected: two semantics to keep equal, and `plan/ARCHITECTURE.md` already says one interpreter.
- **Print every local on a trap.** Rejected: noisy in a 200-line DP; name the locals in the failing expression, then the frame, then "more in the trace".
- **Move the viewer into the site's React app.** Rejected: the CLI and VS Code need it offline without a build; see decision 3.

## Risks

- The raw-cell and VM-trace branches touch `crates/cpl-interp`, which the compiler domain is also changing; rebase conflicts are likely (the trace notes already record two). Mitigation: land early, small, with a compiler worklog note.
- The 3x bar may be unreachable against an ever-faster VM: observed-only hooks alone are 3.98x. Decision 1 settles what we measure.
- `@brute` -> `@alt(program)` touches corpus and bench files other domains own; the stress-gen-alt branch already did the migration, so it needs a careful rebase, not a redo.
- Screenshots go stale silently; the browser test in workstream 5 is what keeps them honest.

## First

Item 2 of the Milestone 1 order: the loop tests on macOS and Windows.
