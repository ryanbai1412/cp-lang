# Codegen design: from a lowered program to one C++17 file

| | |
|---|---|
| Kind | Design |
| Owner | perf |
| Describes | `crates/cpl-codegen/`, `crates/cpl-contest/src/{bench,zero_cost,corpus_bench}.rs`, `bench/` |

How the C++ emitter is built: what each part decides, the one analysis
that decides copies and calls, the rules every change keeps, and where the
known limits are. The domain's qualities and suites are in
`handbook/domains/perf.md`; the rules an optimization follows are in
`handbook/guidelines/performance.md` and its checklist.

## A run through the parts

`cpl build sol.cp` on a program that sorts pairs with a lambda and walks a
list of lists:

1. **Read the facts.** The emitter takes the lowered `cpl_ir::Program` and
   the compiler's effect table (`cpl_ir::effects`: what each fn and closure
   can write, print or trap on). It never reads syntax or re-derives types.
2. **Decide each value once.** `uses.rs` builds the Uses table: for every
   heap binding, element, parameter, capture and loop binding, whether it is
   mutated, moved, escapes, or its source can change while it lives. A
   binding with no reason to be owned is a `const&`; one whose next use is
   an overwrite is moved; any other copy is kept and `cpl explain` names
   why.
3. **Decide each call once.** `devirt.rs` finds the fn values that can only
   ever hold one closure (a field, a parameter, a local `let f = mon.op`)
   and calls them directly; only a value that can really hold different
   closures becomes a `std::function`.
4. **Decide each check and width.** `ranges.rs` and `bounds.rs` drop the
   index and sign checks a range proves; `scope.rs` and the IR's narrow
   layouts give each int and element the fewest bytes its type allows;
   `niche.rs` packs optionals into one scalar where a value is spare.
5. **Rewrite loops when it can't be seen.** Fusion (`fuse.rs`), reduction
   sinking (`sink.rs`), splicing a known closure into its loop
   (`splice.rs`) and presizing (`presize.rs`) each need the effect table
   to show that the rewrite changes no output, order, effect count or
   trap.
6. **Write and fit.** `emit.rs` writes types, globals, fns in order
   (`order.rs`) and `main`; `runtime.rs` adds only the runtime pieces used;
   `lib.rs` and `compact.rs` fit the file under the judge's limit by
   dropping comments, the source header and indentation, never meaning.

## Invariants

- One deterministic C++17 file per program: same input, same bytes
  (`crates/cpl-cli/tests/deterministic_cpp.rs`).
- The interpreter and the C++ agree on every program, including traps and
  rejected input (`crates/cpl-cli/tests/differential.rs`, conformance).
- An optimization majorizes: it is never worse on any axis, changes nothing
  a program can observe, and `cpl explain` can show it.
- A heap value that is only read is read in place; a fn value whose closure
  is known statically is called directly. Both are counted over the corpus
  by `crates/cpl-cli/tests/read_only_copies.rs` and only go down.
- Every decision is one general rule in one module, never keyed to a
  library name.

# Reference

## Modules

| File | What it decides |
|---|---|
| `lib.rs` | Public API (`emit`, `emit_for`, `emit_within_for`, `emit_profiled`, `sizes`, `Target`, `Part`) and fitting a source limit |
| `emit.rs`, `emit/` | The `Emitter`: the whole file, types and layouts, globals, fn order, `main`; `emit/text.rs` the header, inline hints, ahead declarations and dropped list forms |
| `expr.rs`, `stmt.rs`, `show.rs` | Expressions, statements and loops, printing and reading values |
| `uses.rs` | The Uses table: lend, move or copy for every non-scalar binding |
| `lend.rs` | Branches of a `let` that name a place and are read in place |
| `devirt.rs`, `splice.rs` | Direct calls through fn values with one known closure; splicing it into its loop |
| `fuse.rs`, `sink.rs`, `inline.rs`, `presize.rs` | Loop fusion, reduction sinking, loops over list-literal fns, comprehension reserve |
| `bounds.rs`, `ranges.rs`, `divrem.rs`, `prune.rs` | Checks a range proves, sign facts, Euclidean `/` `%`, branches decided at compile time |
| `scope.rs`, `rowcache.rs`, `niche.rs`, `unordered.rs` | Where locals are declared, row reuse, one-scalar optionals, hash-table maps |
| `names.rs`, `order.rs`, `runtime.rs`, `compact.rs`, `checks.rs` | C++ names and types, fn order, runtime pieces, last-resort fitting, the build's run-time checks |
| `explain.rs` | `cpl explain` text: closures, views, copies |

## The copy and effect analysis

Five places once each answered part of "may this be read in place, moved,
dropped or reordered", and each new shape needed its own fix. The design is
one analysis with two tables, computed once per program and read by every
client:

- **Effects** (`cpl_ir::effects`, owned by compiler): per fn and closure,
  whether a call writes outside its own locals, does I/O, can trap under
  the build's profile, or calls a fn value it cannot pin down. A bottom-up
  fixpoint; recursion and unknown callees count as effectful.
- **Uses** (`uses.rs`, owned by perf): per binding, the reason it must be
  owned, if any, and the closures each fn-valued home can hold.
- **One decision:** lend by default, copy only for a reason Uses names (and
  `cpl explain` prints), move when the next use is an overwrite. A rewrite
  that changes how often or when code runs needs the effects to be
  unobservable.

Still to come: specializing a struct per closure held in its fn fields
(`Heap.lt`, `Monoid.op`), as `Window(+)` already is, which removes most of
the `std_functions` left.

## Bench tools

| File | What it is for |
|---|---|
| `crates/cpl-contest/src/zero_cost.rs` | `cpl bench --zero-cost`: each abstraction against its direct form and `hand.cpp` |
| `crates/cpl-contest/src/corpus_bench.rs` | `cpl bench --corpus`: ports against `orig.cpp`, tuned `perf.cp` beside them |
| `bench/v03_perf_bench.py` | The perf set: matched references, build/run/skip accounting, p50/p95/max and tuned max |
| `bench/sizes.py`, `bench/compiles.py` | Generated C++ size and g++ compile checks |

## Changes

- 2026-11-23: first version, folded from the perf plan's architecture notes.
