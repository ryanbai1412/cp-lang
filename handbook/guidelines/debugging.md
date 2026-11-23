# Building debugging tools

| | |
|---|---|
| Kind | Guidelines |
| Owner | debugger |
| Describes | traces, `cpl stress`, `--explain`, `cpl replay`, `cpl dap`, the trace viewer; see `handbook/design/debugger.md` |

Read this before you change what a trace records, how stress finds or
shrinks a case, what `--explain` says, or what the debugger shows. Each
rule says why, with an example, and names the check that enforces it.

## The rules

### 1. Observing never changes the run

A traced, checkpointed or replayed run prints exactly what an untraced run
prints, byte for byte, and fails the same way. A debugger that changes the
bug can't be trusted with it.

Example: `print(x, end: " ")` four times, then `print()`, is one line
`3 2 1 0 ` in the trace's `stdout`, not four lines.

Check: `crates/cpl-cli/tests/trace_goldens.rs`
`a_traced_run_prints_what_an_untraced_run_prints`.

### 2. Show the user's program, in the user's terms

A trace shows the file's own code and values as the source writes them. A
library's own steps are stepped over unless the trace's fn names ask for
them (`--trace 'Heap*'`); a call into a library that changes a `mut`
argument is one step at the call. A position in `domain Cell = (R, C)`
shows as its parts, a fn by its source name.

Example: `s.set(B(2), true)` on a `Subset` is one step in `main` that shows
`s` changed, not steps inside `Subset.set` with its `self` and `bits`.

Check: `crates/cpl-cli/tests/trace_goldens.rs`
`every_program_records_its_saved_trace` (saved traces of std calls).

### 3. No privileged library

What the trace does for a std type it does for any library's type, through
a mechanism the library can use. Never special-case one std type in the
tracer or the viewer.

Example: stepping over library code (rule 2) applies to every imported
module, a user's own included; a type is shown through its `impl Show`,
never through a table of std names.

Check: process: review of changes to `crates/cpl-interp/src/trace*`.

### 4. Every traced construct runs on the VM

The trace is recorded where the program runs. A construct that falls back
to the tree-walker under tracing is a listed exception that only shrinks.

Check: `crates/cpl-cli/tests/vm_fallbacks.rs` over `vm_fallbacks.txt`.

### 5. Store changes, recompute far steps

Each step stores what it changed, with a full snapshot at intervals; steps
beyond the recording come from checkpoints, not from keeping everything.
Never store a whole large value per step.

Example: showing the first 20000 cells of a 25001-cell array at every step
made one trace 2.3 GB.

Check: `crates/cpl-interp/tests/trace_log.rs` budget tests.

### 6. Explanations are general

An `--explain` rule names a line for a reason both recordings show: a value,
a call, an output line, a repeated state. It never matches a program or a
bug by name, and the count of seeded bugs it names never falls.

Check: `crates/cpl-cli/tests/seeded_bugs.rs`
`explain_names_the_line_each_bug_was_planted_on`.

### 7. Same seed, same evidence

Generation, shrinking, saved cases and traces depend only on the file, the
input and the seed: not on `--jobs`, the OS or the time.

Check: `crates/cpl-stress/src/tests.rs`; `.github/workflows/debugger-loop.yml`
on Linux, macOS and Windows.

### 8. Output formats stay compatible

A new JSON field carries information when present and is left out
otherwise; the schema (`tools/trace/trace-schema.json`) and the viewer
tests change in the same commit.

Example: `"stuck":true` appears only for a run that never finishes.

Check: `tools/trace/*.test.cjs`; `crates/cpl-cli/tests/debugger.rs`.

### 9. One entry per job

Every traced run goes through `cpl_interp::observe_on_this_thread` (with
`Checkpoints` for checkpointed runs). A new kind of run is a new
configuration of it, not a new entry point.

Check: process: review of new `pub fn` in `crates/cpl-interp/src/lib.rs`
and `checkpoint.rs`.

## Checklist

- [ ] The output sweep and saved traces pass; any re-recorded trace's diff
  is explained in the commit (rules 1, 2).
- [ ] Nothing names one std type or one program (rules 3, 6).
- [ ] No new tree-walker fallback (rule 4).
- [ ] The seeded-bug count did not fall (rule 6).
- [ ] JSON changes carry their schema and viewer tests (rule 8).

## Changes

- 2026-11-23: first version, from the debugger design doc's rules.
