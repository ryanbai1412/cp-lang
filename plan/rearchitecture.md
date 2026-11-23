# Re-architecture: getting there from today

How today's code reaches the target in `handbook/design/architecture.md`.
Time-bound: steps, current state and the re-architectures in flight.

## B1. Where we are

Measured 2026-11-22; commands in `plan/domains/tech-debt/metrics.toml`
and `plan/domains/compiler/metrics.toml`.

- About 203k lines of Rust in 18 crates
  (`git ls-files 'crates/*.rs' | xargs cat | wc -l`; `ls -d crates/*/`): `cpl-syntax` ->
  `cpl-check` (salsa-style database) -> `cpl-ir` (typed lowering) ->
  `cpl-interp` (a tree-walker plus a register VM that falls back to it)
  and `cpl-codegen` (C++17). "Crates" in `handbook/design/architecture.md` now lists `cpl-contest`, `cpl-json` and
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
  4. diagnostics: `cpl_ir::fix` re-checks and applies in rounds over
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
| fix | `cpl_ir::fix` (checked, in rounds), over `cpl_diag::apply_fixes` (text edits) | landed ebea73d60; lint's `fix_checked_removed_syntax` left | diagnostics |
| run | `cpl_interp::run_with` | `run`, `run_with`, `run_observed`, `run_profiled` | compiler |
| trace | `cpl_interp::observe_on_this_thread` | plus `checkpoint::record_*` | debugger |
