# Contract 6: raw → canonical → optimized

Instance IR has three stages. They use the same node kinds; what
differs is which rewrites have run.

## What crosses

```rust
fn canonicalize(raw: RawProgram) -> CanonicalProgram      // required lowering
fn optimize(c: CanonicalProgram, profile, mode) -> OptimizedProgram   // optional rewrites
```

| Stage | What has happened | Who runs it |
|---|---|---|
| raw | nothing | reference interpreter |
| canonical | rewrites the program needs in order to run at its stated cost: regions inlined into callees or fused, function-value calls as switches, checks placed, widths chosen, hidden values passed | engine in debug and trace runs |
| optimized | rewrites that only make it faster or smaller: inlining, check removal by proof, merging identical instances, sharing by layout, template grouping for the emitter | engine in normal runs, machine code, C++ emitter, `explain` |

```rust
struct PassInfo { name, stage: Required | Optional, reads: Set<FactKind>, preserves: Set<Invariant> }
struct Removed { check: NodeId, by: List<FactId> }        // every removed check is accounted for
struct Shared  { kept: InstanceId, merged: List<InstanceId>, why: IdenticalIr | SameLayout }
```

## Producer guarantees

- **Same meaning.** For every input, all three stages give the same
  output bytes and the same semantic events (contract 8), and a
  checked build stops at the same check.
- **No rewrite changes which error fires first,** or moves an
  observable effect across a check that can fail.
- A check is removed only with `Proved` facts in a checked build. The
  removal records the facts.
- Every node keeps its source identity and gains to its origin chain
  when inlined, so an optimized run still reports in source terms.
- Passes are a registered list. Each declares its stage and what it
  preserves. A pass that edits output text does not exist.
- The cost table of spec chapter 13 describes canonical IR. A cost the
  user was promised is delivered by a required rewrite, never an
  optional one.

## Consumer may assume

- The engine may run canonical or optimized IR without knowing which.
- The emitter is a printer: one C++ shape per node kind.
- Which types share a representation is `Program::shares`, decided once
  in canonicalization: types told apart only by which domain a position
  counts. Every backend gives types that share one representation and
  decides no sharing of its own.

## Must never

- No backend may make a choice that changes behaviour or cost. If the
  engine and the emitter would differ, the choice is a pass.
- No optional pass may be needed for a program to compile or fit the
  size limit by more than its declared margin.

## Checked by

- Verifier after canonicalization and after every optional pass:
  well-formed, proof-accounted, identities present.
- Differential: raw vs canonical vs optimized on the corpus and on
  generated programs, comparing output, semantic events and the
  failing check.
- Each optional pass can be switched off; the suite runs with each off.

## Open

- Template grouping: the emitter prints one C++ template for instances
  that differ only in type arguments. That is a second form of the
  same program, checked only by the C++ compiler. Whether the grouping
  is recorded in optimized IR or is the emitter's own.
- Which stage the size budget is measured on, since merging is
  optional but the 64 KB limit is not.
- **Audit 4 (interactions) answers the template question:** a
  **factoring** pass at the end of the optimized stage takes the
  instances of one generic function, finds their common body, and
  outputs one function with type and constant holes. The verifier
  checks that filling the holes gives back each instance. The emitter
  prints holes as template parameters and stays a printer.
- **Prototype audit: a required rewrite is missing.** A call through
  a set of 2 to 4 closures costs 1.19x to 1.44x of a template when the
  branch is at each call, and 1.03x to 1.09x when the branch is taken
  once on entry to the struct's method, with a copy of the method per
  member. So "specialise a method per member of its struct's sets" is
  needed to meet the 1.1x bar, which makes it a required rewrite with
  an instance budget, not an optional one. Not built or measured for
  the lazy segment tree, which has five such fields.

## Reuse between edits (2026-11-24, not yet reviewed)

Canonical IR, optimized IR, bytecode and the printed C++ of an
instance are each a function of its `InstanceKey` (contract 5) and the
keys of what was inlined into it. A stage records those keys beside
its output and is skipped when they match. The C++ file is the
fragments in a fixed order, so an unchanged fn prints the same bytes.
Merging copies for size reads every instance, so it runs last and
only for `emit_cpp`, never on the run path.
