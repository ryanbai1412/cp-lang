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
`@brute`:

1. **Find the program's parts.** `crates/cpl-stress/src/program.rs` reads
   the decorators: the solution, its `@alt`/`@brute`, any `@gen`, `@check`
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

## Rules every change keeps

- **The trace never changes behaviour.** A traced run prints exactly what
  an untraced one does.
- **Never store what can be recomputed cheaply, never recompute what the
  user waits on.** Steps are deltas plus snapshots; far steps come from
  checkpoints, not from keeping everything.
- **No new tree-walker dependency.** A traced construct runs on the VM. A
  fallback must be listed in `crates/cpl-cli/tests/vm_fallbacks.txt`, whose
  counts only go down.
- **No privileged type.** The trace shows a library type through mechanisms
  any library can use, never a special case for one std type.
- **Explanations are general.** A new attribution rule must name a line for
  a reason the recordings show, and must not lower the seeded-bug count.
- **JSON stays compatible.** A new field is added only when it carries
  information (`"stuck"` appears only for runs that never finish), and the
  schema and viewer tests change with it.
- **One entry per job.** A new kind of traced run is a new configuration of
  `observe_on_this_thread`, not a new entry point.

## Known limits

- The browser traces on the tree-walker, since the VM isn't built for wasm.
- Checkpoints can't yet stand inside a fed loop body.
- An array over 20000 cells is drawn as `[]`, because each step stores
  every value whole; showing it needs per-step changes in the JSON.
- Steps inside std functions are shown, and a std type such as `Subset` is
  shown by its fields, not its members.
- The trace lives inside `crates/cpl-interp`; it should become its own
  crate behind a narrow observer interface.
- Tracing 10^7 steps costs more than the 1.2x target over an untraced run;
  the budget test is run by hand.

## Changes

- 2026-11-23: first version, folded from `plan/domains/debugger/ARCHITECTURE.md` and `design/trace-replay.md`.
