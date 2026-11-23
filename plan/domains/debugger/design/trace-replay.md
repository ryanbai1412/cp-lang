# Trace by replay (2026-11-22, debugger)

Status: proposal. Owner: debugger. Reverse by deleting this file; nothing ships yet.

## Problem

The 10^7-step budget test (`cargo test --release -p cpl-interp --test trace_log ten_million -- --ignored --nocapture`)
measures scalar 5.39x and array 5.62x the untraced run; the observe-only hooks already cost 2.1-2.2x.
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

- Programs reading input: a snapshot must hold the input cursor (the VM already owns `Io`).
- Step-back latency: K = 4096 keeps it near today's 21-70 us bound; measure before choosing.
- Two code paths for values until the delta log is removed; keep the delta path behind the
  existing tests until replay passes all of them.

## Exit

Budget test green on the absolute bar in plan/QUESTIONS.md; `trace_log`, `debugger`, `replay`,
`trace_live`, `trace_filters`, `trace_browser` and `cpl-wasm --test parity` unchanged.
