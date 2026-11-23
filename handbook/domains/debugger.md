# Debugger domain

| | |
|---|---|
| Kind | Domain charter |
| Owner | debugger |
| Describes | `crates/cpl-stress/`, `crates/cpl-interp/src/trace.rs`, `crates/cpl-interp/src/trace/`, `crates/cpl-interp/src/checkpoint.rs`, `crates/cpl-interp/src/replay.rs`, `crates/cpl-cli/src/trace.rs`, `crates/cpl-cli/src/live.rs`, `crates/cpl-cli/src/replay.rs`, `crates/cpl-cli/src/dap.rs`, `crates/cpl-cli/src/interact.rs`, `tools/trace/`, `bench/stress/`, `examples/debugging/` |

## Mission

Turn "wrong answer, and I don't know why" into evidence in seconds: a
minimal failing input, the first state that went wrong and the line that
made it, and a trace you can step through backwards and forwards. The same
evidence comes from the CLI, every supported editor and the browser, on
Linux, macOS and Windows, and the same seed always gives the same evidence.

Example: a contestant's answer to a merging problem is wrong. They add a
slow version of the same function marked `@brute(effort)` and run
`cpl stress effort.cp --explain`. Within a second it prints a three-number
input on which the two disagree, saves it as a test, and says
`look at line 8: heap.pop() at line 12 takes the end that the MaxHeap
declared here keeps`. `cpl stress --trace` opens the run in the viewer at
that step, and `cpl dap` lets their editor step back from it.

The loop the domain is judged on: **wrong answer → `cpl stress` → minimal
input → trap or first divergence → trace / step back → edit → rerun.**

## Scope

| Owns | Does not own (owner) |
|---|---|
| `cpl stress`: input generation from typed `read()`s, `@gen` generators, `@alt` and `@brute` comparison, `@check` checkers, `@interactor` judges, shrinking, seeds, saved cases (`crates/cpl-stress`) | What those decorators mean in the language (`spec/lib/random-stress.md`, spec) |
| `cpl stress --explain`: the first divergence and the lines it blames | The interpreter and VM the trace hooks into (`crates/cpl-interp` outside the trace files, compiler) |
| Trace recording, the step log, checkpoints, windows, replay of an edited call (`cpl run --trace`, `--live`, `cpl replay`) | Moving constructs onto the VM (compiler, tech-debt); the debugger reports what the trace still needs from the tree-walker |
| The time-travel debug adapter `cpl dap` and how it behaves | Editor extensions that host it (`editors/`, tools) |
| The trace viewer `tools/trace/index.html`, its schema and tests | Embedding the viewer in the site and playground (`web/`, site) |
| Which values and call path a runtime trap shows | Trap wording and rendering (diagnostics) |
| Trace and stress entry points in `crates/cpl-wasm/src/lib.rs` | The wasm build and its size cap (site, infra) |
| Seeded-bug and stress-budget benchmarks (`bench/stress/`), the debugging demos (`examples/debugging/`) | `cpl bench`, `cpl profile` (perf, tools); `cpl test` sample runs (tools) |
| The saved traces of `crates/cpl-cli/tests/programs/*.trace` as the trace oracle | The programs' expected output `*.out` (whoever adds the program) |

## Interfaces

**Provides**

| Interface | Used by | Held by |
|---|---|---|
| `cpl_interp::observe_on_this_thread(program, io, output, limits, trace)`: the one entry every trace, live run and stress run goes through; `checkpoint::observe_on_this_thread` is the same entry plus a `Checkpoints` request | `crates/cpl-cli`, `crates/cpl-stress`, `crates/cpl-wasm` | `crates/cpl-interp/tests/checkpoint.rs`, `crates/cpl-cli/tests/trace_goldens.rs` |
| Trace JSON, schema `tools/trace/trace-schema.json` | the viewer, the VS Code trace view (tools), `web/scripts/gen-debug.mjs` (site) | `tools/trace/trace.test.cjs` |
| `tools/trace/index.html`, one offline file | `cpl run --trace` (inlined), the VS Code webview, the site | `crates/cpl-cli/tests/trace_browser.rs` |
| `cpl dap` over stdio (Debug Adapter Protocol, with step back) | VS Code, nvim-dap, JetBrains (tools) | `crates/cpl-cli/tests/dap.rs`, `.github/workflows/debugger-loop.yml` |
| `cpl stress` flags, exit codes, saved cases in `<file>.tests/`, `--format json` | editors' Stress lens (tools), the stress guide (site) | `crates/cpl-cli/tests/debugger.rs` |
| `cpl_stress::{InputSpec, generate, shrink, Rng, ...}` | `cpl bench` (`crates/cpl-contest`), site's lint examples (`sample` in the wasm build) | `crates/cpl-stress/src/tests.rs` |

**Needs**

| From | What |
|---|---|
| compiler | Every construct running on the VM with trace hooks, including fed-loop bodies, so traces and checkpoints never need the tree-walker; `Program.products` and type names for showing values in source terms |
| tech-debt | Rebuilding VM frames from a checkpoint (resume inside any frame) |
| stdlib | A general way to show a library type in source terms (an `impl Show`), not a trace special case per type |
| diagnostics | Stable runtime error codes (`R01xx`) and rule names |
| tools | Editors that pass `cpl dap` requests through and run `cpl stress`/`cpl test` from the editor |
| infra | CI on all three OSes running `.github/workflows/debugger-loop.yml`; the gate running the suites below |

## Quality register

Each row is a quality a user can feel, the number that measures it, and the
strongest mechanism that enforces it. Mechanisms are ranked: a design that
makes the failure impossible, then a generated, property, differential,
metamorphic or mutation suite, then a hand-written suite, then a lint,
ratchet or gate, then a process. `missing` marks a row with no real check
yet. Targets and current values are in `plan/domains/debugger/metrics.toml`.

The rows follow the journey from a failing submission to a fix
(`handbook/product/journeys.md`, J6 and J7), then the qualities that cut
across it. The yardsticks are how mature tools measure the same thing:
QuickCheck and Hypothesis for generation and shrinking, rr and Pernosco
for time travel, gdb and the DAP test suites for debuggers, rustc's UI
tests for wording.

### Finding the bug

| Quality | Metric | Mechanism | Where |
|---|---|---|---|
| Stress finds planted bugs of every P0 class | seeded bugs found, per class | hand-written suite (29 bugs, 11 classes) | `crates/cpl-cli/tests/seeded_bugs.rs` `stress_finds_every_seeded_bug_and_shrinks_it_to_the_minimal_input` |
| Stress finds bugs nobody planted by hand | share of generated mutants of corpus solutions that stress kills | `missing`: a mutation suite over `research/` ports with their brutes | — |
| A bug is found fast | median and max seconds to a minimal case | budget script on three problems; the seeded median is printed, `missing` as an assertion | `bench/stress/budget.py`, `seeded_bugs.rs` |
| Generated inputs satisfy their declared types | invalid cases per spec shape | property tests per shape | `crates/cpl-stress/src/tests.rs` (`cases_are_valid_small_and_the_same_for_the_same_seed`, `simple_connected_and_acyclic_graphs_are_generated_valid`, ...) |
| Every generated input is one the program's own reader accepts | cases the program rejects on read | `missing` as a generated suite: checked only for 10 corpus problems | `crates/cpl-cli/tests/stress_corpus.rs` |
| Every readable input shape can be generated | `read()` types stress can't make, reported where they are | hand-written suite; `missing`: a sweep over every corpus and conformance program | `tests.rs` `types_the_generator_cannot_make_are_reported_where_they_are` |
| Inputs cover the edges (empty, one, max, equal values, paths and stars) | edge shapes reached per spec | property tests for trees; `missing` for others | `tests.rs` `random_trees_include_paths_and_stars`, `full_size_cases_use_the_whole_range` |
| `@gen` generators work without a C++ compiler and still shrink | seeded bugs shrunk with no `g++` on PATH | hand-written suite | `stress_corpus.rs` `cpl_stress_interprets_a_gen_fn_where_no_cpp_compiler_is` |
| A wrong brute or a crashing generator is blamed, not the solution | misattributed failures | hand-written suite | `tests.rs` `a_failing_brute_stops_the_run_with_the_input` |
| Multitest, checker and interactive problems stress as easily as plain ones | loop tests per problem kind | hand-written suite | `crates/cpl-cli/tests/debugger.rs` (the tests built on its `stress_loop` helper) |
| Parallel stress gives the same answer for any job count | differing results across `--jobs` | property test | `tests.rs` `stress_stops_at_the_first_failure_shrinks_it_and_is_the_same_for_any_jobs` |

### Making it small

| Quality | Metric | Mechanism | Where |
|---|---|---|---|
| The counterexample is minimal | seeded bugs shrunk to their known minimal input | hand-written suite | `seeded_bugs.rs` |
| Shrinking never returns a passing or invalid case | invalid or passing results | property tests | `tests.rs` `shrinking_respects_its_budget_and_never_returns_a_passing_case`, `shrinking_keeps_graphs_simple_connected_and_acyclic` |
| A multitest failure shrinks to the tests it needs | tests left after shrinking | hand-written suite | `tests.rs` `a_bug_that_needs_two_tests_shrinks_both_of_them` |
| Shrinking is local-minimal in general, not just on seeded bugs | cases where one more delete or decrement still fails | `missing`: a property check that re-shrinks every result one step | — |

### Explaining it

| Quality | Metric | Mechanism | Where |
|---|---|---|---|
| `--explain` names the line the bug is on | seeded bugs whose planted line is named; only goes up | ratchet over the seeded suite | `seeded_bugs.rs` `explain_names_the_line_each_bug_was_planted_on` |
| `--explain` names the right line for bugs it wasn't tuned on | named share over mutants not in the seeded set | `missing`: shares the mutation suite above | — |
| The first divergence is the first one | divergences reported later than the first differing value | hand-written suite | `debugger.rs` `stress_explain_*` |
| An interactive failure says what the solution misread | interactive bugs explained by the read that went wrong | `missing`: the report names the rejected answer only | — |
| A trap names the rule, the place, the locals and the call path | trap codes with a golden that shows locals | golden suite per code | `crates/cpl-cli/tests/runtime_errors.rs` |
| A trap in the interpreter traps in the judge build too | traps that only one build raises | differential suite | `crates/cpl-cli/tests/trap_parity.rs` |
| Explanations use the program's words: variable names, domain parts, `Subset` members | values shown as internals (ordinals, struct fields of std types) | saved traces; `missing`: a check that no std-internal field reaches a user trace | `trace_goldens.rs` |

### Seeing the run

| Quality | Metric | Mechanism | Where |
|---|---|---|---|
| Tracing never changes what the program does | programs whose traced output differs from untraced | `missing` as a sweep over every program; spot checks only | `crates/cpl-interp/tests/trace_log.rs` `an_untraced_run_keeps_the_last_values_a_traced_one_does` |
| A trace shows the program's real state at every step | values that differ from the state the run had | debug assertion in the tracer ("changed unseen"), saved traces | `crates/cpl-interp/src/trace.rs`, `trace_goldens.rs` |
| A trace's state changes only when it should | unexplained changes between saved traces | saved traces as the oracle; edits to expected output are flagged for review | `trace_goldens.rs` |
| No saved trace records a crash of `cpl` itself | saved traces holding an internal error | gate in the trace suite | `trace_goldens.rs` |
| Any step rebuilt alone equals the step the run took | mismatches | property test | `trace_log.rs` `any_step_rebuilt_alone_is_the_step_the_run_took` |
| A run resumed from a checkpoint equals the whole run there | differing steps, output or input | differential tests over hand-picked programs; `missing` over every saved-trace program | `crates/cpl-interp/tests/checkpoint.rs` |
| Traces run on the VM, not the tree-walker | tree-walker fallbacks by kind; only goes down | ratchet | `crates/cpl-cli/tests/vm_fallbacks.rs` |
| Large values stay visible | values over the trace's size cap shown as `[]` | `missing`: arrays over 20000 cells are drawn empty | — |
| Std code stays out of a user's trace | steps inside `std` functions by default | `missing` | — |
| Huge traces can be narrowed to what matters | filters by fn, test, watch, depth, window | hand-written suite | `crates/cpl-cli/tests/trace_filters.rs` |
| Tracing is cheap enough to leave on | traced vs untraced time on 10^7 steps (target at most 1.2x); memory; worst step back | budget test, run by hand; `missing` as a gate | `trace_log.rs` `ten_million_steps_fit_the_debugging_budgets` (ignored) |
| Tracing off costs nothing | untraced time before and after the observer | design: untraced runs use a no-op observer | `crates/cpl-interp/src/lib.rs` |

### Time travel and the editor

| Quality | Metric | Mechanism | Where |
|---|---|---|---|
| Every step of any run is reachable forwards and backwards | steps unreachable in DAP, viewer or CLI windows | hand-written suite including runs past the recording cap | `crates/cpl-cli/tests/dap.rs` `dap_reaches_every_step_of_a_run_longer_than_a_recording`, `trace_filters.rs` |
| Step back, breakpoints, watches and stack follow DAP | DAP requests answered wrongly | hand-written protocol suite; `missing`: a generated sequence of random DAP requests checked against a straight run | `dap.rs` |
| Restart after an edit keeps your place | restarts landing on another step | hand-written suite | `dap.rs` `dap_restart_rerecords_the_edit_on_the_pinned_input_and_keeps_place` |
| Replaying one call with an edited fn equals rerunning it | differing results | hand-written suite | `crates/cpl-cli/tests/replay.rs`, `dap.rs` `dap_replays_one_call_with_the_edited_fn` |
| The loop works in Neovim, Vim, VS Code and JetBrains on Linux, macOS and Windows | editor x OS cells green | CI matrix | `.github/workflows/debugger-loop.yml` |
| A live trace follows each save | saves not re-recorded | hand-written suite | `crates/cpl-cli/tests/trace_live.rs` |

### The viewer

| Quality | Metric | Mechanism | Where |
|---|---|---|---|
| The viewer reads every trace the recorder writes | traces failing the schema | unit tests against the schema | `tools/trace/trace.test.cjs`, `tools/trace/state.test.cjs` (in `scripts/check.sh`) |
| The first screen shows the answer: the divergence or the trap | demos opening elsewhere | browser test | `crates/cpl-cli/tests/trace_browser.rs` (skipped without Chrome) |
| The viewer works offline from one file | external requests | design: one file, everything inlined | `tools/trace/index.html` |
| The viewer stays fast on large traces | time to first paint and to scrub at 10^6 steps | `missing` | — |
| The viewer in the playground matches the CLI's | traces differing between wasm and native | `missing`: the browser still traces on the tree-walker | — |

### Across the domain

| Quality | Metric | Mechanism | Where |
|---|---|---|---|
| Deterministic: one seed, one result, on every machine | differing cases, shrinks or traces across OSes and runs | property test for jobs; CI on three OSes; saved traces | `tests.rs`, `debugger-loop.yml`, `trace_goldens.rs` |
| Paths, CRLF and missing tools don't break the loop on Windows | loop tests failing on one OS only | CI matrix | `debugger-loop.yml` |
| The demos and guide examples keep running | demos that fail to check or run | hand-written suite | `replay.rs` `every_debugging_demo_checks_clean`, `trace_browser.rs` |
| The suites catch real debugger bugs | mutation score of the debugger suites | `missing` | — |
| Every escaped bug becomes a test | escaped bugs with no regression test | process: each fix lands with its test | `plan/domains/debugger/worklog.md` |
| A stranger can debug from the docs alone | planted bugs a cold agent fixes from the docs, in each editor | process: cold-stranger probe | `plan/domains/debugger/PLAN.md` |
| Handbook and guide text is written for an outsider | context-dependent phrases | lint | `scripts/check-handbook.py` |

## Suites and tools

| Suite | What it catches | Run it |
|---|---|---|
| `crates/cpl-cli/tests/seeded_bugs.rs` | A seeded bug not found, not minimal, slow, or not named by `--explain` | `cargo nextest run -p cpl-cli --test seeded_bugs` |
| `crates/cpl-stress/src/tests.rs` | Invalid generation, bad shrinking, judges, interactors, jobs | `cargo nextest run -p cpl-stress` |
| `crates/cpl-cli/tests/debugger.rs`, `stress_corpus.rs`, `stress_bench.rs` | The stress loop end to end, `--explain` JSON, corpus problems | `cargo nextest run -p cpl-cli --test debugger --test stress_corpus --test stress_bench` |
| `crates/cpl-cli/tests/trace_goldens.rs` | Any change to a saved trace; a saved crash; a program with no saved trace | `cargo nextest run -p cpl-cli --test trace_goldens`; `CPL_BLESS=1` records |
| `crates/cpl-interp/tests/trace_log.rs`, `checkpoint.rs`, `stop.rs` | Step rebuild, windows, checkpoints and resume, time limits; the 10^7-step budget (ignored) | `cargo nextest run -p cpl-interp`; budget: `cargo test --release -p cpl-interp --test trace_log ten_million -- --ignored --nocapture` |
| `crates/cpl-cli/tests/vm_fallbacks.rs` | A new tree-walker fallback in traced runs | `cargo nextest run -p cpl-cli --test vm_fallbacks` |
| `crates/cpl-cli/tests/dap.rs`, `replay.rs`, `trace_live.rs`, `trace_filters.rs` | The debug adapter, replay, live traces, filters and windows | `cargo nextest run -p cpl-cli --test dap --test replay --test trace_live --test trace_filters` |
| `crates/cpl-cli/tests/runtime_errors.rs`, `trap_parity.rs` | Trap reports and their locals; traps one build misses | `cargo nextest run -p cpl-cli --test runtime_errors --test trap_parity` |
| `tools/trace/*.test.cjs`, `crates/cpl-cli/tests/trace_browser.rs` | Viewer logic, schema, and what a browser shows | `node tools/trace/trace.test.cjs`; `cargo nextest run -p cpl-cli --test trace_browser` |
| `bench/stress/budget.py` | Stress slower than its budget on real problems | `python3 bench/stress/budget.py` |
| `.github/workflows/debugger-loop.yml` | The loop in every editor on every OS | CI |

## Changes

- 2026-11-23: first version, folded from `plan/domains/debugger/README.md` and `ARCHITECTURE.md`.
