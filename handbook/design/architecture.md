# Architecture: the target

This is the architecture we would build if no code existed. It decides
every design question; today's code never does. When a piece of code
doesn't fit, that piece is rewritten rather than the target bent around it.
[`overview.md`](overview.md) is the short version; the steps from today's
code to this target are in `plan/rearchitecture.md`.

## What shapes the design

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

## The shape

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

## Layers

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
- There is one IR data type (`cpl-next-ir`), not a second "core" or
  "MIR" language. Its forms in `handbook/design/next/README.md` (generic,
  instances, raw, canonical, optimized) are the same types under stricter
  invariants, each checked at its boundary. The typed tree is the
  checker's output, not an IR. So a language change touches one lowering,
  not three.

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
- Interpreter and C++ agree on every test (Testing); that differential is the
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

## Crates

Crate names follow jobs; a crate whose job already matches keeps its name.

| Crate | Job |
|---|---|
| `cpl-syntax` | today's error-tolerant parser and its AST, over `cpl-next-syntax`'s lexer and item split; the old pipeline and the tools read it, and no `cpl-next` crate does, and it is deleted with the old pipeline (handbook/design/next/syntax.md) |
| `cpl-next-syntax` | the one syntax layer: the lexer, whose tokens with their trivia are every byte of the file, the item split, and the green/red lossless tree (handbook/design/next/syntax.md); depends on `cpl-diag` only, and `cpl-syntax` and every `cpl-next` layer read it |
| `cpl-check` | query database, names, modules, types, traits, generics, domains, ownership; the front end every tool queries. Reads the judge's memory limit from `cpl-judge` for E0904; reads `CPL_STD_DIR` with `cpl-env`; asks `cpl-prover` for inputs that overflow `Int` (`Overflow`; native only, decision 235), so lint reads that fact here; checks `@intrinsic` bodies against `cpl-rt` |
| `cpl-ir` | the IR, lowering, monomorphization, facts (absorbing `prover`), passes, layout; lowers an `@intrinsic` call to its `cpl-rt` primitive |
| `cpl-next` | the pipeline that replaces `cpl-ir`, `cpl-interp` and `cpl-codegen`, grown beside them: one crate per layer, so Cargo's edges hold the layer order (crate rules, `layer order`). `cpl-next` holds no layer of its own: it runs them in order, over `cpl-next-syntax`'s tree with `cpl-diag`'s codes, the declarations in `cpl-next-decls`, the typed tree in `cpl-next-check`, ownership in `cpl-next-own`, the generic IR's lowering in `cpl-next-lower`, link and the canonical IR in `cpl-next-link`, the reference interpreter in `cpl-next-interp` and the C++ emitter in `cpl-next-cpp`, the IR the layers share in `cpl-next-ir` and the fast engine in `cpl-next-engine`. Its facade holds the query database of `cpl-next-db`, and its `program(Job)` query (`linked.rs`) builds a program. Its inputs arrive in the snapshot (contract 0): the judge is whatever the tool that builds the snapshot resolved (`--judge NAME=MB` on its binary), and std is the one baked into it at build time or the directory `--std DIR` names. Its facade reads doc comments with `cpl-next-syntax`'s one doc reader. It depends on no tool crate (`next_layer_foreign_deps`). No crate depends on it (`tests/unshipped.rs`): its tests judge it on every conformance case (read with `cpl-conform`), and the landing that makes `cpl` use it deletes what it replaces Its facade's snapshot holds contract 0's files as `cpl-contract` types. |
| `cpl-next-decls` | the new pipeline's first layer (contract 2): the modules a program loads, each item by its path with spans counted from its start (`modules`, over `cpl-next-ir`'s `ids`), records, declarations and what each name means without looking inside a body (`decls`), which types are copied (`plain`), and the diagnostic every layer reports (`diag`); `modules` reads each file's items as `cpl-next-syntax`'s typed tree through `cpl-next-db`'s item queries, keyed by `cpl-contract`'s `FileId`; with `cpl-diag`'s codes and `cpl-next-ir`'s types, the library roots under `[libs]` read with `cpl-config`, and core's type names from `cpl-contract`'s `Prim` |
| `cpl-next-check` | the new pipeline's checker (contract 3): declarations to the typed tree, deciding every type, target and conversion so nothing after it infers; the typed-tree walks later layers share (`free_vars`, `group_free`); over `cpl-next-syntax`'s typed tree and operator kinds (`ops`) and `cpl-diag`'s codes, the declarations in `cpl-next-decls` and `cpl-next-ir`'s types, and the judge's name and memory limit for E0904 as plain `cpl-next-ir` data (`build::Judge`) |
| `cpl-next-own` | the new pipeline's ownership check: what each instance link made reads, writes and holds, and which passed places are copies; and, on the generic IR before link (`lent`), which values are lent rather than moved, the marks link keys instances by. Its one entry, `link`, runs those marks, `cpl-next-link`'s link, the check and the copies in that order. Over `cpl-next-ir`'s linked program with the declarations in `cpl-next-decls` and `cpl-diag`'s codes |
| `cpl-next-lower` | the new pipeline's lowering (contract 4): `cpl-next-check`'s typed tree to the generic IR in `cpl-next-ir`, with what each call lends read from its signature and the declarations in `cpl-next-decls`; `cpl-next-syntax`'s operator kinds (`ops`) and `cpl-diag`'s spans |
| `cpl-next-link` | the new pipeline's whole-program link (contracts 5 and 6): the generic IR to instances, raw then canonical, as `cpl-next-ir`'s `Program`, with the declarations in `cpl-next-decls` and `cpl-diag`'s codes. What it needs of the program beyond its fns (domains, globals, set adds, deferred errors) comes from lowering as `cpl-next-ir`'s `whole::Whole`, so it reads no typed tree (`link_reads_check`, 0) |
| `cpl-next-cpp` | the new pipeline's C++ emitter (contract 7): `cpl-next-ir`'s optimized instance IR to one C++ file, its runtime helpers tested against `cpl-next-ir`'s `ops` |
| `cpl-next-interp` | the new pipeline's reference interpreter (contract 7): runs `cpl-next-ir`'s instance IR, raw or canonical, with `ops`' arithmetic and faults; the oracle the fast engine and the C++ are judged against |
| `cpl-next-ir` | the new pipeline's IR, split out of `cpl-next` so every layer and engine reads one definition: the types (`ty`), generic and canonical IR (`ir`), the linked program (`prog`), and what each operation computes and how it fails (`ops`: one table the reference interpreter, the engine and the C++ emitter read, faults as a typed enum with their codes); depends on `cpl-diag`, on `cpl-contract` for the `FileId` an `ItemId` names and core's primitive types (`Prim`), and on `cpl-prim` for which primitive an `@intrinsic` names, and only the `cpl-next` crates depend on it |
| `cpl-next-db` | the new pipeline's query database (`db.rs`: inputs, memoized queries, early cutoff, cycles) and the item queries every layer keys on (`items.rs`: `parse(FileId)`, `items(FileId, module)`, `item(ItemId)`, `item_at(ItemId)`, `module_file(root, module)` over `cpl-next-syntax`'s tree, ids from `cpl-next-ir`, files as `cpl-contract`'s `FileId`, and `parse`'s syntax errors as `cpl-diag`'s diagnostics); below the layers, so each layer defines its own queries in its own crate (handbook/design/next/incremental.md) |
| `cpl-next-engine` | the fast engine: canonical IR compiled to code over word registers and one flat stack, storage as counted heap objects shared until written; operations from `cpl-next-ir`'s `ops`, so it agrees with the reference interpreter and the C++ by construction; `tests/programs.rs` runs it on every case and requires the reference interpreter's output, exit and events; depends on `cpl-next-ir` only |
| `cpl-prim` | the primitives `@intrinsic` names (spec R12.14): each one's id and stable number; depends on nothing |
| `cpl-rt` | the intrinsics table: Rust and C++ for each primitive, in one place (decision 573); depends on `cpl-prim` for the primitives |
| `cpl-interp` | the one interpreter: register VM, event stream, traces, replay, profiler; WASM build; reads `CPL_VM_STATS` with `cpl-env`; runs `@intrinsic` primitives and words their traps through `cpl-rt` |
| `cpl-codegen` | C++ emitter, runtime tree-shaking, judge profiles; reads the C++ library names it writes from `cpl-rt` |
| `cpl-facade` | the one way a tool checks, builds and runs a program (contract 9, migration stage 2): an immutable `Snapshot` of the sources, a `Session` that answers from it, one `Diagnostic`, one `run`, one `emit_cpp`. Today it is over `cpl-check`, `cpl-ir`, `cpl-interp` and `cpl-codegen`, with `cpl-diag`'s diagnostic; `today` holds the read-only views of today's compiler (analysis, hover facts, IR, traces, replay) that tools still read and that move behind final queries in stages 3 to 8. `crates/cpl-wasm/tests/one_snapshot.rs` holds every tool to the same diagnostics and output for one snapshot Its contract data (`FileId`, `Hover`, `Definition`, ...) is `cpl-contract`'s. |
| `cpl-contract` | the data contracts 0 and 9 pass between a tool, the facade and the pipeline (`FileId`, `Text`, `World`, `Hover`, `Definition`, `Typed`, and `Prim`, core's primitive types, the one list the checker reads their names from): one definition each, so cpl-facade's API names the same types whichever pipeline answers it; depends on `cpl-diag` |
| `cpl-diag` | diagnostic model and renderers; its JSON shape is printed with `cpl-json`. `plain` turns doc text's LaTeX math and intra-doc links into terminal text for hover, the CLI and lint (`cpl_doc::plain` re-exports it) |
| `cpl-cli` | `cpl check/run/build/test/stress/doc/fmt/lint/explain/judge/dap`; `cpl run --live` waits on file changes with `cpl-wait`; reads its `CPL_*` settings with `cpl-env` |
| `cpl-fmt`, `cpl-lint`, `cpl-doc`, `cpl-stress`, `cpl-judge` | tools over `cpl-facade` ; `cpl-doc` reads doc comments with `cpl-next-syntax`'s one doc reader (`cpl_next_syntax::doc`); `cpl-fmt` and `cpl-lint` read their `cp.toml` tables through `cpl-config`; `cpl-lint` prints its registry (`cpl lint --list --json`) with `cpl-json`; `cpl-stress` runs `cpl-lint` so a clean report can name the bound lints whose witnesses its small cases miss, and waits on its programs with `cpl-wait`; `cpl-stress` and `cpl-judge` read their cache settings with `cpl-env` |
| `cpl-lsp` | the language server. `lib.rs` only re-exports `Server` and `serve`; dispatch is in `server.rs`. Hover facts come from `cpl-check/src/hover.rs`, read through `cpl-facade`; the LSP renders them as markdown, and reads a doc's facts and costs with `cpl-next-syntax`'s doc reader. Uses `cpl-facade`, `cpl-lint`, `cpl-fmt`, `cpl-judge`, `cpl-contest` for the samples and cases its lenses run (`cpl_contest::problem`). Finds the home folder with `cpl-env`, for the cache a std module is opened from. The lenses (`server/lens.rs`), `cpl/command`/`cpl/commands` (`server/command.rs`) and `workspace/executeCommand`'s `cpl.exec` (`server/exec.rs`, which waits on its `cpl` with `cpl-wait`) run `cpl`, so they and `cpl-contest` are not on wasm32, where there is no `cpl` and no files, and `cpl-doc` for doc text without Markdown math or links (`cpl_doc::plain`). `transport` framing is shared with the DAP in `cpl-cli/src/dap.rs`. `build.rs` lists the std modules for `use std.` completion |
| `cpl-conform` | one harness: conformance, corpus, doc examples (read by `cpl-next-syntax`'s doc reader), differential, fuzzing, rewrites (cpl-lint fixes and cpl-fmt keep output), scoreboard; lists the corpus with `cpl-corpus` and makes a seeded package's cases with `cpl-contest`'s `package.rs` before it reads them, so a stale case never judges; judges an interactive corpus case with `cpl-stress`'s interactor judge (decision 556), and waits on its children and on another worker's build with `cpl-wait`; reads its `CPL_CONFORM_*` settings with `cpl-env` |
| `cpl-contest` | the contest workflow behind `cpl new/test/bench/watch`; the corpus benchmark (perf owns `corpus_bench.rs`); the one expansion of a package's `tests.toml` into its cases (`package.rs`, which `cpl test`, `scripts/corpus.py make`, `cpl-conform` and the tests share), reading `CPL_GXX_FETCH` with `cpl-env`; `cpl watch`, the benchmark's runs and the fetcher's page rate wait with `cpl-wait` |
| `cpl-json` | one dependency-free JSON value and printer, shared by every tool that speaks JSON |
| `cpl-config` | the one dependency-free reader of `cp.toml` (`cpl-check`'s and `cpl-next`'s `[libs]`, over disk or the browser's files; `cpl-fmt`, `cpl-lint`, `cpl-judge`, `cpl-contest`'s `[limits]` and template, `cpl-lsp`'s project root): finds the nearest one, yields a table's `key = value` lines, and holds `SCHEMA`, every table and key a tool reads |
| `cpl-corpus` | the one dependency-free Rust corpus walker: the problem packages under `corpus/problems`, listed as `scripts/corpus.py ls` lists them |
| `cpl-wait` | the one dependency-free place a tool waits: a child's exit (a thread owns `Child::wait`, the caller blocks with a deadline), a file change, a rate limit; debt-ratchet's `poll_sleeps` fails a sleep anywhere else |
| `cpl-env` | the one dependency-free reader of `CPL_*` environment variables: each is a typed, documented `Var` (`ALL` lists them); the one home-folder lookup (`home_dir`); debt-ratchet's `cpl_env_reads` fails a `CPL_*` read anywhere else |
| `cpl-wasm` | browser bindings over `cpl-facade`, `cpl-lint` and `cpl-lsp`; no logic of its own |

### Crate rules

`scripts/debt-ratchet.py` reads this table (`layer_violations`,
`backend_std_names`, `consumer_rederived`, `tool_compiler_deps`); it keeps no copy.

| Rule | Crates |
|---|---|
| back end (reads the IR, not the checker) | `cpl-ir`, `cpl-interp`, `cpl-codegen` |
| consumer (asks the checker, never re-derives its facts) | `cpl-lint`, `cpl-lsp`, `cpl-cli`, `cpl-stress`, `cpl-codegen`, `cpl-wasm` |
| tool (runs and checks a program through the facade only, contract 9; `tool_compiler_deps` counts what is left) | `cpl-cli`, `cpl-lsp`, `cpl-lint`, `cpl-fmt`, `cpl-stress`, `cpl-wasm`, `cpl-contest`, `cpl-doc`, `cpl-judge` |
| compiler (behind the facade; `cpl-syntax` is one too, and tools leave its tree in migration stage 8) | `cpl-check`, `cpl-ir`, `cpl-interp`, `cpl-codegen`, `cpl-prover`, `cpl-rt` |
| forbidden edge | `cpl-codegen -> cpl-check`, `cpl-interp -> cpl-check`, `cpl-wasm -> cpl-cli`, `cpl-lint -> cpl-prover` |
| layer order (the new pipeline, bottom first: a crate reads only the crates before it, and crates joined by + are siblings that read neither each other) | `cpl-next-syntax + cpl-next-ir`, `cpl-next-db`, `cpl-next-decls`, `cpl-next-check`, `cpl-next-lower`, `cpl-next-link`, `cpl-next-own`, `cpl-next-cpp + cpl-next-interp + cpl-next-engine`, `cpl-next` |

## First-class concerns

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
| Stress testing | declared input types become a typed input schema; test roles (`@alt`, `@check`, `@interactor`) are program-model queries; seeded determinism | a generator and constraint-respecting shrinker for every input type; first diverging event reported, not just output; counterexample to a false conjecture in under 10 s | `cpl stress` cases; time-to-counterexample drill |
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

## Testing

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

## Repository

```text
spec/        normative rules, grammar data, diagnostics catalog, conformance cases
std/         the stdlib in CP Lang: signatures, docs and bodies in one place
crates/      the crates in Crates
corpus/      problems with ports, tests and metadata; raw submissions
bench/       runtime and size harness against matched C++
editors/     VS Code and others, generated grammars
web/         site, docs and playground, built from spec/ and std/
research/    notes and experiments, non-normative
plan/        plans, non-normative
```

One `mise.toml` pins tools, one `justfile` holds commands, and generated
files are either checked by a test or never committed.

## Changes

- 2026-11-23: moved here from Part A of `plan/ARCHITECTURE.md`.
