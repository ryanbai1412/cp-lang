# compiler architecture

The owning lane keeps this current in the same commit as any structural change. It says: the modules and what they are for, the boundaries and invariants, the interfaces other lanes may use (and their consumers), and the known debt. The architecture & tech-debt lane reviews it against the code. Read from main `4b8f504f`, 2026-11-22; line counts from `wc -l`. The target shape is [plan/ARCHITECTURE.md](../../ARCHITECTURE.md); this file is what exists.

## Pipeline

```
text --cpl-syntax--> ParsedFile (lossless AST per item, spans, Missing/Error nodes)
     --cpl-check---> Database: per-item Analysis (names, Ty, domains, lends), TypedIndex, queries
     --cpl-ir------> Program (typed IR, monomorphized, one per run)
                      |--cpl-interp--> checked run (VM, falling back to the tree-walker), traces, replay
                      '--cpl-codegen-> one C++17 file           (perf domain)
prover: integer facts and laws; today called only by cpl-lint
```

## Modules

### `crates/cpl-syntax` (parse; no knowledge of types)
- `lexer.rs` (980), `token.rs`: tokens with spans and indentation.
- `items.rs`: splits a file into top-level items; the unit of incrementality.
- `parser.rs` + `parser/{item,stmt,expr,pattern,types}.rs` (largest 1935): recursive descent with recovery; a broken item yields a full tree with `Missing`/`Error` nodes.
- `ast.rs` (708): the tree. `visit.rs`: walking it. `literal.rs`: literal values.
- `narrow.rs` (where an optional is known to hold a value), `order.rs` (where a domain step stays inside): syntactic facts the checker reuses.
- `planned.rs`: forms the 0.3 design names but this version doesn't implement, so they get "not yet" instead of a parse error.
- Entry points: `parse_file`, `parse_item`, `split_items`, `lex` (`src/lib.rs`).

### `crates/cpl-check` (meaning; one item at a time, incremental)
- `db.rs` (2519): the incremental `Database`: files, items, modules, per-item `Analysis`, and every query (diagnostics, hover, definition, references, symbols, `typed_index`, storage/cost).
- `check.rs` + `check/{types,call,expr,stmt,lends}.rs` (largest `expr.rs` 4323): resolving types and `Def`s; calls and conversions; expressions; statements; lexical borrows (`plan/19-index-and-lends.md` §3.5-3.6).
- `def.rs` (what a top-level name means: `Def`, `DeclKind`, `DomainInfo`), `ty.rs` (`Ty`, `Param`).
- `modules.rs` (`Loader`, `FsLoader`: where `use std.x` / `use lib.x` lives); `build.rs` embeds every `std` module so CLI, LSP and wasm resolve std identically.
- `prelude.rs` (names without `use`), `builtins.rs` (the one table of members the language gives `List`, `Str`, `Map`, `Set`, arrays: `BUILTIN_MEMBERS`).
- `hover.rs`: hover text, definitions, outline. `edition.rs`: `is_v03()`, now constant `true`.

### `crates/cpl-ir` (one typed program for both back ends)
- `lib.rs` (1501): the IR: `Program`, `Function`, `Stmt`, `Expr`/`ExprKind`, `Ty`, `Dim`, `Place`, `Iter`, `Builtin`.
- `lower/mod.rs` (6123): driver from checked syntax to `Program`; `lower/mono.rs` (4570): multi-module collection, monomorphization, traits/impls, closures, `lends` projections, `laws_hold`.
- `lower/call.rs` (4205): method calls and walk items; its constructs live in `lower/call/`: `user.rs` (user and generic fns, constructors), `prelude.rs` (prelude fns, conversions, read), `fold.rs` (folds and reductions), `field.rs` (fields, members, walk heads), `seq.rs` (sort, domain members, lazy adapters), `collect.rs` (adjacency, spans, counts). `lower/expr.rs` (6107), `lower/stmt.rs` (6470): expressions; statements, blocks, patterns.
- `lower/range.rs` (946): value ranges for check removal and widths. `lower/memo.rs` (`@memo`), `lower/cache.rs`.
- `pushed.rs` (which `for x in f(..)` over a `yields` fn run without a list; both back ends ask here), `show.rs` (type-decided printing shared by both back ends), `sites.rs` (the user's own statements, for profiles and coverage), `walk.rs`.

### `crates/cpl-interp` (the checked reference run)
- `lib.rs` (5615): `run`, `run_with`, `run_observed`, `run_profiled`; `Trap`, `Limits`, `Io`, `Frame`; and the tree-walking evaluator.
- `vm/` (14 files, largest 1771): `CompiledProgram` compiles IR to a register VM; any statement or expression it doesn't lower runs on the tree-walker (`fallback_stmts`/`fallback_exprs`), counted by `CPL_VM_STATS`. `vm/ir.rs` the VM's instructions; `vm/compile/` IR to instructions (`stmt.rs`, `assign.rs`, `expr_i.rs`/`expr_v.rs`/`expr_opt_i.rs` by result kind); `vm/exec/` the run loop (`calls.rs`, `places.rs`, `containers.rs`); `vm/helpers.rs`. Compiled out on wasm (`lib.rs:16`).
- `value.rs` (1386): run-time values. `input.rs`: one buffered token reader.
- `trace.rs` (2350) + `trace/log.rs`: the trace document for `cpl run --trace`/`cpl stress --trace` (`tools/trace/trace-schema.json`). `replay.rs`: rerun one recorded call with its fn edited. `profile.rs`: per-statement counts and time.

### `crates/cpl-prover` (package `cpl-prover`; integer reasoning)
- `term.rs`, `parse.rs`, `lower.rs` (terms to polynomial constraints), `poly.rs`, `omega.rs` (Omega test, complete for linear integer conjunctions), `solve.rs` (case splits plus Omega), `context.rs` (proof context and questions).
- `interval.rs` (open-ended integer intervals), `loops.rs` (loop invariants), `laws.rs` (Monoid/Group/Action/Order laws proved or refuted), `lends.rs` (`law disjoint`), `smtlib.rs` (export), `eval.rs`, `core.rs`.

## Interfaces other domains use

The shared core. The compiler lane owns the code; every consumer lane adds contract tests next to the core for what it needs (for example: the syntax tree round-trips every corpus file, every node has a span, the IR exposes the trace hooks), so a core change cannot land that breaks a consumer. A lane that needs a new interface asks in the compiler worklog instead of working around the gap; architecture & tech-debt reviews every interface change against all its consumers.

| Interface | Concrete API | Consumers (domain: crate, from `Cargo.toml`) |
|---|---|---|
| Lossless syntax tree and spans | `cpl_syntax::{parse_file, ParsedFile, ast, visit, split_items, lex}` | lint-fmt: `cpl-fmt`, `cpl-lint`; tools: `cpl-lsp`, `cpl-doc`, `cpl-cli`; debugger: `cpl-stress`; judge: `cpl-contest`; compiler: `cpl-check`, `cpl-ir` |
| Typed tree and query API (types, definitions, references, ranges) | `cpl_check::{Database, Analysis, TypedIndex, Hover, Definition, Symbol, Ty, Def, DomainInfo, builtin_members, std_names}` | tools: `cpl-lsp`, `cpl-cli`, `cpl-wasm`; lint-fmt: `cpl-lint`; perf: `cpl-codegen`; debugger: `cpl-stress`; judge: `cpl-contest`; diagnostics via `cpl-conform` |
| IR | `cpl_ir::{lower, Program, Function, Stmt, Expr, Ty, Builtin, pushed, show, sites}` | perf: `cpl-codegen`; compiler: `cpl-interp`; tools: `cpl-cli`, `cpl-lsp`, `cpl-wasm`; debugger: `cpl-stress`; judge: `cpl-contest`, `cpl-conform` |
| Run, trace, replay | `cpl_interp::{run, run_with, run_observed, run_profiled, Trap, Limits, Io, trace, replay, profile}` | debugger: `cpl-stress`, trace viewer via `cpl-wasm`; tools: `cpl-cli`; judge: `cpl-conform` |
| Diagnostics and fix API | `Database` diagnostics carry `cpl_diag` values (code, span, help, fix edits) | diagnostics: `cpl-diag`; lint-fmt: `cpl-lint`; tools: `cpl-lsp`, `cpl fix` |
| Std/builtin boundary | `cpl-check/build.rs` (embedded `std`), `prelude.rs`, `builtins.rs`, std names matched in `check/*.rs` and `lower/call.rs` | stdlib (`std`); architecture & tech-debt |
| Integer facts | `cpl_prover` (intervals, laws, lends, solve) | lint-fmt: `cpl-lint` only |

Contracts each consumer may rely on (and where the test lives today):
- **Syntax:** `parse_file` never fails; every node has a span inside the file; spans of an item are stable when another item changes (`crates/cpl-cli/tests/recovery.rs`, `crates/cpl-syntax/src/tests.rs`). No round-trip-every-corpus-file test exists yet: the first contract test to add.
- **Query API:** answers are per item and only change when that item or its dependencies change; LSP and CLI give the same diagnostics for the same text (`crates/cpl-cli/tests/lsp_parity.rs`).
- **IR:** one `Program` feeds both back ends; anything both must agree on (printing, pushed loops, statement sites) is decided once in `cpl-ir` (`show.rs`, `pushed.rs`, `sites.rs`), and the diff suite checks the result (`cpl-conform diff`, `crates/cpl-cli/tests/differential.rs`).
- **Run:** deterministic for a given program and input; a trap carries a span and the frame's values; traces follow `tools/trace/trace-schema.json`.
- Owned crates have no `tests/` directories: contract tests live in consumers (`crates/cpl-cli/tests`). New ones go next to the core they protect.

## Invariants

- `spec/` is the meaning; a disagreement is a compiler bug or a spec-worklog request, never a test edit.
- Front end never panics on any input and stays per item: a change to one item re-checks only it and its dependents (`db.rs`).
- Lowering only runs on a program with no errors; the IR is fully typed and monomorphic.
- Both back ends read the same `Program`; neither re-derives types or reads syntax.
- `cpl-syntax` knows no types; `cpl-check` knows no IR; `cpl-ir` knows no C++ and no VM.
- std is CP Lang source embedded at build time, not Rust; the compiler's knowledge of std names is debt to remove, not a mechanism to extend.

## Known debt

| Debt | Where | Fix (milestone) |
|---|---|---|
| Two interpreter engines: tree-walker plus VM with fallback; every runnable corpus program (13/13) falls back, top tag `expr:Read` | `crates/cpl-interp/src/lib.rs`, `crates/cpl-interp/src/vm/` | VM lowers everything, tree-walker deleted, VM on wasm (M3) |
| Giant files: 13 owned files over the 2000-line cap, allowlisted in `scripts/debt-ratchet.json` | `lower/call.rs` 13357, `lower/stmt.rs` 6470, `lower/mod.rs` 6123, `lower/expr.rs` 6107, `cpl-interp/src/lib.rs` 5615, `lower/mono.rs` 4570, `check/expr.rs` 4323 | split by concern as code is touched, pure moves first, as `vm.rs` (15367) was (M3) |
| std privilege: 146 string literals of 35 std type names (`Prefix` 16, `Gen` 16, `Bitset` 12, `Subset` 11, ...) | `cpl-check/src/prelude.rs` 27, `check/call.rs` 21, `check/types.rs` 18, `cpl-ir/src/lower/call.rs` 14, `check/expr.rs` 13, rest scattered | one intrinsics table, then a spec mechanism (M3; tech-debt's builtin-floor question) |
| Edition switching left over: 120 `is_v03()`/`spec_edition()` call sites, always true | `crates/cpl-check/src/edition.rs` and callers | inline the 0.3 branch, delete the other (M3) |
| Range facts in two places, prover unused by the compiler | `crates/cpl-ir/src/lower/range.rs`, `crates/cpl-codegen/src/ranges.rs`, `crates/cpl-prover/src/interval.rs` | one facts pass in `cpl-ir` over `prover::interval` (after M3) |
| Product domains half-built: position-as-tuple vs nominal paths disagree (16 `E0302`, 18 wrong outputs cluster in `core-04`) | `crates/cpl-check/src/check/call.rs`, `crates/cpl-ir/src/lower/call.rs`; `devin/wip-product-domain` | M1, Q1 |
| 9 ignored tests in owned crates | `grep -rn '#\[ignore' crates/cpl-*` | fix or delete with reason (M3) |
