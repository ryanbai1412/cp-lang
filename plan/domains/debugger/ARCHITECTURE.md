# debugger architecture

The code as it is on 2026-11-22 (main `4b8f504f`), read from the source. Kept current in the same commit as any structural change. The planned `crates/cpl-trace` and `web/packages/trace-viewer` do not exist yet; see Known debt.

## Data flow

```
source -> cpl-check -> cpl-ir::Program ──> cpl-interp (VM, or tree-walker when traced on wasm / CPL_TRACE_ENGINE=tree)
                                              │  Interp.tracer: Option<Box<trace::Tracer>>  (hooks: declared / changed / print / call / case)
                                              ▼
                              trace::Recording (trace/log.rs: per-step deltas + a snapshot every 1024 steps)
                                              │  trace JSON (tools/trace/trace-schema.json)
           ┌──────────────────────────────────┼──────────────────────────────┐
  cpl-cli trace.rs / live.rs          cpl-cli dap.rs (step back)       cpl-wasm lib.rs (playground)
  -> <file>.trace.html (viewer inlined)                                -> web/ hands JSON to the viewer
cpl-stress: Program (spec.rs typed reads) -> gen.rs/choices.rs -> engine.rs (process or interp) -> judge.rs -> shrink.rs -> Failure -> trace of the minimal case
```

## Modules

| Path | Lines | What it is for |
|---|---:|---|
| `crates/cpl-stress/src/spec.rs` | 2911 | `InputSpec` from typed `read()` declarations: ranges, domains, records, tuples, `Perm`, graph/tree shapes, `@!multitest` |
| `crates/cpl-stress/src/gen.rs` | 690 | random values for a spec; `MAX_LEN` |
| `crates/cpl-stress/src/choices.rs` | 213 | `ChoiceSource`: recorded choices so a `@gen` generator can be replayed and shrunk |
| `crates/cpl-stress/src/shrink.rs` | 1324 | structural shrinking that keeps the input valid for its spec |
| `crates/cpl-stress/src/engine.rs` | 495 | `Engine` trait; `ProcessEngine` (compiled binaries) and `InterpEngine` (in-process, also wasm) |
| `crates/cpl-stress/src/judge.rs` | 346 | compare outputs, run `@check` checkers and `@interactor`s |
| `crates/cpl-stress/src/run.rs` | 489 | the stress loop: `stress`, `stress_with`, `Options`, `Report`, `Failure`, `save` |
| `crates/cpl-stress/src/program.rs` | 592 | `Program`: solution, brute and parts found from decorators |
| `crates/cpl-stress/src/cli.rs` | 830 | `cpl stress` argument handling and output, `--trace` of the failing case |
| `crates/cpl-stress/src/args.rs`, `rng.rs`, `explain.rs` | 102/34/38 | shared flag parser, seeded RNG, text for a failure |
| `crates/cpl-interp/src/trace.rs` | 2350 | `TraceConfig`, `Tracer` (the hooks), `Recording`, `Step`, `Call`, `Place`, JSON for the viewer, divergence of two runs |
| `crates/cpl-interp/src/trace/log.rs` | 1255 | the delta log: one entry per step and change, a full snapshot every `SNAPSHOT_EVERY = 1 << 10` steps |
| `crates/cpl-interp/src/replay.rs` | 581 | rerun one recorded call with an edited function |
| `crates/cpl-cli/src/trace.rs` | 205 | `cpl run --trace`: inlines `tools/trace/index.html` (`include_str!`) with the JSON |
| `crates/cpl-cli/src/live.rs` | 196 | `--live`: local server, rerun on edit |
| `crates/cpl-cli/src/replay.rs` | 250 | `cpl replay <file> --trace t.json --step n` |
| `crates/cpl-cli/src/dap.rs` | 1178 | `cpl dap`: time-travel debug adapter over a `Recording` |
| `crates/cpl-wasm/src/lib.rs` | 661 | `trace_with_stack`, `replay`, `stress`, `stress_with_stack` for the browser; JS in `crates/cpl-wasm/js/cpl.js` |
| `tools/trace/index.html` | 1489 | the viewer: one offline file, sections `trace-style`, `trace-core`, `trace-ui`; schema `trace-schema.json`; tests `trace.test.cjs`, `state.test.cjs` |
| `examples/debugging/` | | the three v0.3 demos and their screenshots |
| `bench/stress/` | | `budget.py` and the seeded bugs run by `crates/cpl-cli/tests/seeded_bugs.rs` |

## Interfaces other domains use

| Interface | Consumers |
|---|---|
| `cpl_stress::{args::Args, fit, make, Case, Exit, InputSpec, Pick, Program, Rng, Shape, TreeShape, Value, Outcome, gen::MAX_LEN}` | `crates/cpl-contest/src/bench.rs`, `corpus_bench.rs` (perf/tools: `cpl bench`) |
| `cpl_stress::stress_source`, `InterpEngine` | `crates/cpl-wasm/src/lib.rs` (site playground) |
| `cpl_interp::trace::{TraceConfig, Recording}`, `cpl_interp::run_observed` | `crates/cpl-cli` (`trace.rs`, `live.rs`, `dap.rs`, `replay.rs`, `interact.rs`), `crates/cpl-stress/src/engine.rs`, `cpl-wasm` |
| Trace JSON, `tools/trace/trace-schema.json` | the viewer, `editors/vscode/src/traceView.ts` (tools), `web/scripts/gen-debug.mjs` (site) |
| `tools/trace/index.html` (raw file) | `crates/cpl-cli/src/trace.rs` (`include_str!`), `web/scripts/gen-debug.mjs`, the VS Code webview |

What this domain uses from the shared core: `cpl_ir::Program` and `cpl_interp` hooks (`Interp.tracer`, `traced(...)`, `crates/cpl-interp/src/lib.rs` 854-920), listed as consumed in `plan/domains/compiler/ARCHITECTURE.md`.

## Invariants

- Deterministic: a seed reproduces the same cases, shrink and trace on every machine and in the browser.
- A shrunk input still satisfies its `InputSpec`; shrinking never produces input the program would reject.
- The trace never changes behaviour: a traced run prints the same output as an untraced one.
- Step `k` is rebuilt from the nearest snapshot plus deltas; nothing is re-executed (`trace/log.rs` header).
- The VM trace equals the tree-walker trace on every program in `crates/cpl-cli/tests/programs` (`tests/trace_engines.rs`).
- The viewer works offline from one file; the trace JSON validates against `trace-schema.json`.

## Known debt

- **Trace lives in a compiler-owned crate.** `crates/cpl-interp/src/trace.rs`, `trace/log.rs`, `replay.rs` (4186 lines) should be `crates/cpl-trace` behind an observer interface; today the tracer reaches into `Interp` internals (`lib.rs` 394, 627, 854-920).
- **Two trace engines.** `CPL_TRACE_ENGINE=tree` (`crates/cpl-interp/src/lib.rs` ~772) exists only for `tests/trace_engines.rs`; and the VM path is `#[cfg(not(target_arch = "wasm32"))]` (~786), so browser traces always tree-walk.
- **Overhead.** Observed-only hooks cost 3.98x / 4.95x untraced; recording adds 1.67x (`metrics.toml`).
- **Viewer is one 103 KB file** with a hard-coded version `0.2.2` (`tools/trace/index.html` 7, 337, 855) and `@brute(main)` wording; its node tests are in no gate; no browser test.
- **Trap evidence stops at the trace.** The terminal R01xx report names no locals and no call path, though the recording has them.
- **Demos rot unchecked.** `examples/debugging/shortest_path.cp` fails E0301 on v0.3 and nothing but the ignored 10^7 test compiles it; all demos spell `@brute`.
- **`cpl-stress` is a grab bag for `cpl-contest`.** `args.rs` and the generator types are reused by `cpl bench` through `cpl_stress::*`; the shared part should be a named module with a stated contract.
- **`spec.rs` is 2911 lines**, the largest file in the crate; split by shape (scalar, compound, graph) when touched.
