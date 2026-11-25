# Contract 3: declarations → typed tree

The checker decides every type, every name's target and every
conversion. After it, nothing is inferred again.

## What crosses

One result per **unit**: a declared function, or one top-level
statement of a program file. A lambda is checked inside the unit that
contains the call it is passed to.

```rust
fn check_unit(unit: UnitId, inputs: UnitInputs) -> TypedUnit

struct UnitInputs {
    syntax:     ItemSyntax,
    signatures: Set<DeclId>,              // the ones it names
    bindings:   Map<Name, Binding | NotFound>,  // outer names it looked up, one by one
    used_later: Set<Name>,                // which of its bindings later statements name
    contexts:   ContextValues,
}
// Built in the slice as a tree of its own, each node carrying its source span, not as a
// table over the syntax tree. Lowering then walks one structure and needs no lookup.
struct TypedUnit {
    nodes:    Map<NodeIx, TypedNode>,
    locals:   List<Local>,                // name, type, mutability, value params
    exports:  Map<Name, Binding | LocalDecl>,   // values, and the types and domains it declares or opens
    diagnostics: List<Diagnostic>,
}
struct TypedNode {
    type_:    Type,                       // never an inference variable
    target:   Option<Target>,             // DeclId + type arguments + ImplChoice, or a local
    convert:  Option<Conversion>,         // from the closed list
    expected: Option<Type>,               // what flowed in; for hover and fixes
    contexts: ContextValues,              // only where they change
}
struct Binding { type_: Type, mutable: bool, value_params: List<LocalId>, fn_set: Option<FnSetVar> }
```

`Type` is one interned representation for every layer. Its forms are a
closed list (see `facts/types.md`). Three points matter at this
boundary:

- A type may carry **value parameters**: run-time values such as the
  `n` of `[n]Int` or `1..=n`. In the typed tree each is a named or
  hidden local.
- A stored function type carries a **set**. In the slice the checker
  solves it like a type argument: a set is a variable inside one fn or
  file, a hidden parameter in a signature, or "what `f` returns". Two
  conversions exist: a closure enters a set, and a set enters a larger
  one. A `let` bound to a closure keeps the closure's own type, so the
  call is direct; a `var` gets a set. See `slice.md`, findings 2 to 4.
- An associated type of a parameter (`G.Arc`) is kept normalized and
  opaque.

## Producer guarantees

- **Total.** Every node has a type. A node that failed has the error
  type, and a diagnostic exists for it or for a node it depends on.
- **Accepted means complete.** A unit with no diagnostics has no error
  type and no missing target.
- **Generic bodies are checked once** against declared bounds. What
  cannot be checked from bounds is a closed list of instance-time
  errors (contract 5).
- Operators resolve bottom-up with one lookup each.
- Diagnostics come in order: syntax, names, types. A later layer does
  not report on a node an earlier layer rejected.

## Consumer may assume

- Lowering needs nothing else: no inference, no trait solving, no
  member lookup. Every call site names its target and type arguments.
- Hover, completion and type errors need only this and contract 2.

## Must never

- No later layer may compute a type, pick an overload or choose an
  impl for a concrete type parameter except by `solve` at
  instantiation, which coherence makes unique.

## Checked by

- Compiler-held: `Type` has no inference-variable form outside the
  checker crate; `TypedNode.type_` is that restricted type.
- Verifier: an accepted unit has no error type.
- Count gate: a run over std, the corpus and the conformance suite
  reports zero untyped expressions without a diagnostic.
- Test: lowering is built without a dependency on the checker's
  inference or solver modules.

## Open

- `used_later` makes a statement depend on statements below it. It is
  needed only by ownership (contract 4), so it may belong there and
  not here. To settle with the interaction audit.
- Lambda bodies checked with their call: completion inside a lambda
  rechecks the enclosing statement. Cost unmeasured.
- Range arithmetic during checking uses the same algebra as the facts
  engine (`facts/ranges-and-proofs.md`); how much of it the checker
  may call on the typing path is unbudgeted.
- **Slice.** A top-level `var f` of function type gains members from
  later statements, so its type is not final at its own statement. The
  file is one scope for sets.
- **Slice.** Pushing a new function into a list of functions received
  as a parameter cannot be typed with set parameters alone: a container
  cannot be widened in place. Choices: a compile error; or that set
  stops being a parameter and is shared by all callers.
- **Prototype audit and slice.** One set per variable gives a false
  "contains itself" on `m = dual(m)`, where `dual` wraps a stored
  operation. In the slice, `let m = if c: base.dual() else: base`
  works: the branches of an `if` are joined into a new set, and a
  struct whose function fields grow is rebuilt field by field (a third
  conversion). Reassigning the same variable is a check error that
  says to bind a new name. The auditor would give each assignment its
  own set instead; not built.
- **Prototype audit.** "Which later statements use this binding" is
  the wrong input. A statement's input is the lends alive when it
  starts; the statement that breaks one neither defines nor names it.
- Facts leave the typing path. A function-body edit re-types 1
  statement but re-analyses a median of 726 of 2,279 unless equal
  outgoing facts stop the ripple and facts on top-level variables
  written in several statements are widened at statement boundaries.

## Round 5 (2026-11-25, not yet reviewed)

- **Direction: function-value sets leave the type.** A function type
  is one type here; which closures a slot holds is chosen at link
  (`representation.md`). This removes the dependence of a top-level
  `var f` on later statements. The slice still has sets in the type.
- **Closures record capture modes** (`Copy` or `Ref`), from the
  binding's kind. A closure with a `Ref` capture is lent. Built in the
  slice for `var`.
- **Reflection** (`for f in T.fields`) types `x.(f)` per instance,
  against "generic bodies are checked once". It needs a bound over
  fields, such as `T: Record<each: Show>`. Open.
