# The v0.3 compiler: architecture and how we implement the spec

Owner: Devin (parent session). Status: proposal, 2026-11-21.

## 1. Where we are

- About 170k lines of Rust in 17 crates. The pipeline is `cpl-syntax` ->
  `cpl-check` (salsa-style database) -> `cpl-ir` (typed lowering) -> two
  back ends: `cpl-interp` (a tree-walker plus a bytecode VM that falls
  back to it) and `cpl-codegen` (C++17).
- It runs all 100 corpus ports in their 0.2 form and meets 168 of 503
  conformance cases.
- The structural problems, which incremental fixes don't remove:
  1. **The stdlib lives in the compiler.** `cpl-ir/src/lower/call.rs`
     (10k lines) hard-codes library operations, the C++ runtime spells
     them a second time, and the interpreter a third. This breaks "no
     privileged stdlib", and every library function costs three
     implementations.
  2. **No real generics or traits.** The checker says "borrows, value
     ranges and overflow, generics and traits are later phases". The v0.3
     library is generic (`Next<V>`, `Indexed<K, T>`, operation values), so
     it can't be written today.
  3. **Two languages in one checker,** switched by the `CPL_SPEC`
     environment variable.
  4. **Two interpreters** with different semantics on the edges.
  5. **Signatures in two places:** `spec/std/*.cp` and the Rust
     hard-coding.

## 2. Design principles

1. **One meaning.** The source is lowered once into a typed,
   monomorphized middle IR (MIR). The interpreter runs it and the C++ back
   end prints it. Checked and release builds differ only in which check
   ops are kept.
2. **The stdlib is CP Lang.** `spec/std/*.cp` becomes the library source:
   the same files gain bodies. Primitives (`List` storage, I/O, bit ops,
   `I128`) are `@cpp` intrinsics with an interpreter twin, about 150 of
   them in one table (`crates/cpl-rt`). Everything else (`SegTree`,
   `Graph`, `dijkstra`, windows, strings) is ordinary CP Lang that user
   libraries could also write.
3. **Zero-cost abstraction by construction.** Monomorphization, inlining
   of small fns and operation values, and generators lowered as internal
   iteration: a `for` over a `yields` fn splices the loop body at each
   `yield`, so no state machine exists in the hot path. Domains,
   positions and optionals are plain integers in MIR.
4. **Layout is a pass, not a convention.** One `layout` module computes
   widths, niches, field order and padding from types and range facts. The
   C++ back end, the interpreter's memory accounting and LSP hover all
   read it.
5. **Incremental and error-tolerant from day one.** A lossless syntax
   tree and query-based analysis serve the CLI and the LSP. Every phase
   keeps going after an error.
6. **Spec-first, test-first.** Every rule ID has a conformance case. A
   phase is done when its chapters' cases pass, not when the code looks
   complete.

### Keeping the stdlib out of the compiler

Ryan: "stdlib needs to be cp lang i want as little of stdlib in the
compiler as possible". Enforced mechanically, from the first commit of the
new spine:

1. **One intrinsics table** (`cpl-rt`), about 150 primitives, each with a
   Rust twin and a C++ snippet. Growing it needs a decision entry; a test
   pins the count.
2. **Name-ban gate.** A script collects every public name in `std/` and
   fails `just check` when one appears in compiler source outside `cpl-rt`.
3. **No-privilege test.** A stdlib module copied into a user library under
   another name type-checks, runs and emits the same C++ as the original.
4. **Powers are general.** A speed need becomes a general feature any
   library can use (inlining, law dispatch, `@cpp`), never a name check.
5. **Tracked.** The scoreboard reports the intrinsic count and stdlib
   names found in compiler code (target 0).

## 3. Pipeline and crates

```text
source --cpl-syntax--> CST (lossless) + AST
       --cpl-hir-----> names resolved, modules, prelude from std/prelude.cp
       --cpl-ty------> types, traits, generics, inference (local, R02.10)
       --cpl-mir-----> typed CFG per monomorphized fn; moves, lends, checks explicit
       --cpl-own-----> ownership and lending on MIR (ch5 R1-R10, E06xx)
       --cpl-facts---> intervals, positions, non-None, overflow proofs (ch3, ch4, ch9)
       --cpl-opt-----> inline, generator splice, check elision from facts
       --cpl-layout--> widths, niches, field order, SoA (ch3)
       -> cpl-run (interpreter over MIR, one engine)
       -> cpl-cpp (C++17 emitter, runtime tree-shaken, 32 KB budget)
```

| Crate | Job | Reuse from today |
|---|---|---|
| `cpl-syntax` | lexer, error-tolerant parser, CST/AST, precedence generated from one table | lexer and much of the parser (it already parses most v0.3 syntax) |
| `cpl-db` | query database: files, modules, cached phase results | `cpl-check/db.rs` design |
| `cpl-hir` | name resolution, `use`, prelude, decorators, contexts | new |
| `cpl-ty` | type checker and trait solver: coherence (one impl per type and trait), generics, range types, domains, `?T`, operation values, record rows | parts of `cpl-check/ty.rs` |
| `cpl-mir` | MIR definition and lowering from typed HIR; monomorphization | new; ideas from `cpl-ir` |
| `cpl-own` | lending and move checker on MIR | new |
| `cpl-facts` | interval and relational facts, linear time | `prover` where it fits |
| `cpl-layout` | sizes, niches, column layouts | `cpl-codegen/niche.rs`, `ranges.rs` |
| `cpl-rt` | the intrinsic table: Rust for the interpreter, C++ snippets for the emitter | C++ helpers in `cpl-codegen/runtime.rs` |
| `cpl-run` | MIR interpreter: checked semantics, traps with source values, traces, profile | trap and trace formats from `cpl-interp` |
| `cpl-cpp` | C++ emission from MIR, deterministic, size-aware | emit helpers, naming, judge glue |
| `cpl-diag` | diagnostic model and renderer; codes from `spec/diagnostics.md` | keep as is |
| `cpl-cli` | `cpl check/run/build/test/stress/doc/fmt/lint/explain/judge` | keep the command surface |
| `cpl-lsp` | LSP over `cpl-db`: hover with layout, completion from stub docs, go-to, rename | transport and snapshot tests |
| `cpl-fmt`, `cpl-lint` | on the CST and on typed HIR | port rules, drop the dead ones |
| `cpl-doc` | reference from the stdlib doc labels (JSON for the site) | keep, repoint to `std/` |
| `cpl-stress`, `cpl-judge` | stress, shrinking, interactor harness, judge profiles | keep, rewire to the new driver |
| `cpl-conform` | test harness: conformance, corpus, doc examples, differential | replaces scattered runners |

Deleted when the new path replaces them: `cpl-check`, `cpl-ir`,
`cpl-interp`, `cpl-codegen`, `cpl-verify`, the `CPL_SPEC` switch, `proto/`.

## 4. Key designs

- **Generics:** monomorphize everything. Trait bounds are checked at the
  definition (no C++-template-style late errors). Inference is local and
  bidirectional, with the expected type flowing into calls (`read<T>()`,
  literals, closures). No overloading. Named and defaulted arguments are
  resolved before type checking.
- **Domains:** a nominal type per declaration; positions are `u32`/`u64`
  ordinals in MIR. Product domains, `Pair<V>`, compressed domains and ring
  ranges are library types over one core `Domain` trait (R12) with
  `@cpp` fast paths only where measured.
- **Operation values and laws:** an operation value is a zero-sized type
  carrying its fn and its declared laws (`law` items). Structures dispatch
  on laws at compile time, so `LazySegTree` with a commutative action
  drops pushdown by specialization in library code, not in the compiler.
- **Generators:** `yields T` fns become internal-iteration MIR. When the
  consumer is a `for`, an adapter chain (`map`/`filter`/`zip`), or a fold,
  the body is spliced in. A stored generator, which is rare, becomes an
  explicit state machine.
- **Checks:** every potentially trapping operation is a MIR op with a rule
  ID. `cpl-facts` removes the proven ones. Release builds drop the
  unproven ones by default (decision: no release traps), and a profile
  option keeps them.
- **Errors:** one per root cause. Every diagnostic names its rule and has
  a fix. Codes and messages come from `spec/diagnostics.md`, and a test
  fails on any undocumented code.
- **Performance budget:** front end under 200 ms on the largest port;
  emitted C++ under 32 KB; runtime p50 at or below 1.0x the C++ originals.
  `cpl explain` shows layouts, elided checks and inlining.

## 5. How the spec is implemented

The order follows dependencies, and each step ends on a test count:

1. **Harness first** (`cpl-conform`): one runner for conformance cases,
   corpus problems (expected outputs from the original C++ on stored
   tests), stdlib `example:` lines as tests, and interpreter-vs-C++
   differential runs. Output is a scoreboard JSON plus ratchets: no count
   may drop.
2. **Syntax** to all ch1 cases; every spec code block parses (`spec-blocks`
   ratchet goes to 100% parse).
3. **HIR and types** to ch2, ch4, ch8 and ch11; the whole of `spec/std`
   type-checks as a library. That last check is the gate for the library:
   the stubs must be internally consistent.
4. **MIR and interpreter** for the core; then ch5–7, ch9 and ch10 cases.
5. **Stdlib bodies,** module by module in dependency order: prelude, seq,
   io, math/mod, containers, ranges (prefix, fenwick, segtree, lazy),
   graph, tree, flow, strings, geometry, the rest. A module is done when
   its doc examples run and its conformance cases pass.
6. **C++ back end** from MIR, with `cpl-layout`; then differential tests
   on every conformance case and corpus problem.
7. **Corpus:** every `spec.cp` runs in both engines and matches stored
   expected outputs; the size and time budget is measured against the
   original C++.
8. **Tools switch over:** LSP, fmt, lint, doc, stress and judge move to
   `cpl-db`; the old crates are deleted.

## 6. Testing

| Suite | What it proves | Where it runs |
|---|---|---|
| unit tests per crate | phase internals | `just check` (changed crates only) |
| conformance (rule-linked) | the spec's meaning, including exact diagnostics | `just check` |
| stdlib doc examples | every documented `example:` holds | `just test` |
| corpus (100 problems) | real solutions work in both engines | `just test` |
| differential fuzz | interpreter and C++ agree, with shrinking | `just test` (short), nightly (long) |
| C++ bench | p50 and max runtime vs originals, size under 32 KB | `just bench` |
| LSP snapshots | hover, completion and diagnostics | `just test` |
| bug replay | each P0 bug class is prevented or caught | `just test` |

`just check` stays under a minute. No timing tests in correctness suites.

## 7. Workflow

- I own the architecture and write the spine myself: syntax through MIR,
  the interpreter and the C++ back end. Interfaces between crates are
  settled before anyone else writes code against them.
- Parallel work starts only where an interface is fixed and a test suite
  defines done: stdlib modules (against the type-checked stubs and their
  doc examples), tools, the bench harness. Each delegated piece is small,
  time-capped and reviewed by me before it lands.
- Commits are small and go straight to `main`. Each passes `just check`;
  ratchets stop regressions.
- Spec ambiguities found while implementing are fixed in the spec first,
  with a decision entry and a conformance case, then in code.

## 8. Risks

- **Scope:** a full generic compiler plus a 1,982-function stdlib is
  large. Mitigation: the old pipeline keeps working until the new one
  passes the corpus, so there is always a usable compiler; stdlib modules
  are ranked by corpus usage.
- **Trait solver complexity:** kept simple on purpose (no overlapping
  impls, no specialization in user-visible traits; law dispatch is
  compile-time `if`).
- **Compile time of monomorphization:** measured on the largest ports;
  budget enforced by a test.
