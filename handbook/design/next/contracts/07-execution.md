# Contract 7: instance IR → engines and C++

Four things consume instance IR. They must agree.

## What crosses

```rust
trait Backend { fn run(program: &Program, input: RunInput, events: &mut dyn Sink) -> Outcome }
// `program` is immutable and shared: any number of runs may use one program at once, on any threads.
// A run owns all of its state (heap, globals, salt, choice tape). Nothing survives a run.

enum Outcome {
    Finished { output: Bytes, exit: i32, used: Usage },
    Failed   { check: CheckName, at: SourceId, origin: List<Origin>, message: Str, output_so_far: Bytes },
    Limit    { which: Time | Memory | Stack | Output | PeerTurns, estimated: bool },
    Stalled  { waiting_on: Program | Peer },         // interactive: both sides are reading
    CompilerBug { instance: InstanceId, detail },
}
struct Intrinsic {              // the only meeting point outside the IR
    id, signature, effects, cost, result_facts,
    model:  fn,                 // reference interpreter: written for clarity
    engine: fn,                 // bytecode engine
    cpp:    Snippet,            // emitter
}
struct JudgeProfile { compiler, flags, limits, kept_checks, size_limit, grader_widths, float_rules }
```

| Backend | Reads | Used for |
|---|---|---|
| reference interpreter | raw | tests, constants computed by library code, lend and move tracking |
| bytecode engine | canonical or optimized | every platform; the only engine in the browser, the debugger and under trace |
| machine-code backend | the engine's bytecode | `cpl run`, tests and stress on desktop x86-64 and arm64 |
| C++ emitter | optimized | submission, `--native` |

## Producer guarantees (each backend)

- **No check fails:** same output bytes and same semantic events on
  every backend and mode.
- **A check fails in a checked build:** same check, same source
  identity, same message, after the same output.
- **Release:** a failed check has the result the spec gives it
  (overflow wraps, a range store truncates, division by zero is 0).
  Outside the contract are only an unchecked index, `x!`, `D(x)`,
  `@trusted` and `@cpp`.
- Values are typed and flat, laid out by `facts/layout.md`, on every
  backend. Memory counted locally is the memory the C++ uses.
- Floats agree bit for bit: IEEE operations exact, printing and
  parsing one algorithm, `sin`/`exp`/`pow` one CP Lang body. The
  emitted C++ turns off fused multiply-add; a profile with fast-math
  is rejected.
- A `@fast(op)` fn means its CP Lang body. A backend that runs the op
  in its place gives what the body gives on every input; the op is an
  optimisation, never a second meaning.
- Ownership places every release: `Func::drops` at frame exit and
  `Stmt::Drop` at a discarded value and an overwrite (cpl-next-own).
  A backend runs each one and places none of its own. A lent or viewed
  (`Func::views`) param is the caller's value, so it is never in
  `Func::drops`: a fresh value given to a viewed param is set in a
  temporary the caller releases, and an earlier operand with an effect
  goes to a temporary before it, so effects keep their order. The
  engine counts a copy only where it lowers an `EK::Copy`; a view and a
  load take no count. Which values hold
  storage is decided once, by `Program::stores`: ownership places drops
  by it and the engine counts a copy by it, and no backend keeps a bit
  of its own. The engine's optimizer removes a `Drop` only when its
  registers are still zero from a `Zero` on every path to it; releasing
  a zero word does nothing, so removing it changes no backend's
  behaviour and needs no shared stage. C++ runs each `Stmt::Drop` as a
  reset of its place. At frame exit its locals' destructors release
  exactly `Func::drops`: a C++ value owns its storage outright (a copy
  is a deep copy), and a lent slot is a reference that owns nothing.
- Sorting is one stable algorithm with a stated comparator call order.
  `F64` has one total order.
- `HashMap` iterates in insertion order.
- Local limits are estimates and say so. The native build under the
  judge profile is the authority.

## Consumer may assume (tools)

- Any backend can stand in for another for a program that is not
  native-only. A program that reaches a grader or a body-less `@cpp`
  function is **native-only**, by one rule: run, test and stress build
  the C++; trace is native streaming; the playground refuses.

## Must never

- A backend must not implement a container, a sort or a string
  operation. Those are library IR.
- The emitter must not decide anything (see contract 6).

## Checked by

- Differential on the corpus and generated programs, for each pair:
  reference vs engine, engine vs machine code, engine vs C++,
  checked vs release.
- Intrinsic-level: model vs engine vs C++ on generated arguments,
  including stateful sequences for storage.
- `@fast`: `fast_ops_agree_with_their_bodies` calls every `@fast` fn on
  edge and seeded random words; the reference interpreter (body), the
  engine and the C++ with the op, and both with `--bodies` agree.
  `fast_unchecked` counts the `@fast` fns it cannot call (0).
- Benchmarks with gates: interpreter 8x of C++ on desktop and 12x in
  the browser, machine code 5x, emitted C++ p50 1.1x.

## Open

- The machine-code backend is a second path to keep correct. It is
  built last, and the interpreter's bar stands alone until then.
- A user library's `@cpp` function must have a CP Lang body; how the
  two bodies are compared on inputs the tests did not think of.
- **Audit 4, finding 9.** Every engine bounds-checks every storage
  operation always, also inside `unchecked:`. "Outside the contract"
  means only that the C++ may differ. Otherwise an unchecked write
  corrupts the language server or the playground.
- One output comparison rule (tokens, 1e-6 on floats, `@check`,
  `checker.py`, interactor verdict) used by test, submit, watch,
  stress and the site. It belongs here and is not yet a data shape.
- **Slice: shapes that worked in C++17.**
  - A function value: a struct with a tag and one member per closure;
    a call is a chain of tests on the tag, each a direct call.
  - A generator or lender: a template taking the caller's code as a
    lambda that returns how it left (went on, break, return, continue).
  - A generator that can reach itself: the caller's code is passed as
    two words and called indirectly, since a template would need a new
    type per level. This is the one indirect call in the output.
  - A by-value parameter that is never written: `const&`.
- **Prototype audit, measured.**
  - Two lends at once: nesting ends the last-lent first; today's
    compiler ends the first-lent first, and the spec has no rule. Pick
    last-lent first, the only order a recursive callee can give.
  - A lender that is not recursive should become "before" and "after"
    code in the caller; only a recursive one takes the caller's code.
    Passing it erased costs 1.12x to 1.24x on early return; as a
    template 1.01x.
  - `HashMap` in insertion order (dense entries plus a table of 32-bit
    indexes) is the fastest layout tested on insert, erase, iteration
    and churn, at half the memory of a plain flat map. Lookup is 1.06x
    to 1.15x of the best, 1.08x with hash bits in the index table.
    Today's `std/hashmap.cp` fails everywhere and must be rewritten.
  - One stable merge sort misses 1.1x of `std::sort` on 12 of 14
    workloads. A family passes 12 of 14: keys computed once, a stable
    radix sort for integer keys, a run-detecting merge otherwise. The
    spec must promise the result and "each key computed once, in
    order", and leave the comparisons unspecified.

## Added from the journeys (2026-11-24, not yet reviewed)

- **Many runs of one program.** Stress, sample tests and the test
  runner start many runs against one `Program`. The guarantee above
  (immutable program, run-owned state) is what makes that safe, and it
  rules out a backend caching anything per program that a run writes.
  Checked by: the same 1,000 inputs run on 1 thread and on N give the
  same outcomes in the same order of test id.
- **A fn as the entry.** `Entry::Call` is a property of the run, not
  of the program: the instance set is the program's, plus the
  instances the call's argument types need. A backend gets one entry
  instance id and argument values in the engine's value form.
- **Interactive runs.** A peer is a second `run` on its own program
  whose output is this run's input and the reverse. Both use the same
  backend trait; the pump between them lives in the facade, not in a
  backend. The C++ path runs two processes joined by pipes.
- **Each judge's compiler.** A `JudgeProfile` names a compiler and its
  flags. Gate: the emitted C++ for the conformance suite compiles
  with no warnings promoted to errors under every profile's compiler,
  and the size limit holds. A profile whose compiler is not installed
  in CI fails the gate; it is not skipped.

## Round 5 (2026-11-25, not yet reviewed)

- **`unchecked:`** The spec says every check is off in every build;
  this contract says engines always bounds-check storage. Resolution:
  the engine keeps the check and reports the fault as "undefined in
  the submitted program", which is what stress needs to find.
  The C++ has no check there. This is the one place engines and C++
  differ by design, and the differential comparison marks it.
- **Sorting.** The spec's cost table says `sort(by:)` is `std::sort`.
  With sort in library source that row is replaced by the measured
  family of three (decision on stable sort), with its own 1.1x gate.
- **Native-only programs** (a grader, a body-less `@cpp` function)
  are refused by the engines with a named reason and run only as C++.
