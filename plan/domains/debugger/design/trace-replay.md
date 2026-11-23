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

