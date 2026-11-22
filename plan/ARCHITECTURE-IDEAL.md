# The ideal architecture, from a clean slate

What the CP Lang toolchain would look like if no code existed yet. This is
a reference point, not the plan: `plan/ARCHITECTURE.md` is how we get
close to this from today's code. Where the two differ, this file says what
we'd want and `ARCHITECTURE.md` says what we do first.

## 1. What shapes the design

Contest programs are unusual inputs for a compiler, and the design should
exploit that:

- **Small, whole programs.** One file of 50–500 lines plus the stdlib. Whole
  program analysis is cheap, so we never need separate compilation, an ABI
  or a linker.
- **Two targets with one meaning.** A checked interpreter for local runs and
  one readable C++17 file for the judge. They must agree on every valid
  program.
- **The editor is the main interface.** Most of the time the program is
  half-written and wrong. Every phase must work on broken input.
- **Speed is a correctness property.** Too slow on the judge is a wrong
  answer, so performance and layout are part of the semantics the
  contestant can see, not an optimization detail.
- **The spec is the product.** A program's meaning is defined by numbered
  rules, and the implementation should be traceable to them.

## 2. The shape

```text
           +-------------------- query database (incremental) ------------------+
 text ---> | syntax -> names -> types -> core IR -> facts -> MIR -> opt -> layout |
           +--------------------------------------------------------------------+
                 |          |        |         |          |               |
               fmt/lint   LSP     reference   explain   interpreter    C++ emitter
                                   interp               (VM, traces)   (judge target)
```

Everything is a query over one database. The CLI, the LSP, the formatter,
the stress tool and the docs generator are all clients of the same
queries; none has its own parser or checker.

## 3. Layers

### Syntax: one lossless tree

- A hand-written recursive-descent parser over a lossless concrete syntax
  tree (green/red tree, as in rust-analyzer). Every byte, comment and
  error is in the tree.
- The grammar and the operator table are data, in one file. The spec's
  grammar chapter, the precedence table, the parser tests and the editor
  highlighting are generated from it, so they can't drift.
- Error recovery is designed per construct (statement, block, call,
  pattern), so one typo yields one error and the rest of the file still
  types.
- The formatter is a pure function of the tree. The LSP's rename and code
  actions are tree edits.

### Names and modules

- Modules are files. The stdlib is a set of ordinary modules; the prelude
  is one module whose public names are in scope everywhere.
- Name resolution produces a HIR keyed by stable IDs (module path + item
  path + local index), so an edit inside one fn doesn't invalidate the IDs
  of others. This is what makes per-fn incrementality work.

### Types

- **Bidirectional local inference.** The expected type flows into
  expressions (`read<T>()`, literals, closures, `[D] i => ..`). No global
  unification across fns; signatures are the boundary. Errors stay local
  and messages stay simple.
- **Traits with coherence and no overlap.** One impl per (type, trait).
  Bounds are checked at the definition, so a generic stdlib fn can't fail
  inside its body when instantiated.
- **Laws are compile-time facts.** `law commutative(+)` is an item. Library
  code branches on laws with a compile-time `if`, which is how
  `LazySegTree` drops pushdown for a commutative action. No specialization
  machinery in the type system.
- **Domains are nominal index types.** A position of `V` is its own type;
  product domains, `Pair<V>`, compressed domains and ring ranges are
  library types implementing one `Domain` trait.
- **Range types are refinements.** `1..=n` is `Int` plus an interval fact.
  The checker treats it as `Int` for typing and hands the interval to the
  facts layer. Runtime-bounded ranges (`1..=n` with `n` read) are the same
  mechanism with a symbolic bound.
- **Ownership is flow analysis, not types.** Owned and lent are properties
  of places on the control-flow graph (ch5), checked on core IR. Types stay
  simple; the lending rules get precise, flow-sensitive answers.
- **Error types poison.** An expression that failed to type has type
  `Error`, and no further error is reported about it. That is the
  mechanism behind "one error per root cause".

### Core IR: the executable spec

- After type checking, the surface language is desugared into a small core:
  `let`, assignment to places, calls, `if`, `loop`, `break`, `match` on
  enums, closures, `yield`, and explicit check ops. Every desugaring is a
  numbered spec rule, so the spec chapter "what this means" is literally
  the lowering.
- A **reference interpreter** runs core IR directly. It is slow and simple,
  written to match the spec's wording, and every function cites its rule
  ID. Conformance runs against it first, and the fast engines are tested
  against it. When the spec and the implementation disagree, this is where
  it shows.

### Facts

- One abstract interpretation framework, linear time, over core IR:
  intervals with symbolic bounds, differences between variables (`i < n`),
  non-`None`, domain membership, and "this value was read from input with
  type T".
- Its results serve five clients: check removal, storage widths, overflow
  diagnostics, LSP hover ("`i` is in `0..n` here") and `cpl explain`.
- Facts are never guesses: if the analysis can't prove something, the check
  stays and `explain` says why.

### MIR and optimization

- MIR is a typed CFG per monomorphized fn, close to C: places, moves,
  calls, branches and check ops. Every value carries a source span and a
  source name, so traps, traces and the debugger speak in source terms.
- MIR keeps structured regions (loops, ifs, blocks) rather than raw gotos,
  so the C++ emitter can print code that reads like a contestant's.
- A small, deterministic pass pipeline:
  1. inline small fns and all operation values;
  2. splice generators into their consumers (internal iteration), so a `for`
     over `filter().map()` is one loop with no state machine;
  3. scalar replacement of records and tuples;
  4. remove checks the facts prove;
  5. choose layouts (widths, niches, field order, column layouts);
  6. dead code removal.
- Each pass logs its decisions for `cpl explain`.

### Layout

- One module decides every value's representation from its type and facts:
  width, niche for `?T`, field order, padding, array-of-structs vs
  struct-of-arrays where the type asks for it.
- The interpreter's memory accounting, the C++ emitter and LSP hover
  (sizes, padding, peak bytes against the judge's memory limit) all read
  the same answer, so what the editor shows is what the judge runs.

### Back ends

- **Interpreter.** A register VM over optimized MIR with checks kept: index
  bounds, overflow, `!` on `None`, range stores, input validation. It
  records an event log for time-travel traces and the HTML trace view, and
  keeps per-line counters for the profiler. Compiled to WASM, the same VM
  is the playground.
- **C++ emitter.** Prints optimized MIR as one C++17 file:
  - deterministic names and order;
  - a runtime of small helper snippets, each included only if used;
  - a byte budget tracked per item, so `cpl build` reports what costs space;
  - judge profiles (C++ standard, flags, pragmas, instruction set, stack)
    as input to codegen, not post-processing.
  Release builds drop unproven checks by default; a profile flag keeps them.
- **Agreement by construction.** Both back ends consume the same MIR, and
  the reference interpreter checks both.

### The stdlib

- Written in CP Lang, one module per area, with machine-readable doc labels
  (`cost:`, `empty:`, `ties:`, `order:`, `traps:`, `example:`).
- About 150 intrinsics in one table, each with a VM implementation and a
  C++ snippet: list storage, I/O, bit ops, `I128`, hashing and memory.
  Nothing else in the compiler knows a library name; a gate enforces it.
- Every `example:` is a test. The reference docs, the site and LSP hover
  are generated from the same labels.

### Diagnostics

- A diagnostic is data: code, rule ID, primary and secondary spans,
  message, notes, and machine-applicable fix edits. Renderers turn it into
  terminal output, LSP diagnostics and code actions, or JSON.
- Codes and message templates live in the spec's diagnostics chapter and
  are generated into the compiler, so an undocumented code can't exist.

## 4. Tools as clients

| Tool | Built on |
|---|---|
| `cpl check` / `run` / `build` | the pipeline, with a judge profile |
| LSP | the same queries, with sizes, facts and docs on hover |
| `cpl fmt` | the lossless tree |
| `cpl lint` | typed HIR and facts; lints are rules with IDs |
| `cpl explain` | pass decision logs and layouts |
| `cpl stress` | generators derived from declared input types; brute and checker swaps; shrinking over the input type |
| `cpl trace` / debugger | the VM's event log, mapped to source |
| `cpl test` | a contest folder's samples, with outputs diffed |
| `cpl doc` | the stdlib's doc labels as JSON for the site |
| `cpl judge` | profiles and the submission file |

## 5. Testing architecture

- **Spec as tests.** Every rule ID has conformance cases with exact expected
  output or diagnostics. Coverage is a gate.
- **Three-way differential testing.** Reference interpreter, VM and compiled
  C++ must agree on every conformance case, every corpus test and every
  generated program. Checked and release builds must agree on valid
  programs.
- **Program fuzzing.** A type-directed generator writes valid CP Lang
  programs; disagreements shrink automatically to a minimal case.
- **Metamorphic tests.** Rename a variable, reorder independent items,
  format the file, or inline a fn by hand: the output must not change.
- **Corpus acceptance.** Real problems with tests from accepted C++, run in
  every engine, with runtime and size compared against the original.
- **Budgets as tests, on a quiet machine.** Latency, runtime and size live
  in `just bench`, never in the correctness gate.
- **Snapshots** for diagnostics, LSP, explain and traces, blessed with one
  command.

## 6. Repository

```text
spec/        normative rules, grammar data, diagnostics catalog, conformance cases
std/         the stdlib in CP Lang (one home: signatures, docs, bodies)
crates/      syntax, db, hir, ty, core, facts, mir, opt, layout, vm, cpp, rt, diag, cli, lsp, fmt, lint, stress, doc
corpus/      problems with ports, tests and metadata; raw submissions
bench/       runtime and size harness against matched C++
editors/     VS Code and others, generated grammars
site/        docs and playground, built from spec/ and std/
research/    notes and experiments, non-normative
plan/        plans, non-normative
```

One `mise.toml` pins tools, one `justfile` holds commands, and generated
files are either checked by a test or never committed.

## 7. What it costs to build

Roughly in order, each step usable on its own:

1. syntax, names, types and the reference interpreter for the core
   chapters, with conformance running;
2. MIR, the VM and the stdlib's core modules;
3. the C++ emitter, layout and the judge profiles;
4. facts and check removal;
5. the LSP, explain, stress and traces;
6. the rest of the stdlib, in order of corpus usage.

From scratch, this is weeks of focused work before it matches what today's
compiler already runs, which is why `ARCHITECTURE.md` evolves the existing
code toward this shape instead.

## 8. Where today's code differs most

| Ideal | Today |
|---|---|
| one library tree in CP Lang | two trees: `spec/std` signatures and `stdlib` bodies |
| no library names in the compiler | 63 names special-cased in lowering |
| traits with coherence, checked at definition | generics and traits partial |
| one language | v0.3 behind `CPL_SPEC` |
| reference interpreter on core IR | none; conformance runs on the fast path |
| one VM | tree-walker plus a bytecode VM that falls back |
| grammar as data | grammar in parser code and prose |
| facts as one framework | range reasoning spread across checker and codegen |
