# debugger plan

The debugger's time-bound plan; the charter is [handbook/domains/debugger.md](../../../handbook/domains/debugger.md). Numbers are from [metrics.toml](metrics.toml) (2026-11-22, main `4b8f504f`).

## Milestone 1

Due about 09:30 PST 11/23, on edge; checkpoints every 2 h in the [worklog](worklog.md). What [MILESTONE-1](../../../handbook/product/milestones.md) needs from this domain: the loop **WA → `cpl stress` with a brute in the same file → minimized counterexample in seconds → fix → rerun**, smooth from the CLI, Neovim/Vim, VS Code and JetBrains, on Linux, macOS and Windows. Exit criteria:
- [x] The loop passes end to end on the CLI for `@alt(program)`, `@check` and `@interactor` multitests (`cargo nextest run -p cpl-cli --test debugger`: `stress_loop` tests).
- [x] Every seeded bug (`bench/stress/`, 29) found and shrunk; median < 1 s, max < 5 s (`cargo nextest run -p cpl-cli --test seeded_bugs --no-capture`: median 0.24 s, 11 of 11 P0 classes).
- [x] A cold run on 3 problems written fresh for it (`bench/stress/cold-*.cp`: max subarray, pair sums, increasing runs): each WA found and shrunk to a minimal case in 0.54-0.70 s, `cpl test` fails on it, and after the one-line fix `cpl test` and `cpl stress` pass (about 1 s). Run by debugger, not a stranger; the stranger run is Milestone 2.
- [x] The loop tests pass on Linux, macOS and Windows CI: paths, CRLF in saved cases, no `g++` or `sh` needed for a same-file brute (`.github/workflows/debugger-loop.yml`: 41 of 41 on each OS).
- [x] Neovim (headless loop `editors/neovim/test/stress_loop.lua` green on ubuntu, macOS and Windows in `debugger-loop.yml`; Vim's `stress_loop_vim.sh` green on ubuntu and macOS): `:Cpl stress` (or the LSP Stress lens) then `:Cpl test` runs the loop from the documented one-step setup (`editors/neovim/plugin/cplang.lua`); Vim: `:!cpl stress %` then `:!cpl test %`, documented. Tools owns `editors/` and the LSP; debugger writes the scenario.
- [x] JetBrains (`CpPluginSmokeTest` stress-loop test, headless IDE, green on ubuntu, macOS and Windows in `debugger-loop.yml`): a `cpl` run configuration with command `stress`, then `test` (`editors/jetbrains/src/main/kotlin/dev/cplang/ide/CplRun.kt`), runs the loop; its output names the saved case.
- [x] VS Code (loop test in `editors/vscode/test/integration/suite.ts` passes on ubuntu, macOS and Windows in `debugger-loop.yml`): Stress lens shows the failing case, Run All Samples reruns `<file>.tests/`; wording is v0.3's (`@alt(program)`, `@check`).
- [x] Traps name the rule, the place and the locals (all 12 R-codes in `runtime_errors.rs`).
- [x] Stress guide (`web/content/docs/guides/stress.mdx`, site's) teaches the whole loop including the rerun (section "The loop"); its examples run (they do today).

Milestone 1.5, started the moment the loop above works on all 3 OSes and in all 3 editors: replay (time travel, the +20% trace target) per [design/trace-replay.md](design/trace-replay.md), needing compiler's Interp snapshot/restore. Deferred past both: the `crates/cpl-trace` split, the VM in the browser, `gen<T>` generators beyond a row, DAP and viewer polish. What works stays working.

Every criterion above passes, so Milestone 1.5 has started.

Next:
1. Milestone 1.5: replay. Time travel already runs in every editor under test: VS Code and nvim-dap step back through `cpl dap` (`suite.ts`, `dap_loop.lua`), JetBrains through the trace view (`CpPluginSmokeTest`). Every program shape (single, multitest, interactive) now takes checkpoints on the VM, and each reader replays from them: `cpl run --trace` windows (`--from`, `--last`), `--live` windows, a trap's locals, and `cpl dap` past its first 10^7 steps (`dap_reaches_every_step_of_a_run_longer_than_a_recording`). Left: checkpoints inside a fed loop body (tech-debt), and the +20% bar on `--full` (`trace_budget`).
2. Done: with no C++ compiler, `cpl stress` runs the file's `@gen` fn in the interpreter, which writes down and replays its `rand`/`shuffle` choices as the C++ does, so its inputs still shrink (`cpl_stress_interprets_a_gen_fn_where_no_cpp_compiler_is`). Next: the divergence report (Milestone 2).

## Milestone 2

The most ambitious complete version ([MILESTONE-2](../../../handbook/product/milestones.md)): any doubt becomes evidence in seconds, in every editor and OS, for strangers. Exit criteria:
- [ ] Time travel: any step of any run (single, multitest, interactive) is reachable backwards and forwards in the CLI, the viewer and DAP (VS Code, JetBrains, nvim-dap), via replay; trace overhead at most +20% over the interpreter, which is itself within 5x of C++ (`trace_budget` test, ungated today).
- [ ] Inputs of any structure are generated and shrunk: trees, graphs, strings, permutations, queries, via `gen<T>` for user types with no std privilege (`core-06/gen-*` 6 of 6 conformance).
- [ ] "Where do they first disagree": stress reports the first diverging line and value between solution and `@alt`, not just outputs. Today: `cpl stress --explain` names the first differing value (`@alt(program)`) or the first call of a part its `@alt(f)` answers differently, with its arguments, innermost in a recursion, and the solution's last own steps in it with their locals (`stress_explain_names_the_first_call_of_a_part_its_alt_answers_differently`, `..._innermost_wrong_call_of_a_recursive_part`); not yet a line-by-line match against the alt's steps, which have no common lines.
- [x] Seeded bugs cover all 11 P0 classes, every one found and shrunk, median < 1 s (29 bugs, median 0.24 s; `io-flush` is `guess-unread-verdict`, a judge verdict left unread).
- [ ] Traces stream in `crates/cpl-trace`; the viewer runs in the browser playground on the same VM; one vocabulary across trace, errors and spec.
- [ ] A cold stranger debugs 3 planted bugs from the docs alone, in every editor.
Next steps: the divergence report. `--explain` blames the planted line of 17 of the 29 seeded bugs (`explain_names_the_line_each_bug_was_planted_on`; each bug's planted line ends in `// planted`); a wrong call now names where it made its value, the lines that last changed what that line reads, and a global a multitest leak carries. Open: `@alt(program)` divergences (blame the line that made the first wrong output), calls with no own steps (`noip01p2`, `noip18p2`), and bugs upstream of the value (`dpk`, `noip04p2`, `noip10p1`, `noip14p2`, `nccc10s2`).

## Milestone 1 order

Milestone 1 ([exit criteria](#milestone-1)) sets the order, in hours: due about 09:30 PST 11/23.
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
- **Keep two engines (tree for trace, VM for run).** Rejected: two semantics to keep equal, and `handbook/design/architecture.md` already says one interpreter.
- **Print every local on a trap.** Rejected: noisy in a 200-line DP; name the locals in the failing expression, then the frame, then "more in the trace".
- **Move the viewer into the site's React app.** Rejected: the CLI and VS Code need it offline without a build; see decision 3.

## Risks

- The raw-cell and VM-trace branches touch `crates/cpl-interp`, which the compiler domain is also changing; rebase conflicts are likely (the trace notes already record two). Mitigation: land early, small, with a compiler worklog note.
- The 3x bar may be unreachable against an ever-faster VM: observed-only hooks alone are 3.98x. Decision 1 settles what we measure.
- `@brute` -> `@alt(program)` touches corpus and bench files other domains own; the stress-gen-alt branch already did the migration, so it needs a careful rebase, not a redo.
- Screenshots go stale silently; the browser test in workstream 5 is what keeps them honest.

## First

Item 2 of the Milestone 1 order: the loop tests on macOS and Windows.

## General fixes owed

Each local fix made so far, and the mechanism that would remove its whole class.

| Instance (local fix) | General fix | Owner |
|---|---|---|
| A traced run of a fn with a `mut` param runs on the tree-walker (perf: `eval`, `for_loop`), so counting its steps is 8.7x untraced (1951 vs 225 ms) | Done (tech-debt 345573204): traced chunks compile fns with `mut` params; the count is 428 vs 170 ms untraced (2.5x) | tech-debt |
| A trace counted every item of a long list at each step it showed it (`cells` takes one per item of a collection of numbers, a02eccaf5) | The trace document repeats every variable's whole value at every step: a list-filling loop writes 51 MB for 5,872 steps and hits the byte budget. Send each step's changes (the pushed item, the written cell) and let the viewer rebuild values, as the log already does; recording pushes as items alone moved the cost to writing the document (tried and reverted) | debugger |
| Windows: stress JSON `expected`/`got` carried `\r\n` (`crates/cpl-stress/src/cli.rs` normalizes at serialization) | Normalize line endings once where `crates/cpl-stress/src/program.rs` captures a program's output, so every consumer (compare, report, saved `.out`) sees LF | debugger: done, `Outcome` is LF |
| Windows: `runtime_errors.rs` strips both `dir/` and `dir\` from diagnostics | Diagnostics render a path relative to the cwd with `/` on every OS, so goldens never see the platform separator | diagnostics |
| Windows: tests planting bugs by `\n` patterns missed CRLF checkouts (`debugger.rs` reads LF) | `.gitattributes` `*.cp text eol=lf`, so checkouts never carry CRLF into sources | infra |
| LSP Stress lens matched decorator names as substrings and missed v0.3's `@alt`/`@check` (`crates/cpl-lsp/src/server.rs`) | The lens asks the parsed file for its stress parts (the same part discovery `cpl stress` uses), so a vocabulary change cannot drift between them | tools |
| No `g++` on the PATH: same-file stress falls back to `cpl run` (`program.rs`), but `@gen` still needs the compiler | Every stress part, `@gen` included, runs through one backend choice (compiled when a compiler exists, interpreted otherwise) | debugger |
| Neovim test on Windows: `glob` missed the saved case on a mixed-separator temp path (now `readdir`) | Editor tests find saved cases by listing `<file>.tests/`, never by glob; one helper per editor test suite | tools |
| JetBrains test compared `File.path` strings and read a temp folder outside the VFS roots (`CpPluginSmokeTest`, now canonical paths) | One test helper makes a VFS-visible temp project and compares paths as files | tools |
| VS Code: a failing debug test left its session open and broke the next test | Each integration test runs in a fresh state: the runner stops debug sessions and closes editors between tests | tools |
| A solution that hangs hung `cpl stress` after the time limit: the failing case is rerun in the debug interpreter, which has no time limit (`crates/cpl-stress/src/cli.rs` skips that rerun on TLE) | Done: one `cpl_interp::Stop` (`Io::until`), looked at every VM back edge and call and every tree-walker block, traced or not; `cpl test` (`run_for`), `cpl stress`, DAP, `--explain`, `cpl run --time-limit` and cpl-conform use it | compiler |
| A `@test` fn that never ends hung `cpl test` | `cpl_interp::run_for`, at the judge profile's time limit | debugger (done) |
