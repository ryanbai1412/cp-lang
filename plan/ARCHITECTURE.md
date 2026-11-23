# CP Lang toolchain architecture

Owner: lead agent. Status: adopted, 2026-11-21; revised
2026-11-21 for maintainability (one IR, one interpreter; see "Dropped from the first plan" in B3).

This file has two halves. Part A is the target: the architecture we would
build if no code existed. Part B is how today's code gets there. The
target decides every design question; today's code never does. When a
piece of today's code doesn't fit the target, we rewrite that piece rather
than bend the target around it.

# Part A: the target

## A1. What shapes the design

Contest programs are unusual inputs for a compiler, and the design exploits
that:

- **Small, whole programs.** One file of 50–500 lines plus the stdlib.
  Whole-program analysis is cheap, so there is no separate compilation, ABI
  or linker.
- **Two targets with one meaning.** A checked interpreter for local runs and
  one readable C++17 file for the judge. They agree on every valid program.
- **The editor is the main interface.** Most of the time the program is
  half-written and wrong, so every phase works on broken input.
- **Speed is a correctness property.** Too slow on the judge is a wrong
  answer, so performance and layout are part of what the contestant sees.
- **The spec is the product.** A program's meaning is defined by numbered
  rules, and the implementation is traceable to them.

**Correctness comes first.** A wrong answer from the compiler is worse than
a slow one or a missing feature, because the contestant can't see it:

- every valid program produces the same output in the reference
  interpreter, the VM and the compiled C++, in checked and release builds;
- every invalid program is rejected with a diagnostic naming its rule, or
  traps in checked builds;
- emitted C++ has no undefined behaviour for valid programs, and sanitizer
  builds check this on every test;
- every miscompile becomes a regression test, and the count of known
  miscompiles is zero.

## A2. The shape

```text
         +------------- query database (incremental) --------------+
 text -> | syntax -> names -> types -> IR -> ownership, facts -> opt |
         +---------------------------------------------------------+
              |         |       |          |              |
          fmt/lint     LSP   explain   interpreter     C++ emitter
                                       (traces, wasm)  (judge target)
```

Everything is a query over one database. The CLI, the LSP, the formatter,
the stress tool and the docs generator are clients of the same queries;
none has its own parser or checker.

## A3. Layers

### Syntax: one lossless tree

- A hand-written recursive-descent parser over a lossless concrete syntax
  tree (green/red, as in rust-analyzer). Every byte, comment and error is
  in the tree.
- **The operator table is data, in one file**; the precedence table in the
  spec and the editor grammars are generated from it. The parser stays
  hand-written; drift between grammar prose and parser is caught by
  parsing every spec code block, not by generating the parser.
- Error recovery is designed per construct (statement, block, call,
  pattern), so one typo yields one error and the rest of the file still
  types.
- The formatter is a pure function of the tree. LSP rename and code
  actions are tree edits.

### Names and modules

- Modules are files. The stdlib is ordinary modules; the prelude is one
  module whose public names are in scope everywhere.
- Name resolution produces a HIR keyed by stable IDs (module path, item
  path, local index), so an edit inside one fn doesn't invalidate the
  others. This is what makes per-fn incrementality work.

### Types

- **Bidirectional local inference.** The expected type flows into
  expressions (`read<T>()`, literals, closures, `[D] i => ..`). No
  unification across fns; signatures are the boundary.
- **Traits with coherence and no overlap.** One impl per (type, trait).
  Bounds are checked at the definition, so a generic stdlib fn can't fail
  inside its body when instantiated.
- **No overloading.** Named and defaulted arguments are resolved before
  type checking.
- **Laws are compile-time facts.** `law commutative(+)` is an item, and
  library code branches on laws with a compile-time `if`. That is how
  `LazySegTree` drops pushdown for a commutative action, with no
  specialization machinery in the type system.
- **Domains are nominal index types.** A position of `V` is its own type;
  product domains, `Pair<V>`, compressed domains and ring ranges are
  library types implementing one `Domain` trait.
- **Range types are refinements.** `1..=n` is `Int` plus an interval fact.
  The checker types it as `Int` and hands the interval to facts. A
  runtime-bounded range (`n` read from input) is the same mechanism with a
  symbolic bound.
- **Error types poison.** An expression that failed to type has type
  `Error`, and nothing further is reported about it. That is the mechanism
  behind "one error per root cause".

### IR: one typed, structured IR

- After type checking, the surface language lowers into one IR: typed,
  monomorphized, structured (loops, ifs, blocks as regions, not gotos),
  close to C: places, moves, calls, `match` on enums, closures, `yield`
  and explicit check ops. Every lowering cites its spec rule ID.
- Every value carries a source span and name, so traps, traces and the
  debugger speak in source terms.
- **Ownership is flow analysis on the IR, not types.** Owned and lent are
  properties of places on the control-flow graph (ch5); errors point at
  the source constructs through spans.
- There is no second "core" or "MIR" layer: one IR is the contract between
  the front end and both back ends, so a language change touches one
  lowering, not three.

### Facts

- One abstract interpretation framework over core IR, linear time:
  intervals with symbolic bounds, differences between variables (`i < n`),
  non-`None`, domain membership, and "read from input with type T".
- It serves five clients: check removal, storage widths, overflow
  diagnostics, LSP hover ("`i` is in `0..n` here") and `cpl explain`.
- Facts are never guesses. If the analysis can't prove something, the
  check stays and `explain` says why.

### Optimization

- A small, deterministic pass pipeline over the IR:
  1. inline small fns and all operation values;
  2. splice generators into their consumers, so a `for` over
     `filter().map()` is one loop with no state machine (a stored
     generator, which is rare, becomes an explicit state machine);
  3. scalar replacement of records and tuples;
  4. remove the checks facts prove;
  5. choose layouts;
  6. remove dead code.
- Each pass logs its decisions for `cpl explain`. The interpreter runs the
  unoptimized IR, so the passes are checked by interpreter-vs-C++
  differential on every test.

### Layout

- One module decides every value's representation from its type and
  facts: width, niche for `?T` (`Int.MIN` for `Int`, inherited through
  fields), field order, padding, and column layouts where the type asks.
- The VM's memory accounting, the C++ emitter and LSP hover (sizes,
  padding, peak bytes against the judge's limit) read the same answer, so
  what the editor shows is what the judge runs.

### Back ends

- **Interpreter: one engine.** A register VM over the IR with every check
  kept: index bounds, overflow, `!` on `None`, range stores, input
  validation. It is the executable spec (plain code, rule IDs cited), the
  debugger, the stress runner and, compiled to WASM, the playground. It
  records the event stream and per-line counters. There is no separate
  tree-walker or reference interpreter to keep in sync; a second engine
  would double the cost of every language change for a cross-check the
  C++ back end already gives.
- **C++ emitter.** Prints the optimized IR as one C++17 file:
  - deterministic names and order;
  - runtime helper snippets included only when used;
  - bytes tracked per item, so `cpl build` reports what costs space;
  - judge profiles (standard, flags, pragmas, instruction set, stack) as
    input to codegen, not post-processing.
- Release builds drop unproven checks by default; a profile flag keeps
  them.
- Interpreter and C++ agree on every test (A6); that differential is the
  correctness check on both.

### The stdlib, outside the compiler

Ryan: "stdlib needs to be cp lang i want as little of stdlib in the
compiler as possible".

- `std/` is CP Lang, one module per area, with machine-readable doc labels
  (`cost:`, `empty:`, `ties:`, `order:`, `traps:`, `example:`). Every
  `example:` is a test; reference docs, the site and LSP hover are
  generated from the same labels.
- **One intrinsics table** (`cpl-rt`), about 150 primitives (list storage,
  I/O, bit ops, `I128`, hashing, memory), each with a VM twin and a C++
  snippet. Growing it needs a decision entry; a test pins the count.
- **Name-ban gate.** A script collects every public name in `std/` and
  fails `just check` when one appears in compiler source outside `cpl-rt`.
- **No-privilege test.** A stdlib module copied into a user library under
  another name type-checks, runs and emits the same C++ as the original.
- **Powers are general.** A speed need becomes a feature any library can
  use (inlining, law dispatch, `@cpp`), never a name check.
- Decorators are the exception that is not library: the closed set in
  spec §6.4 is core language, because each changes meaning (`@memo` table
  sizing, `@get` call syntax, `@cache`/`@trusted` safety).

### Diagnostics

- A diagnostic is data: code, rule ID, primary and secondary spans,
  message, notes and machine-applicable fix edits. Renderers turn it into
  terminal output, LSP diagnostics and code actions, or JSON.
- Codes and message templates live in `spec/diagnostics.md` and are
  generated into the compiler, so an undocumented code can't exist.

## A4. Crates

Crate names follow jobs; a crate whose job already matches keeps its name.

| Crate | Job |
|---|---|
| `cpl-syntax` | operator table, lexer, error-tolerant parser, lossless tree |
| `cpl-check` | query database, names, modules, types, traits, generics, domains, ownership; the front end every tool queries |
| `cpl-ir` | the IR, lowering, monomorphization, facts (absorbing `prover`), passes, layout |
| `cpl-rt` | the intrinsics table: Rust and C++ for each primitive, in one place |
| `cpl-interp` | the one interpreter: register VM, event stream, traces, replay, profiler; WASM build |
| `cpl-codegen` | C++ emitter, runtime tree-shaking, judge profiles |
| `cpl-diag` | diagnostic model and renderers; its JSON shape is printed with `cpl-json`. `plain` turns doc text's LaTeX math and intra-doc links into terminal text for hover, the CLI and lint (`cpl_doc::plain` re-exports it) |
| `cpl-cli` | `cpl check/run/build/test/stress/doc/fmt/lint/explain/judge/dap` |
| `cpl-fmt`, `cpl-lint`, `cpl-doc`, `cpl-stress`, `cpl-judge` | tools over `cpl-check` queries |
| `cpl-lsp` | the language server. `lib.rs` only re-exports `Server` and `serve`; dispatch is in `server.rs`. Hover facts come from `cpl-check/src/hover.rs`; the LSP renders them as markdown. Uses `cpl-check`, `cpl-lint`, `cpl-fmt`, `cpl-judge`, `cpl-ir`, and `cpl-doc` for doc text without Markdown math or links (`cpl_doc::plain`). `transport` framing is shared with the DAP in `cpl-cli/src/dap.rs`. `build.rs` lists the std modules for `use std.` completion |
| `cpl-conform` | one harness: conformance, corpus, doc examples, differential, fuzzing, scoreboard |
| `cpl-contest` | the contest workflow behind `cpl new/test/bench/watch`; the corpus benchmark (perf owns `corpus_bench.rs`) |
| `cpl-json` | one dependency-free JSON value and printer, shared by every tool that speaks JSON |
| `cpl-wasm` | browser bindings over `cpl-check`, `cpl-interp`, `cpl-codegen`, `cpl-lint` and `cpl-lsp`; no logic of its own |

## A5. First-class concerns

Each concern says what every phase must preserve, its budget, and the test
that enforces it. A phase that drops what a concern needs is a bug.

| Concern | Every phase preserves | Budget | Enforced by |
|---|---|---|---|
| Correctness | the spec's meaning through every lowering; checks as explicit ops until proven | zero known miscompiles; same output in every engine and build; no undefined behaviour in emitted C++ | rule-linked conformance; interpreter-vs-C++ differential; checked vs release agreement; sanitizer builds; program fuzzing; corpus acceptance; each bug lands with its regression test |
| Editor (LSP) | per-position queries on partial programs; recovery after errors | edit to diagnostics under 50 ms; front end under 200 ms on the largest port | LSP snapshots and latency bench |
| Diagnostics | spans and rule IDs through lowering; one error per root cause; a fix-it | every code documented and tested | conformance `expect: E…`; undocumented-code test |
| Performance | layout facts, inlining and check elision visible in IR | runtime p50 at most 1.0x originals, max 1.5x, 1.1x always reachable by tuning | `just bench` against matched C++ |
| Size | bytes per emitted item; runtime helpers only when used | under 32 KB per solution, 64 KB hard | corpus size report |
| Interpreter and C++ agreement | one IR meaning; checks as explicit ops | zero disagreements | differential runs on every test and fuzzing |
| Debugging | names and spans on every IR value; one source-level event stream (statement, write, print, trap) from every engine; one display rule per type | time-travel step back under 50 ms at 10^7 steps; trace overhead at most 3x; interpreter at most 10x C++; a planted bug diagnosed from the trace in minutes | event-stream agreement across engines; trace and DAP snapshots; trace viewer browser test; planted-bug drill |
| Explain | why each layout, check and inline was chosen | `cpl explain` covers all of them | explain snapshots |
| Judge targets | profile (standard, flags, stack, ISA) flows to codegen | one profile per supported judge | judge profile tests |
| Determinism | no hash-order or address dependence | identical bytes on rebuild | build twice and compare |
| Stress testing | declared input types become a typed input schema; test roles (`@brute`, `@alt`, `@check`, `@interactor`) are program-model queries; seeded determinism | a generator and constraint-respecting shrinker for every input type; first diverging event reported, not just output; counterexample to a false conjecture in under 10 s | `cpl stress` cases; time-to-counterexample drill |
| Spec traceability | rule ID from spec to diagnostic, test and docs | every rule has a case | `conformance.py --rules` |
| Library power | no name checks in the compiler | intrinsics table only | name-ban gate; no-privilege test |

### Debugging and stress testing

Both are designed in, not bolted onto the interpreter:

1. **One event stream.** The interpreter and a trace build
   of the emitted C++ emit the same source-level events. Traces, the
   debugger, stress alignment and trap reports all consume it, so a
   judge-size input too slow to interpret is traced from compiled C++.
2. **Spans survive every lowering** (IR, optimized IR, C++), so every event maps
   to a source line and variable name.
3. **Time travel by construction.** Runs are deterministic (seeded RNG, fixed
   hash order, recorded input); periodic snapshots make stepping back cheap.
4. **Values display in source terms.** One display rule per type serves
   `print`, traces, the debugger and traps: domain labels on indices, `None`
   for empty optionals, tables over domains as grids.
5. **Input types are a schema.** The front end exports the typed input schema
   from `read<...>` declarations; generators and shrinkers derive from it and
   keep constraints valid while shrinking.
6. **Test roles are queries.** Stress compares solution and brute event by
   event and reports the first diverging state.
7. **Incomplete programs run.** `todo` traps at its location.
8. **Edit, re-run, replay.** One action re-records from the edited file on
   the pinned input; the cursor returns to the same logical position; the
   first step where the old and new runs differ is highlighted. Any recorded
   call can be replayed alone after an edit: state at its entry is rebuilt
   from the recording and only that call runs. One implementation in the
   interpreter serves the editor (DAP), the browser (wasm) and the CLI.

## A6. Testing

- **Spec as tests.** Every rule ID has conformance cases with exact
  expected output or diagnostics. Coverage is a gate.
- **Differential.** The interpreter and compiled C++ agree on every
  conformance case, corpus test and generated program, including the
  event stream in trace builds; checked and release builds agree on valid
  programs.
- **Program fuzzing.** A type-directed generator writes valid programs;
  disagreements shrink to a minimal case.
- **Metamorphic tests.** Renaming a variable, reordering independent
  items, formatting the file or inlining a fn by hand leaves the output
  unchanged.
- **Corpus acceptance.** Real problems with tests from accepted C++, run
  in every engine, with runtime and size compared against the original.
- **Snapshots** for diagnostics, LSP, explain and traces, blessed with one
  command.
- **Budgets on a quiet machine.** Latency, runtime and size live in
  `just bench`, never in the correctness gate.

| Suite | Where it runs |
|---|---|
| unit tests per crate; conformance | `just check` (under a minute, changed crates only) |
| doc examples; corpus; LSP and explain snapshots; bug replay; short fuzz | `just test` |
| long fuzz | nightly |
| runtime and size | `just bench` |

## A7. Repository

```text
spec/        normative rules, grammar data, diagnostics catalog, conformance cases
std/         the stdlib in CP Lang: signatures, docs and bodies in one place
crates/      the crates in A4
corpus/      problems with ports, tests and metadata; raw submissions
bench/       runtime and size harness against matched C++
editors/     VS Code and others, generated grammars
web/         site, docs and playground, built from spec/ and std/
research/    notes and experiments, non-normative
plan/        plans, non-normative
```

One `mise.toml` pins tools, one `justfile` holds commands, and generated
files are either checked by a test or never committed.

# Part B: getting there from today

## B1. Where we are

Measured 2026-11-22; commands in `plan/domains/tech-debt/metrics.toml`
and `plan/domains/compiler/metrics.toml`.

- About 203k lines of Rust in 18 crates
  (`git ls-files 'crates/*.rs' | xargs cat | wc -l`; `ls -d crates/*/`): `cpl-syntax` ->
  `cpl-check` (salsa-style database) -> `cpl-ir` (typed lowering) ->
  `cpl-interp` (a tree-walker plus a register VM that falls back to it)
  and `cpl-codegen` (C++17). A4 now lists `cpl-contest`, `cpl-json` and
  `cpl-wasm`; `prover` (package `cpl-prover`) is still its own crate.
- One language: edition 0.2, its `stdlib/` and the `CPL_SPEC`/`CPL_STD`
  switches are deleted (`edition_refs` 0).
- Conformance 1661/1723; interpreter == C++ on 1273 of 1348 (75 skipped);
  corpus 12/100 ports pass; 19/185 spec blocks pass
  (`cpl-conform conformance corpus spec-blocks diff` on `e069e99de`).
- The stdlib is 40 modules in `std/` with 162 `todo` stubs
  (`ls std/*.cp`; `debt-ratchet.py --show stdlib_todo`).

| Target | Today |
|---|---|
| one `std/` tree | `std/`, bundled into `cpl-check` by `build.rs` |
| no library names in the compiler | 89 names string-matched in `cpl-ir/src/lower/call.rs` (`ir_builtin_names`) |
| operator table as data; every spec block parses | precedence in parser code and prose; 19/185 spec blocks pass |
| coherent traits checked at definition; `Error` poisons | generics and traits partial; cascading errors |
| harness and scoreboard in one place | `cpl-conform`, plus `scripts/corpus-tests.py` |
| ownership on the IR | partial checks in `cpl-check` |
| one facts framework | range reasoning spread across checker and codegen (`prover`, `ranges.rs`); `cpl-lint -> cpl-prover` (`layer_violations` 1) |
| one interpreter | a tree-walker (`cpl-interp/src/lib.rs`) plus a VM (`vm/`) that falls back to it on 13 of 13 runnable corpus programs |
| layout as one module | `niche.rs` and `ranges.rs` inside codegen |
| one test harness | `cpl-conform` plus per-crate tests and scripts |

## B2. Rules that stop path dependence

1. **New code lands in its target crate and target shape.** Nothing new
   goes into a crate on the delete list except a deletion or a bridge.
2. **A replacement deletes what it replaces.** Two implementations of one
   job live only until the new one passes the old one's tests, and the
   deletion is part of the same step.
3. **Bridges are listed.** Any adapter between old and new code is named
   in B3 with the step that removes it.
4. **Reuse is allowed only when the code already has the target shape.**
   Lexer, diagnostics and the C++ helper snippets qualify; anything that
   needs bending to fit is rewritten.
5. **Each step closes a row of the B1 table** and updates it. Part B is
   done when the table is empty, and then it gets deleted.
6. **No step leaves the corpus worse.** Ratchets on conformance, corpus
   and spec-block counts stop regressions.

## B3. Steps

Ordered by how much each removes from what a maintainer must understand.
Each step ends on a test count, deletes what it replaces and is usable on
its own.

1. **Harness** (`cpl-conform`): conformance, corpus (expected outputs from
   the original C++), doc examples and interpreter-vs-C++ runs, one
   scoreboard JSON with ratchets. Replaces `conformance.py`,
   `corpus-tests.py`, `scoreboard.py` and the ad hoc runners.
2. **One interpreter.** Every IR construct gets a VM lowering; the
   tree-walker and the VM's fallback path are deleted, along with the
   duplicated value and trap code. The debugger, traces, stress and WASM
   run on the VM; the DAP recording becomes the delta log plus periodic
   snapshots.
3. **`cpl-rt` and the name-ban gate.** The 89 special-cased names in
   `lower/call.rs` move into `std/` as CP Lang or into the intrinsics
   table; the gate and the no-privilege test turn on. `lower/call.rs`
   shrinks to call resolution.
4. **One library tree.** `std` moves to `std/`; the remaining 0.2
   ports (`v02_port_files`) are deleted or rewritten. (One mode is done:
   0.2, `CPL_SPEC`, `CPL_STD` and `stdlib/` are deleted.)
5. **Front end in place.** `cpl-check` gets coherent traits, local
   inference and `Error` poisoning as modules of the same crate; the whole
   of `std/` type-checks as a library.
6. **Facts and layout.** `prover`, codegen's `ranges.rs` and `niche.rs`
   become the facts and layout modules of `cpl-ir`; codegen and the
   interpreter read their answers.
7. **Tools on the queries:** LSP, fmt, lint, doc, stress and judge stop
   re-deriving anything the front end knows.
8. **Stdlib bodies** fill in throughout, in order of corpus usage: a
   module is done when its doc examples and conformance cases pass.

Dropped from the first plan, and why:
- **Separate reference interpreter and core IR.** Two engines and two IRs
  to keep in step with every language change; interpreter-vs-C++
  differential gives the cross-check at no extra maintenance.
- **Separate MIR, `cpl-opt`, `cpl-vm`, `cpl-cpp`, `cpl-hir`, `cpl-ty`
  crates.** Renames and splits with no change in behaviour; module
  boundaries inside the existing crates give the same structure.
- **Parser generated from grammar data.** Parsing every spec block
  catches the same drift.

## B4. Workflow

- I own the architecture and write the spine: syntax through the IR, the
  interpreter and the emitter. Interfaces between crates
  are settled before anyone else writes against them.
- Delegated work starts only where an interface is fixed and a test suite
  defines done: stdlib modules, tools, the bench harness. Each piece is
  small, time-capped and reviewed by me before it lands.
- Commits are small and go to `main`, each passing `just check`.

## B5. Risks

- **Scope.** 1,982 stdlib functions. Bodies are filled in order of corpus
  usage, and there is a working compiler after every step.
- **Trait solver complexity.** Kept simple on purpose: no overlapping
  impls, no user-visible specialization, law dispatch by compile-time `if`.
- **Monomorphization compile time.** Measured on the largest ports, with a
  budget test.
- **Rewrite drag.** Steps 4, 5 and 8 replace large crates. Each runs both
  old and new on the corpus until the new one matches, then deletes the
  old one in the same step (B2.2).

## B6. Re-architectures in flight

Short-lived: each paragraph folds into Part A or the owning domain's
ARCHITECTURE.md as its steps land, and goes when its ratchet reads 0.
Each targets a bug class that recurred on 2026-11-23. The ratchets are in
`scripts/debt-ratchet.py` and only go down.

### 1. One engine (M1; tech-debt with compiler)

B3 step 2. Every IR construct runs on the VM, traced or not, and the
tree-walker is deleted. Ratchet: `vm_fallbacks`, the tree-walker fallback
kinds over `crates/cpl-cli/tests/programs` (242 at the start, 149 today),
to 0. Tech-debt moves traced statements, checkpoints and fed loops. Compiler
moves the untraced fallbacks and the runs that bypass the VM.

### 2. One analysis pipeline (M1)

Bug class: tools that each re-derive what the checker knows, and drift
apart. On 2026-11-23 there were two CLI/LSP divergence reds, and L0206 and
L0513 guessed from syntax what E0304 and E0201/E0204 already report.

- **Target.** `cpl-check` alone answers "what does this file mean": one
  session API (`Database::analyze` plus `diagnostics`, each diagnostic with
  its fixes as edits) and one facts API (types, ranges, effects, resolved
  std items). Lints read only those facts and the lossless tree, and a lint
  never reports what a checker code covers. The CLI, LSP, wasm, `cpl fix`,
  stress and the contest loop open the same session through one entry per
  job (below), never their own `Database::new()`.
- **Steps.**
  1. compiler: a `cpl_check::Session` (file set, loader, analyze,
     diagnostics, facts) that the current `Database::new()` callers can take
     as they are.
  2. tools: `cpl-cli`, `cpl-lsp`, `cpl-contest` and `cpl-wasm` open sessions;
     `apply_error_fixes_in` and `cpl_lsp::checked_fix` become one fix path.
  3. lint-fmt: `cpl-lint` takes a session's facts instead of building its
     own (`checker.rs`, `facts.rs`, `lib.rs`); each rule that restates a
     checker code is retired for that code's fix, as L0206 and L0513 were.
  4. diagnostics: `cpl_check::fix` re-checks and applies in rounds over
     `cpl_diag::apply_fixes`, and a checker diagnostic carries its fixes, so `cpl fix`, the
     editor's fix-all and the playground apply the same edits.
  5. tools (stress, explain, trace, migrate) and perf (`checks.rs`): the
     remaining sites.
- **Ratchets.** `own_databases`: `Database::new()` outside tests and
  `cpl-check` (36 in 22 files today), to 0. `lint_checker_overlap`: lint
  findings on the lint fixtures and corpus at the span of a checker
  diagnostic, to 0 (built with the detectors).

### 3. Std semantics out of the compiler (M2; starts now)

Bug class: the compiler deciding by name what a std fn means, so the std
body is not what runs and each name needs its own case in the checker,
lowering, interpreter and emitter.

- **Target.** The compiler knows a fixed table of intrinsics (machine
  operations with no CP Lang body), stated once as data and read by the
  checker, lowering, the VM and the emitter. Everything else is a std body
  with library mechanisms open to user code. Op values (`+`, `min`, a
  closure given as a `Monoid` op) are monomorphized, so a std body costs
  what the builtin did.
- **Steps.**
  1. compiler: the intrinsics table as data; the arms in
     `lower/call.rs` and `lower/call/*.rs` that are intrinsics read it.
  2. compiler: op values and fn-valued parameters monomorphized per known
     closure, so step 3 meets the perf gate.
  3. stdlib: each remaining name moves into its std body, in corpus-usage
     order, behind the gate (interpreter and C++ within 1.1x of the builtin
     on a matched kernel, same output).
  4. lint-fmt: lint rules match std items by resolved path, not by name.
  5. perf: the emitter's std-name cases (`backend_std_names`) go with the
     builtins they served.
  6. stdlib: the no-privilege test: a std module copied into a user
     library runs the same.
- **Ratchets.** `ir_builtin_names` (163 today, counting tuple arms since
  decision 410) to the intrinsics count, then the intrinsics leave the measure;
  `backend_std_names` 18, `prelude_privilege` 24 and `lint_std_names` 180,
  each to 0.

### 4. One copy and effect analysis in codegen (M2; starts now)

Plan in `plan/domains/perf/ARCHITECTURE.md` ("Re-architecture #4"): one
Effects table (compiler) and one Uses table (perf), lend by default, a copy
only with a reason `cpl explain` prints, rewrites gated on proven purity.
Ratchets: `cpp_deep_copies` 101 and `cpp_std_functions` 777, from
`crates/cpl-cli/tests/read_only_copies.rs`, each to 0 or to the copies a
reason requires.

### One entry point per job

Each job has one public entry, named in its owner's ARCHITECTURE.md. The
CLI, LSP, wasm and the other tools call it and add only presentation.
`crates/cpl-cli/tests/lsp_parity.rs` grows from check and lint to every
row (owner tools).

| Job | Entry (target) | Today | Owner |
|---|---|---|---|
| check | `cpl_check::Session::diagnostics` | 36 `Database::new()` sites | compiler |
| lint | `cpl_lint::lint` on a session's facts | `lint`, `lint_parsed`, `lint_named` build their own | lint-fmt |
| fix | `cpl_check::fix` (checked, in rounds), over `cpl_diag::apply_fixes` (text edits) | `apply_error_fixes_in` (cli), `checked_fix` (lsp), `fix_checked_removed_syntax`, `apply_fixes` (lint) | diagnostics |
| run | `cpl_interp::run_with` | `run`, `run_with`, `run_observed`, `run_profiled` | compiler |
| trace | `cpl_interp::observe_on_this_thread` | plus `checkpoint::record_*` | debugger |
