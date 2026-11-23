# Compiler design

| | |
|---|---|
| Owner | Compiler |
| Status | Active |

How a `.cp` file becomes a checked run and a C++ file, as seven parts:
syntax, names, types, IR, facts, VM and codegen. For each part: its job, the
data it owns, the invariants it keeps, and what it must never do.

```
text --cpl-syntax--> ParsedFile   (lossless tree per item, spans, Missing/Error nodes)
     --cpl-check---> Database     (per-item Analysis: names, Ty, domains, lends; queries)
     --cpl-ir------> Program      (typed, monomorphic IR, one per run)
                       |--cpl-interp--> checked run on the VM, traces, replay
                       '--cpl-codegen-> one C++17 file
```

Example: in `let a = [10, 20, 30]; print(a.rev.take(2))`, syntax gives a
call tree with spans. Names resolve `take` to the inherent method on `Rev`.
Types give `a.rev` the type `Rev<List<Int>>`, which is not `Indexed`, so it
can't be passed to an arbitrary `S: Indexed` fn. The IR lowers the call once,
and the VM and the C++ both run that one `Program`, so both print `30 20`.

## Syntax (`crates/cpl-syntax`)

- **Job:** turn text into a lossless tree, one item at a time.
- **Data:** tokens with spans and indentation (`lexer.rs`, `token.rs`); items,
  the unit of incrementality (`items.rs`); the tree (`ast.rs`, `visit.rs`);
  syntactic facts the checker reuses (`narrow.rs`: where an optional is known
  to hold a value; `order.rs`: where a domain step stays inside).
- **Invariants:** `parse_file` never fails; a broken item still gives a full
  tree, with `Missing`/`Error` nodes; every node has a span inside the file;
  an item's spans don't move when another item changes.
- **Never:** know about types; report more than one error per root cause.

## Names (`crates/cpl-check`: `modules.rs`, `prelude.rs`, `def.rs`)

- **Job:** say what each name means: a local, a fn of this file, an imported
  module member, or a prelude name.
- **Data:** `Def` and `DeclKind` per top-level name; the module `Loader`
  (`use std.x`, `use lib.x`); std embedded at build time (`build.rs`), so
  the CLI, LSP and wasm resolve it the same way; `ItemCheck.resolved`, the
  module each name use resolves to.
- **Invariants:** a local name shadows a prelude name (R11.2.3), with no
  exceptions by arity or by name; the same text resolves the same way in
  every tool.
- **Never:** match a std name as a string to decide meaning. Each such match
  is debt counted by `prelude_privilege` and `ir_builtin_names`.

## Types (`crates/cpl-check`: `check.rs`, `check/`, `ty.rs`, `db.rs`)

- **Job:** give every expression a type, every index a domain, and every
  borrow a lend; reject what the spec rejects, with the rule's code.
- **Data:** `Ty`; per-item `Analysis`; the incremental `Database` and its
  queries (diagnostics, hover, definition, references, `typed_index`).
- **Invariants:** a trait bound is met only by a type that implements the
  trait. A view type such as `Rev<S>` is never unwrapped to `S` to satisfy a
  bound. Generic std is checked at its definition, so an error in a user
  program points at user code, not inside `std/`. Checking is per item: an
  edit re-checks that item and its dependents only.
- **Never:** panic on any input; accept a program by guessing a type the
  spec doesn't give; report one root cause twice.

## IR (`crates/cpl-ir`: `lib.rs`, `lower/`)

- **Job:** lower a program with no errors to one typed, monomorphic
  `Program` that both back ends run.
- **Data:** `Program`, `Function`, `Stmt`, `Expr`/`ExprKind`, `Ty`, `Dim`,
  `Place`, `Iter`, `Builtin`. Lowering is split by construct under
  `lower/call/` (user fns, prelude fns, folds, fields, sequences,
  collections) and `lower/{expr,stmt,mono,range}.rs`.
- **Invariants:** lowering runs only on a program with no errors; every
  expression has a concrete type; a generic fn instantiation is keyed by
  everything that changes its meaning (for a reversed view, its direction).
- **Never:** know C++ or the VM; re-derive a type the checker gave; pick a
  different algorithm than the one the program names.

## Facts (`crates/cpl-ir`: `effects.rs`, `pushed.rs`, `show.rs`, `sites.rs`; `crates/cpl-prover`)

- **Job:** decide once what both back ends must agree on.
- **Data:** `effects::Table`: per fn and closure, the globals it writes,
  whether it does I/O, may trap, or calls an unknown fn value. `pushed`:
  which loops over a `yields` fn run without a list. `show`: how each type
  prints. `sites`: the user's own statements, for profiles and coverage.
  `cpl-prover`: integer intervals, laws and lends.
- **Invariants:** each fact has one home here; a back end asks, never
  recomputes; a fact is conservative, so "unknown" means "may".
- **Never:** depend on a back end.

## VM (`crates/cpl-interp`)

- **Job:** the checked reference run: every check the spec names fires,
  with source values in the trap.
- **Data:** `vm/`: IR compiled to a register VM (`vm/compile/`), its run
  loop (`vm/exec/`), run-time values (`value.rs`, with packed arrays for
  scalars); traces (`trace.rs`), replay (`replay.rs`), profiles
  (`profile.rs`).
- **Invariants:** deterministic for a given program and input; a trap has a
  span, the frame's values and the test case number; a packed array element
  is read and written as a copy, the same as a store into it.
- **Never:** disagree with the C++ on a valid program or on input either
  rejects; hand a construct to the old tree-walker. Each fallback is counted
  by `CPL_VM_STATS` and the `vm_fallbacks` ratchet, and the tree-walker is
  being deleted.

## Codegen (`crates/cpl-codegen`, owned by perf)

- **Job:** one deterministic C++17 file per program, as fast as matched
  hand-written C++.
- **Data:** it reads the `Program` and the facts above; its own analyses
  (devirtualization, lends, fusion) refine them.
- **Invariants:** output agrees with the VM on every program; a read-only
  heap value is read in place, not copied.
- **Never:** read syntax or re-derive types; emit undefined behaviour.

## Where the contracts are tested

- Meaning: `cpl-conform conformance` over `spec/conformance/`.
- Agreement: `cpl-conform diff` and `crates/cpl-cli/tests/differential.rs`.
- Recovery and one error per cause: `crates/cpl-cli/tests/recovery.rs`.
- Std holds to user rules: `crates/cpl-cli/tests/std_checks.rs`.
- Editor and CLI agree: `crates/cpl-cli/tests/lsp_parity.rs`.
- Effects: `crates/cpl-ir/src/effects_tests.rs`.

The full list, with what is still missing, is the quality register in
`handbook/domains/compiler.md`.
