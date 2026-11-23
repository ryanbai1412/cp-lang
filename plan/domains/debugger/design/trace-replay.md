# Trace by replay (2026-11-22, debugger)

Status: proposal. Owner: debugger. Reverse by deleting this file; nothing ships yet.

## Problem

The 10^7-step budget test (`cargo test --release -p cpl-interp --test trace_log ten_million -- --ignored --nocapture`)
measures scalar 5.39x and array 5.62x the untraced run (124.6 ms untraced, 307.2 ms observed, 671.3 ms recorded;
array 178.2 / 451.2 / 1001.3 ms), so the observe-only hooks alone cost 2.47x and 2.53x. That is the floor
for this design: recording only lines and frames adds `Log::step` to the observed run, which leaves under 0.5x
of headroom against a 3x ratio, and more against the absolute bar.
A `perf record` of that test spreads the time flat: `exec_chunk_core` 28%, `exec_trace` 14%, then
`step_rest`, `raw_locals`, `raw_write`, `Arc::make_mut`, `locals_ints`, `set_int_item` at 1-4% each.
No single hot spot is left; a 64-slot line cache and an inline loop-counter path were both noise.
Recording every step's delta is the cost itself.

## Idea

The interpreter is deterministic, so a step can be recomputed instead of stored.
Record per step only the line and frame run (what the scrubber and the timeline need), plus a full
snapshot every K steps (the log already takes snapshots, `Log::snapshot_if_due`). Values at step n
come from re-running from the snapshot before n with tracing on for at most K steps.

## What changes

- `Tracer` gains a cheap mode: line and frame only, no `raw_locals` / `raw_write` deltas.
- `Recording::step(n)` replays from the nearest snapshot; step-back cost is bounded by K steps.
- The HTML trace still embeds full values: the CLI replays the kept window (`last_steps`) once
  before writing it, so the viewer and its tests do not change.
- `cpl replay` and DAP use the same replay path.

## Risks

- Needs an `Interp` snapshot and restore (stack, globals, heap values, `Io` cursor, call frames) from the shared core; requested in plan/domains/compiler/worklog.md. Until then a prototype can only replay from the start of the run.

- Programs reading input: a snapshot must hold the input cursor (the VM already owns `Io`).
- Step-back latency: K = 4096 keeps it near today's 21-70 us bound; measure before choosing.
- Two code paths for values until the delta log is removed; keep the delta path behind the
  existing tests until replay passes all of them.

## Exit

Budget test green on the absolute bar in plan/QUESTIONS.md; `trace_log`, `debugger`, `replay`,
`trace_live`, `trace_filters`, `trace_browser` and `cpl-wasm --test parity` unchanged.

## Measured (2026-11-22)

An `Observer` that records only each step's span and call depth, the log
replay needs, over the 10^7-step budget loops (best of 3, through
`run_as` in `crates/cpl-interp/src/observe.rs`): scalar 303.2 ms against
118.3 ms untraced (2.56x), array 447.2 ms against 177.1 ms (2.53x). That
matches the hooks-on floor (2.47x / 2.53x), under the 3x bar; the full
recorder is 5.6-5.9x. With every hook body skipped, trace ops still cost
1.30x / 1.49x, so the rest of the floor is building each hook's `View`,
which a line log does not need: the next cut after replay.

## The +20% target (2026-11-22)

Ryan set recording at most 1.2x the untraced interpreter. Measured on the
10^7-step loops: a full recording is 6.10x / 6.01x, a line-only one 2.56x /
2.53x, and trace ops with every hook body skipped already 1.30x / 1.49x. So no
scheme that runs hooks on every step meets it. What can: the recorded run is
the untraced instance (the no-op observer, which costs nothing) plus a
snapshot every K back edges or calls, a counter costing a few percent; the
trace of any window is rebuilt by restoring the nearest snapshot and replaying
it with the full tracer, which only ever traces K steps at a time. That needs
compiler's Interp snapshot/restore and nothing else from the core.


## Measured: windows by running again (2026-11-22)

`TraceConfig::from_step` skips the first N steps and keeps the rest; `trace::tail` counts a run's steps in one run and keeps the last K in a second. The window holds exactly the steps the whole recording holds there (`a_window_of_a_run_holds_the_steps_the_whole_recording_holds_there`), and memory follows the window. Time does not improve: the last 20,000 of 10^7 steps cost 5.96x (scalar) and 4.42x (array) the untraced run, against 5.09x and 4.98x for recording everything, because a skipped step still pays for its `View` and hooks (the 2.5x observed floor). Next: skip before the `View` is built, counting trace points instead of steps in `vm/exec/trace.rs`, so a skipped step costs the 1.30x dispatch floor; then checkpoints remove the replayed prefix.

Skipping before the locals (a skipped step's `View` built without them, `Observer::skipping`) did not move the tail: 1.05-1.42 s against 1.12-1.39 s over four runs, inside the shared VM's noise. A window by running again costs two observed runs, and the observed run's floor is the hooks themselves. Exactly how much of that floor is the `View` was not measured. Closing the gap needs compiler's option (b), checkpoints on an explicit VM frame stack, so the prototype was not landed.

## Checkpoints, first step (2026-11-23, tech-debt for compiler)

Landed in `cpl-interp/src/checkpoint.rs`. Every value is `Rc`-shared and copied
on write, so a checkpoint copies main's locals and the globals, not the heap.

- **Clock:** `Checkpoint::ticks`, the number of `Stop` ticks so far (one per VM
  backward jump and call). It runs on the untraced VM and costs nothing extra
  until a checkpoint is due: `Stop`'s countdown already takes the cold path.
- **Checkpoint:** taken on the untraced VM at a jump back to the top of a
  `while` or `for v in lo..hi` of main's, once `every` ticks have passed. It
  holds where each enclosing loop is (loop id, and `v`/`hi` for a range),
  main's locals, globals, domains, input cursor, rng state, `queries`, and
  `printed` (bytes printed so far).
- **Restore:** `resume_on_this_thread(program, input, &checkpoint, out, limits,
  trace)` rebuilds that state and runs main on the tree-walker from that loop
  round, traced or not. It prints exactly `whole_output[printed..]`, and a
  traced resume records exactly the last steps of the whole run's trace
  (`tests/checkpoint.rs`, step by step, nested loops, calls, input and `rand`).
  The tree-walker resumes so that the traced build never needs the untraced
  VM's register layout.
- **Overhead (10^7-round loop, release, best of 5, two runs,
  `cargo run --release -p cpl-interp --example checkpoint_overhead`):** a
  checkpoint every 4096 ticks (2441 checkpoints) costs 1.007x-1.075x the
  untraced run; every 65536 ticks (152), 0.999x-1.004x; every 2^20 (9),
  1.001x-1.014x. The untraced run without checkpoints is unchanged (`cpl run`
  on a 3*10^7-round loop and a 10^7-call loop, within noise of main).
- **Step N anywhere:** resume from the last checkpoint before N and keep the
  window with `from_step`/`tail`. A resumed trace numbers its steps from the
  checkpoint, so debugger maps step numbers to ticks (one recorded run that
  counts both is enough); that map is not in this step.
- **Owed (the maximal version):** an explicit VM frame stack, so a checkpoint
  can be taken inside calls (a run spent in one deep call takes none until it
  returns), plus multitest cases and loops over sequences.

## Checkpoints inside calls and test cases (2026-11-23, tech-debt for compiler)

- **Frames as data:** while recording, every VM call pushes a `Suspended`
  record (caller chunk, call pc, register base, value registers, stack base,
  guarded domains) and pops it on return. A checkpoint due at any loop back
  edge, in main or in a callee at any depth, copies the innermost frame
  (locals, loop path) and each suspended caller (locals, loop path, the
  statement and the call expression it waits on) into an immutable `Rc`
  chain. A caller's snapshot is cached until it returns, so the next
  checkpoint in the same call copies only the innermost frame; a caller with
  closures is read again each time, from the stack its locals live on during
  the call. `Checkpoint::depth()` and `Checkpoint::case()` say where it is.
- **Restore:** the tree-walker re-enters main at the recorded loop round,
  runs the waiting statement up to the recorded call (nothing before it in
  that statement has an effect, checked when compiling), and enters the
  callee with its saved locals, down to the innermost frame. Multitest runs
  resume in the recorded case with the cases left.
- **Tests (`tests/checkpoint.rs`):** a resume prints exactly the rest of the
  output, and a traced resume records exactly the whole trace's last steps,
  for nested loops, calls, input, `rand`, 4-deep recursion (checkpoints at
  every depth 1-4) and multitest (checkpoints in several cases); the
  `resumed_tail` and `Timeline` window tests run on both too, and a counted
  (traced) run takes checkpoints inside calls as well, stamped with `steps`. On the
  corpus, sampled checkpoints of 335 programs all resume to the same output;
  12 of the 26 programs running at least 2^17 ticks take checkpoints.
- **Overhead (`examples/checkpoint_overhead.rs`, release, best of 5, two quiet
  runs):** a 10^7-round loop costs 0.990x-1.012x the untraced run, and a
  call-heavy run checkpointing inside calls 1.002x-1.013x, at a checkpoint
  every 4096, 65536 or 2^20 ticks. Without checkpoints, `cpl run` of
  `fib(32)` is within 2% of the base commit (best of 9, 647 vs 635 ms).
- **Not covered (no checkpoint is taken there; the run takes the next one it
  can):** a statement with a call and another effect, calls from a feed loop,
  closure or function value, `for` over a sequence, `var` parameters,
  interactive programs. The VM still recurses on the Rust stack; the frames
  are mirrored as data, not run from it. Step numbers to ticks is still owed.
