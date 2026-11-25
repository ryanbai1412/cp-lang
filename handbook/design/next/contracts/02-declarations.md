# Contract 2: syntax → declarations

Answers "what does this name mean" without looking inside any body.

## What crosses

```rust
struct ModuleDecls {
    module:  ModuleId,
    imports: List<Import>,
    decls:   Map<DeclId, Decl>,
    scopes:  ScopeTree,                 // name → DeclId | LocalBinding, at each point
    impls:   List<Impl>,
}
struct Decl {
    id: DeclId, kind: Fn | Struct | Enum | Trait | Domain | Const | Alias | Context | Attribute,
    visibility, doc, attributes: List<AttrUse>,
    generics: List<Param>,              // each with kind (Type | Domain | Const | Pack) and bounds
    signature: Signature,
}
struct FnSignature {
    params:  List<ParamSig>,            // name, label, type, mode (value | mut | lend), default
    result:  Declared(Type) | Derived(UnitId),   // Derived: ask contract 3 for the body's type
    lends:   List<LendClause>,          // which parameters the result borrows from
    uses:    List<ContextId>,
    preconditions: List<ExprSyntax>,    // leading asserts over parameters; lowered in the caller
    captures: Derived(UnitId),          // for a fn declared inside a program file
    value_params: List<ValueParam>,     // run-time values its types mention
}
struct Impl { trait_: DeclId, for_: TypePattern, where_: List<Bound>, assoc: Map<Name, Type>, methods }
struct Interface { decls, impls, hash: InterfaceHash }     // what another library can see
```

Two queries live here and nowhere else:

```rust
fn lookup_member(recv: Type, name: Name, from: ModuleId) -> Result<Candidate, Ambiguity>
fn solve(type_: Type, bound: Bound, env: Bounds) -> Result<ImplChoice, NoImpl | Overlap>
```

## Producer guarantees

- The module graph is acyclic, or the cycle is one error.
- **Coherence:** at most one impl applies to a trait and a type. An
  overlap is an error where the second impl is written.
- `lookup_member` has one fixed order: own members, then traits, then
  free functions by specificity; the calling module wins a tie; a
  remaining tie is an error. A candidate from another library that
  beats a fitting one from the calling library is an error with a fix.
- Core types (`Int`, `List`, `Str`) are ordinary `Decl`s. There is no
  second table.
- Every attribute use is classified: marker, decorator (a library
  function), or directive (closed list in the spec).
- A `Derived` result type or capture set has a value. A caller depends
  on that value, not on the body.

## Consumer may assume

- A `DeclId` survives edits to bodies and to other declarations.
- A signature's types are fully resolved or are the error type with a
  diagnostic already recorded.

## Must never

- No consumer may test a declaration's name to decide behaviour ("is
  this `List`"). It asks for a marker or an impl.
- No consumer may resolve a member by its own search.

## Checked by

- Compiler-held: later layers get `DeclId`s, not names; the name table
  is private to this crate.
- Test: copy a std module under a new name; programs using the copy
  behave the same and emit the same C++ apart from names.
- Scan: no string comparison against a std name outside `core`.

## Open

- **Declarations inside a body.** A struct, type or domain may be
  declared inside a function, or below `@!multitest`, and use that
  body's run-time values (R01.9.1). Such a declaration is not in
  `ModuleDecls`. It is exported by the unit that contains it
  (contract 3), its `DeclId` includes that unit, and its signature
  names the body's value parameters. Not yet worked through.

- `Derived` signatures make this layer depend on contract 3 for one
  unit. The dependency is by demand with a cycle check; recursion
  without a declared result type is the cycle to specify.
- Signatures that need a constant computed by library code depend on
  contract 5 (raw IR) and the reference interpreter. Same demand rule.
- Whether preconditions are a declared clause or "leading asserts".
- **Audit 4, finding 2. In a program file this layer is not below
  the typed tree.** `domain V = 0..n` runs where it is written and
  needs the type of `n`; hoisted functions with no result type have
  signatures that depend on `V`. A program that does all of this runs
  today. So inside a program file, signatures and statement units are
  computed by demand over each other, with a cycle check. The layer
  order holds for library modules only.
- **R06.1.7:** every capture of a hoisted function must be initialized
  before the first call. That is a "definitely initialized" check in
  contract 4 that reads the callee's capture set. Nothing enforces it
  today (verified bug).
- **Audit 4, finding 6.** The spec says the decorator set is closed
  (about 25 names, unknown is E0107); the design said libraries may
  declare their own. At least twelve entries change checking or
  lowering and four are std-only. Each needs a class or a language
  feature. A decision for Ryan: may a library declare a decorator.
- A `DeclId` path needs the enclosing block: two structs named `P` in
  the two arms of one `if` compile today.

## Round 5 (2026-11-25, not yet reviewed)

- **Summaries are part of this contract.** Each declaration carries a
  derived `Summary` (effects, lends, called parameters, mode
  variables, witnesses), computed by demand like result types. See
  `representation.md`. Callers are checked against summaries, never
  against bodies.
- **Types written in a body can declare.** `let a: [0..n]Int` makes an
  index type. Turning written types into types is pure for signatures
  and not for bodies (slice finding 17).
