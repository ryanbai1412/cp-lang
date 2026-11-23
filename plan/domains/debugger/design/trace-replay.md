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
  can):** calls from a feed loop, closure or function value, `var`
  parameters, interactive programs (`for` over a sequence: below). The VM still recurses on
  the Rust stack; the frames are mirrored as data, not run from it. Step
  numbers to ticks is still owed.

## Calls that share a statement (2026-11-23, tech-debt for compiler)

- **Shape:** a statement whose head makes several calls, or a call and other
  effects, e.g. `t = f(a) * f(b) + g()` or `print(f(x), f(y))`. Each call
  in evaluation order is a call site until the first other effect (print,
  read, `rand`, a write, a closure); a call after one is not.
- **Restore without doing anything twice:** while recording, each call op a
  later call site needs keeps what it returned (`Chunk::keeps`, and
  `keep_called` for calls the tree-walker makes in a statement the VM falls
  back to). A checkpoint inside the n-th call saves what calls 1..n-1
  returned; the resumed statement takes those values instead of calling
  again, then enters call n from its saved frame. Statements the traced VM
  runs on the tree-walker (`FallbackStmt`) are waits too, so counted
  (traced) runs take checkpoints inside their calls.
- **Found on the way:** a fallback statement's callee runs on a fresh
  register vector, so its caller's registers are now saved by address; and
  the VM's `ForInc` left a finished loop's variable at `hi` where the
  tree-walker leaves `hi - 1`, which a checkpoint after the loop copied into
  the trace. `ForInc` now keeps the last value, as `ForIncClosed` did.
- **Tests (`tests/checkpoint.rs`, fixture `SHARED`: calls sharing a
  statement, with `print` and `rand` between them, and a 3-deep recursion
  of two calls per statement):** the run, traced-run, window, tail and
  timeline tests, and every checkpoint of a counted run resumes to the rest
  of the output. `Timeline::starts()` is checked spread over the whole run
  (first before 1/4, last after 7/8, no gap over 1/4) for each fixture.

## Loops over sequences (2026-11-23, tech-debt for compiler)

- **Shape:** `for x in seq` and `for x in seq.rev` over a list, list view,
  array or string, in main or any fn, at any depth, e.g. `for c in word:` or
  `for x in a.rev:` around or inside calls. The VM's sequence loops now have
  a back edge like range loops (`LoopReg::Seq`: the position register and
  where the loop holds the sequence, a local register or a global).
- **Restore:** a checkpoint saves the position and the held sequence itself
  (`FrameAt::held`, a shallow `Rc` copy), so a loop over a temporary
  (`for x in f(a)`) resumes on the value the loop began with, as the
  tree-walker walks it (the body can't change the sequence: E0602). The resumed tree-walker
  takes that value instead of evaluating the sequence again
  (`Interp::seq_held`) and starts at the saved round
  (`checkpoint::rounds`). Sets and maps take no checkpoint yet (the run
  takes the next one it can).
- **Found on the way:** a run resumed from an untraced recording showed
  locals not declared yet (`"c": "\u0000"`, `"x": 0`), because an
  untraced register reads 0 before its `let`. A local is now taken as
  undeclared when its registers hold nothing yet, unless it surely is
  declared (a parameter, a `let` earlier in a block on the way, or the
  variable of a loop the checkpoint is in). Left over: a local in an
  earlier loop or branch that holds 0 or `Unit` shows as undeclared after
  such a resume where the full trace shows it; counted (traced) runs keep
  every declaration exact. `ForDec` now ticks like `ForInc`.
- **Tests (`tests/checkpoint.rs`, fixture `SEQ`: a list walked in a fn
  called from a loop over a list literal, a string loop, and a reverse
  loop):** the run, traced-run, window, tail and timeline tests, and every
  checkpoint of a counted run resumes to the rest of the output.
- **Overhead (`examples/checkpoint_overhead.rs`, new `seqs` workload,
  release, best of 5):** 1.007x-1.021x the untraced run at a checkpoint every
  4096, 65536 or 2^20 ticks. cpl.wasm gzipped grows 1,121 bytes (the
  `rounds` walk), to 1,978,925 under the 1,980,000 cap. Without checkpoints,
  a 1500-round forward plus reverse walk of a 10^4 list runs 530 ms against
  538 ms on the base commit (interleaved, best of 9).

## Closures and function values (2026-11-23, tech-debt for compiler)

- **Shape:** a loop inside a closure or a nested fn, and a loop calling a
  function value (`f(i)` on a `(Int) -> Int` parameter, a `let` holding a
  lambda, a lambda passed inline), at any depth, e.g. `apply(x => x * k + 1,
  n)` or `t = (t + walk(3000) + step(r)) % p` in a fn whose `walk` and
  `step` are closures.
- **Mechanism:** a frame now says whose body it runs (`checkpoint::Body`:
  main, a fn or a closure) instead of only a fn id, so the VM's
  `CallClosure` suspends its caller like a fn call does, and a run from a
  checkpoint makes the closure's frame again on the tree-walker
  (`resume_closure`, as `call_lending` makes it, captures already in the
  saved locals). A call to a function value is a call site like a fn call
  (`Order::walk`, `call_op`, `call_result` know `CallExpr` and
  `CallClosure`), so the calls before it in its statement are kept and not
  run again, on the VM and on the tree-walker (`ExprKind::CallExpr` resumes
  and keeps like `call_or_resume`).
- **Not taken:** a frame holding a closure that captures a stack place by
  reference (`CaptureVal::StackRef`/`PartRef`) takes no checkpoint, as with
  aliases; nor a closure with `var` parameters.
  (`var` parameters of fns: see below.)
- **Found on the way:** a checkpoint came due on a tick no checkpoint is
  taken at (a call, a statement on the tree-walker) and the clock looked
  again only 4096 ticks later; when a loop's round has the same ticks every
  time, every look fell on such a tick, so a traced `print(apply(...))`
  took no checkpoint at all. While one is due the clock now looks again on
  the next tick, for up to 4096 looks after each checkpoint taken
  (`Clock::after_look`, cold path only).
- **Tests (`tests/checkpoint.rs`, fixture `CLOSURE`: an inline lambda
  passed to a fn's loop, a nested fn and a block lambda each looping, called
  in one statement from a loop):** the run, traced-run, window, tail and
  timeline tests, and every checkpoint of a counted run resumes to the rest
  of the output.
- **Overhead (`examples/checkpoint_overhead.rs`, new `closures` workload,
  release, best of 5):** 1.124x-1.140x the untraced run at a checkpoint every
  4096, 65536 or 2^20 ticks: each closure call now records its frame while
  checkpoints are on. Without checkpoints, 3000 rounds of the
  same workload run 1479 ms against 1487 ms on the base commit
  (interleaved, best of 9). cpl.wasm gzipped grows 25 bytes, to
  1,662,878 under the 1,720,000 cap.

## `var` parameters

- **Taken:** a checkpoint stands in a fn with `var` parameters like in any
  other, at any depth, for a local or a global argument (`bump(mut t, n)`,
  `fold(mut xs, n)`). An element argument (`f(mut a[i])`) is untested.
- **Why it differs:** the VM passes a `mut` argument by copy-in, copy-out:
  while the callee runs, the callee's register holds the value and the
  caller's place is stale. The tree-walker passes a reference
  (`Value::Alias`/`Value::Part`) to the caller's place. So when a resumed run
  reaches the waited call (`resume_pending`), each `mut` argument's value goes
  back from the saved callee frame into the caller's place, and the callee's
  slot becomes the tree-walker's borrow of it (`Interp::lend_at`, the same
  borrow a plain call makes). The callee then writes through, as in the C++.
- **Still not taken:** a closure with `var` parameters (closures with `mut`
  captures run on the tree-walker).
- **Tests:** fixture `MUT` (a global and a local `Int`, and a `List`, passed
  `mut`, at depth 1 and 2) is in every `tests/checkpoint.rs` test, and
  `a_checkpoint_can_stand_inside_calls_and_test_cases` checks checkpoints
  land at both depths.

## Feed loops

- **Taken (untraced recordings):** a checkpoint stands in a loop of a
  `yields` fn feeding a `for` (`for x in evens(n)`), in main or in any fn,
  at any depth.
- **How:** the VM's `CallVmFeed` now suspends its caller like a call
  (`Body::Fed`, the fed chunk in `compiled.fed_fns`) and pushes the fn's
  call frame, as the tree-walker does. The `for` is the call site
  (`note_fed_site`). A resumed run makes the `for` again on the tree-walker;
  at the waited call it resumes the fn's frame while it feeds the loop
  (`resume_fed`, `resume_callee` with a `Fed`), so each later `yield` runs
  the body as before.
- **Found on the way:** the frame waiting on a call is read once and kept
  for later checkpoints (`known`), since it can't change while it waits. A
  fed loop's caller does change: the body runs in it at each `yield`. Each
  yield now marks that frame to be read again (`Interp::feeding`).
- **Traced runs:** the traced VM leaves the `for` to the tree-walker,
  which now runs the fn on its traced chunk (`call_feeding`), so a counted
  run and `Timeline` take checkpoints in its loops. Each `yield` there is a
  fallback: the tree-walker feeds the loop as it would, so the trace is
  the tree-walker's. The `for` is the call site its caller waits at
  (`note_call_site`), and the tree-walker's `feed` marks the caller to be
  read again, like the VM's.
- **Not taken:** in the body of a fed loop (`Interp::fed_bodies`): it runs
  in the caller's frame inside the fn's, which a resume does not make
  again.
- **Traced speed:** the traced VM also runs the rest. A traced chunk that
  leaves a fed `for` to the tree-walker compiles the body out of line
  (`Chunk::fed_bodies`), and the tree-walker's `feed` runs it there
  (`fed_body_vm`, found through `Interp::fed_hosts`). The fn's `yield` is
  `Op::FeedTree`, which feeds the loop and reports the push as the
  tree-walker's `yield` does, with no fallback. A counted run of a 4×10^6
  `yield` loop with a one-line body takes 490 ms against 160 ms
  untraced (3.1x), from about 7x.
- **Tests:** fixture `FEED` (fed loops in main and in a fn, with a body
  loop) is in the run, traced-resume, window, tail, `Timeline` and counted
  tests.

## Traced fns with `var` parameters

- **Before:** a traced run left every fn with a `mut` parameter to the
  tree-walker, so a counted run took no checkpoint in it and cost 10x
  untraced.
- **Now:** the traced VM runs them. A traced call always comes from the
  tree-walker, which lends the caller's place (`Value::Alias`); the VM
  moves the value into the param and moves it back on return, as untraced.
  While the tracer or a fallback looks at the frame, the param holds the
  alias again and the caller's place the value (`show_lent`/`hide_lent`
  around each `Trace` op and fallback), so both see what the tree-walker
  would. A param lent on from a param follows the aliases to the place.
- **Cost:** a counted run of a 1.6×10^7-round loop in `bump(mut s, n)`
  takes 569 ms against 199 ms untraced (2.9x), from 2027 ms on the
  tree-walker.
- **Tests:** `trace_engines` (traced VM = tree-walker trace) covers the
  `mut` programs in `tests/programs` (`dsu`, `functions` and 21 more, which
  differed or trapped before the lending fix); fixture `MUT` is in the
  counted and `Timeline` tests.

