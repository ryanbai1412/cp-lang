# Contract 5: generic IR → instances

Starting from the entry points, the collector makes one concrete copy
of each unit per distinct use. This is where the program stops being
generic.

## What crosses

```rust
fn collect(entries: List<DeclId>, units: AnalysedUnit provider) -> RawProgram

struct RawProgram {
    instances: Map<InstanceId, RawInstance>,
    types:     Map<ConcreteTypeId, Layout>,      // facts/layout.md
    fn_sets:   Map<FnSetId, List<ClosureInstance>>,
    errors:    List<InstanceError>,
}
struct InstanceId {                 // identity: stable when code inside a closure is edited
    decl:      DeclId,
    type_args: List<ConcreteType>,  // types, domains, constants
    closures:  List<ClosureNode>,   // by node identity
    contexts:  ContextValues,
    lend_modes, fn_sets: List<FnSetId>,
    // Not part of identity: the hidden run-time values it is handed (see below).
}
struct Fingerprint(Hash)            // identity + generic-IR hashes + mode + profile + requested events
struct RawInstance { id, fingerprint, body: Block /* same node kinds as contract 4 */, facts: FactTable }
```

What changes from the generic IR:

- every type is concrete; every `CallTrait` is a `Call`;
- every stored function type has its **set**: the closures that can
  reach that slot. Sets and instances are found together, to a fixed
  point. A set with one member is that closure's own type;
- a loop over a type's fields is unrolled; `if const` is decided;
- effect placeholders are filled.

## Producer guarantees

- **Raw IR is the meaning of the program.** Nothing has been chosen
  for speed. The reference interpreter runs it as it is.
- Instantiation cannot fail for a reason the generic check could have
  seen. The reasons it can fail are a **closed list**: recursion that
  makes a new instance each level, a type that would contain itself
  (including a closure capturing a function value of its own set), a
  `@memo` key with no bound, a grader type with no C++ form. Each is
  reported at the user's call with the chain that led there.
- Identity and fingerprint are separate. Caches use the fingerprint.
  Recordings, traces and `explain` use the identity.
- The same snapshot gives the same instances in the same order.

## Consumer may assume

- A closed world: every call target is known, every function value is
  one of a listed set.
- Layout is a function of the concrete type and nothing else, with the
  spec's three named exceptions.

## Must never

- The collector must not merge, share or drop instances. That is an
  optional rewrite (contract 6).
- No consumer may change a fact; it may add instance facts.

## Checked by

- Verifier on every raw instance: no type parameter, no trait call,
  no set variable.
- Test: instantiating twice gives equal fingerprints.
- Differential: reference interpreter on raw IR against corpus answers
  and spec examples.

## Open

- How precise the flow that builds function-value sets must be, and
  what it costs on std. Being measured by a prototype.
- Whether the fixed point between sets and instances always ends; a
  generic function that makes a closure per level does not, and must
  land in the closed error list.
- Instances are whole-program. Which part of an editor request, if
  any, may need them (hover showing a layout or a cost does).
- **Audit 4.** A `let`-bound lambda that calls itself (R01.8.1) is a
  direct call, not a capture of its own set, and stays legal.
- **Audit 4 (interactions): the link step.** What this contract calls
  "the collector" is really a step that solves whole-program
  questions from per-unit summaries: which instances exist, what each
  function-value set holds, and whether each ownership obligation
  holds. Its result is a **program table**. It reads summaries, never
  unit bodies. This is the one structural addition the audit asks
  for.
- Two candidate designs for sets, to be settled by the thin slice:
  (a) the set is a named variable on a declaration, units export flow
  edges, the link step solves them, and membership is in the
  fingerprint only; (b) the set is an implicit type parameter solved
  by the checker and instantiated like any type argument. A set
  inside a type argument (`List<Int -> Int>`) runs today, so the set
  is a parameter of `List` under either.
- A recursive generic function that passes a closure capturing its own
  function parameter makes a new instance per level. It is ordinary
  divide and conquer and runs today through `std::function`.
- Compile-time evaluation may call only functions whose instances are
  closed without the link step. Everything else is evaluated at
  start-up.
- **Slice: run-time values of types.** Generic IR has one pure node,
  "the size of this domain type". The link step computes, to a fixed
  point over calls and closure creations, which instances need a size
  they cannot read themselves. Each gets hidden leading parameters,
  every call passes them, and a closure captures them when it is made.
  A file-level domain is one shared variable; one declared in a fn or
  inside a top-level block is not. This passed recursion with closures
  that outlive the inner call, where a global slot gives wrong answers.
- **Slice.** Arrays store their length. So a hidden size is needed only
  by code that names the domain (count, conversion, iteration), and
  generic code holding an opaque `[V]Int` needs nothing.

## Reuse between edits (2026-11-24, not yet reviewed)

The link step reruns on every whole-program request. It reads only
per-unit summaries, so its cost follows the number of units, not their
size. Everything after it is reused by key.

```rust
struct InstanceKey { body: ContentHash,          // the generic fn's IR
                     types: List<TypeId>, sets: List<SetId>,
                     contexts: List<(ContextName, Value)>,   // only the ones this body reads
                     events: Option<RequestHash> }
```

- Two instances with equal keys have equal instance IR. Checked by
  building twice from unrelated snapshots and comparing bytes.
- A `SetId` is the sorted list of its members' keys, so adding a
  closure elsewhere in the file changes only the instances whose set
  gained a member.
- An edit to one fn body changes that fn's key and the keys of
  instances whose sets or inlined callees include it. Count gate: for
  a fixed list of edits, the number of instances rebuilt.
- Open: source identities inside an instance. If they are positions,
  an edit above a fn changes its key. They must be relative to the
  unit, with the unit's place in the file held outside the key.

## Round 5 (2026-11-25, not yet reviewed)

- **The link step chooses representations and does not reject**, with
  the two closed exceptions in `representation.md` (profile budgets;
  a function value that contains itself, pending Ryan). The
  `InstanceError` list shrinks to those.
- **Identity is `(declaration, types, modes, sets, contexts)`.** Modes
  are owned-or-lent per heap-holding slot.
- **Witnesses** go down as hidden parameters and up as hidden results;
  `@!multitest` scopes them per case.
- **Recursive generators** need one indirect call per item. Spec
  R06.2.5 says there is none. The spec gains this one exception, or
  recursive generators are refused. Open.

### From the slice review (round 5, third report)

- **A run-time value belongs to a binding, not to a declaration.** In
  `fn rec<D>` that declares `domain V` and calls `rec(n - 1, b)` with
  `b: [V]Int`, the callee's `D` is the caller's `V` and its own `V` is
  a different one. The slice keyed hidden sizes by declaration, mixed
  the two, and printed `44` for `51` on all three paths. A witness is
  keyed by (type parameter or local declaration) within one instance,
  and a type argument's witness always arrives as a hidden argument.
  Today's compiler rejects this program (E0319); with witnesses it
  runs, and until they are built the slice must reject it too.
- **Instantiation ends by an argument, not by a cap.** The slice's cap
  of 400 instances rejected 300 lambdas through two small functions.
- **Factoring instances that differ only by closure is required,** not
  optional: emitted C++ grew 1.5 KB per closure, 80 KB at 50 lambdas
  against the 64 KB limit.
- **Every link error carries the user's call and the chain to it.** The
  slice's carry no location.
