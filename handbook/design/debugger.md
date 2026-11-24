# Debugger design: stress, trace, replay, the debug adapter

| | |
|---|---|
| Kind | Design |
| Owner | debugger |
| Describes | `crates/cpl-stress/`, `crates/cpl-interp/src/trace.rs`, `crates/cpl-interp/src/trace/`, `crates/cpl-interp/src/checkpoint.rs`, `crates/cpl-interp/src/replay.rs`, `crates/cpl-cli/src/trace.rs`, `crates/cpl-cli/src/dap.rs`, `crates/cpl-cli/src/live.rs`, `crates/cpl-cli/src/interact.rs`, `crates/cpl-wasm/src/lib.rs`, `tools/trace/` |

How the debugging tools are built: what each part does, the data it hands
on, the rules every change keeps, and the known limits. The domain's
qualities and suites are in `handbook/domains/debugger.md`.

## A run through the parts

`cpl stress sum.cp --explain --trace` on a file with a solution and a
`@alt` fn:

1. **Find the program's parts.** `crates/cpl-stress/src/program.rs` reads
   the decorators: the solution, its `@alt`, any `@gen`, `@check`
   and `@interactor`.
2. **Generate inputs.** `spec.rs` turns the typed `read()` declarations
   (`let n: 1..=5 = read()`, `let a: [n]0..=9 = read()`) into an
   `InputSpec`; `gen.rs` draws cases from it with a seeded `Rng`. A `@gen`
   fn is run instead when the file has one; `choices.rs` records its random
   choices so it can be replayed and shrunk.
3. **Run and judge.** `engine.rs` runs both sides, as compiled C++ or in
   the interpreter; `judge.rs` compares tokens or calls the checker or the
   interactor.
4. **Shrink.** `shrink.rs` deletes and lowers parts of the failing case,
   keeping it valid for its `InputSpec`, while it still fails. The result
   is saved in `sum.tests/` so `cpl test` reruns it.
5. **Explain.** Both sides are traced on the minimal case and the two
   recordings are compared (`trace/compare.rs`, `crates/cpl-cli/src/trace.rs`):
   the first value or call they disagree on, then the lines that made it.
6. **Show.** The recording becomes trace JSON, inlined into
   `tools/trace/index.html`, which opens on the divergence.

```
source -> cpl-check -> cpl-ir::Program
                            │  cpl_interp::observe_on_this_thread(program, io, output, limits, Some(TraceConfig))
                            ▼
              Tracer hooks (declared, changed, print, call, case) on the VM
                            ▼
              trace::Recording: a delta per step, a snapshot every 1024 steps
                            │
        ┌───────────────────┼──────────────────────────┐
  cpl run --trace /     cpl dap (step back,       cpl-wasm (playground)
  --live / cpl replay   breakpoints, restart)
        ▼
  trace JSON (tools/trace/trace-schema.json) -> viewer
```

Tracing is a flag on the commands that already run a program (`cpl run
--trace`, `--live`, `cpl stress --trace`, `cpl replay --trace --step`) plus
`cpl dap`; there is no `cpl trace`, `cpl snap` or `cpl debug` (decision 273).
One flag on a command the contestant already types is guessable, and
`replay` of a recorded call does what a snapshot command would without a
second file format. Lost: separate debugging commands, which would be thin
wrappers over the same code.

## Parts

### Stress (`crates/cpl-stress`)

- **Inputs come from types.** A declaration says the range, length, shape
  (permutation, tree, graph, grid) and whether the file is a multitest. The
  generator and the shrinker both work on the `InputSpec`, so a shrunk case
  is always one the program would accept. An input shape the generator
  can't make is reported at its declaration, never guessed.
- **Determinism.** Every random choice comes from the seed. The same seed
  gives the same cases, the same shrink and the same saved file for any
  `--jobs` and on every OS.
- **Who is wrong.** A brute that crashes, an interactor that gives up and a
  generator that fails are each named as such; only a disagreement or a
  solution failure is a counterexample.
- **No C++ compiler needed.** With no `g++`, every side, `@gen` included,
  runs in the interpreter.

### The trace (`crates/cpl-interp/src/trace*`)

- **One entry.** Every traced run goes through
  `cpl_interp::observe_on_this_thread`. A run that also takes checkpoints
  uses `checkpoint::observe_on_this_thread`, the same entry plus a
  `Checkpoints` request (how often, how many to keep, where to resume
  from). It returns the checkpoints beside the result, because a checkpoint
  holds `Rc`s and can't cross threads.
- **Hooks, not a second interpreter.** The VM calls the `Tracer` when a
  variable is declared or changed, something is printed, a call starts or
  ends, or a test case starts. Untraced runs use a no-op observer, so
  tracing off costs nothing.
- **The log.** `trace/log.rs` stores each step as the changes it made, plus
  a full snapshot every `SNAPSHOT_EVERY` (1024) steps. Step `k` is rebuilt
  from the snapshot before it and the changes since; nothing is rerun.
- **Windows and checkpoints.** A recording keeps at most a fixed number of
  steps. To show a step beyond it, the run is resumed from the nearest
  checkpoint and traced for one window. The CLI (`--from`, `--last`),
  `--live`, trap reports and `cpl dap` all read long runs this way.
- **Source terms.** Values are shown as the program wrote them: a position
  in `domain Cell = (R, C)` as its parts, a variable by its source name, a
  fn by the name the source gives it.
- **Saved traces are the oracle.** Each program in
  `crates/cpl-cli/tests/programs/` has a saved `.trace`;
  `trace_goldens.rs` fails on any change, on a program without one, and on
  a saved trace that records a crash of `cpl` itself.

### Divergence (`trace/compare.rs`, `crates/cpl-cli/src/trace.rs`)

Given two recordings of the same input, find the first thing they disagree
on, then walk back to the line responsible. The rules it applies, in order:

1. The first call of a part whose `@alt(f)` answers differently, innermost
   in a recursion, with its arguments.
2. Otherwise the first value of a variable both sides name that differs.
3. Otherwise the first output line that differs, and the variables its
   `print` shows.
4. From the wrong value: the line that last changed it, the lines that
   last changed what that line reads, the `if` or `continue` that let it
   run, a container declaration whose kind decides which end a `pop()`
   takes, and a run-wide `var` when a later call with the first call's
   arguments returns another value (multitest state leaks).
5. A run that never finishes: the loop whose state repeats.
6. An interactive run: the answer the judge rejected and what set it.

Each rule is general: it reads the recordings and the source, never a
bug's name. The seeded-bug suite measures how often the planted line is
named, and that count may only go up.

### The debug adapter (`crates/cpl-cli/src/dap.rs`)

`cpl dap` speaks the Debug Adapter Protocol over stdio. It records the run
once, then answers `stepBack`, `reverseContinue`, breakpoints, watches and
the stack from the recording. Past the recording's limit it re-traces
windows from checkpoints. `restart` records the edited file on the pinned
input and lands on the same place in the program, not the same step
number.

### The viewer (`tools/trace/`)

One offline HTML file with three sections (`trace-style`, `trace-core`,
`trace-ui`) and a JSON schema. The CLI inlines it with the trace; the VS
Code extension and the site load the same file. It has no build step and
makes no network requests.

---

# Reference

## Rules every change keeps

The rules are explained in `handbook/guidelines/debugging.md` and listed, with their checks, in `handbook/guidelines/debugging-checklist.md`.

## Invariants

- A seed reproduces the same cases, shrink and trace on every machine and in
  the browser (`debugging/R7`).
- A shrunk input still fits its `InputSpec`: shrinking never makes input the
  program would reject.
- A traced run prints what an untraced run prints (`debugging/R1`).
- Step `k` is rebuilt from the nearest snapshot plus deltas, or re-run from
  the nearest checkpoint (`debugging/R5`).
- Every program in `crates/cpl-cli/tests/programs` records its saved
  `.trace`, and no saved trace holds a cpl crash.
- The viewer works offline from one file; trace JSON validates against
  `tools/trace/trace-schema.json` (`debugging/R8`).

## Modules

| Path | What it is for |
|---|---|
| `crates/cpl-stress/src/spec.rs` | `InputSpec` from typed `read()`s: ranges, domains, records, tuples, `Perm`, graph and tree shapes, `@!multitest` |
| `crates/cpl-stress/src/gen.rs`, `choices.rs` | random values for a spec; recorded choices so a `@gen` fn replays and shrinks |
| `crates/cpl-stress/src/shrink.rs` | structural shrinking that keeps the input valid |
| `crates/cpl-stress/src/engine.rs`, `judge.rs` | running both sides (compiled or in-process) and judging them (`@check`, `@interactor`) |
| `crates/cpl-stress/src/run.rs`, `program.rs`, `cli.rs` | the stress loop, the parts found from decorators, `cpl stress` itself |
| `crates/cpl-interp/src/trace.rs`, `trace/` | `TraceConfig`, the `Tracer` hooks, the step log, rendering, divergence |
| `crates/cpl-interp/src/checkpoint.rs`, `replay.rs` | checkpoints and windows; rerunning one call with an edited fn |
| `crates/cpl-cli/src/trace.rs`, `live.rs`, `replay.rs`, `dap.rs` | `cpl run --trace`, `--live`, `cpl replay`, `cpl dap` |
| `crates/cpl-wasm/src/lib.rs` | trace, replay and stress for the browser |
| `tools/trace/` | the viewer, its schema and tests |
| `bench/stress/`, `examples/debugging/` | seeded bugs and the stress budget; the debugging demos |

## Known limits

- The browser traces on the tree-walker, since the VM isn't built for wasm.
- Checkpoints can't yet stand inside a fed loop body.
- An array over 20000 cells is drawn as `[]`, because each step stores
  every value whole; showing it needs per-step changes in the JSON.
- A value whose type has an `impl Show` is shown through the impl, but the
  trace runs that `show` fn on the tree-walker.
- The trace lives inside `crates/cpl-interp`; it should become its own
  crate behind a narrow observer interface.
- Tracing 10^7 steps costs more than the 1.2x target over an untraced run;
  the budget test is run by hand.
- `cpl bench` reuses `cpl-stress`'s flag parser and generator types; the
  shared part should be a named module with a stated contract.
- `crates/cpl-stress/src/spec.rs` is the crate's largest file; it should be
  split by shape (scalar, compound, graph) when next changed.

## Changes

- 2026-11-23: first version, folded from the debugger plan's architecture notes and `design/trace-replay.md`.
