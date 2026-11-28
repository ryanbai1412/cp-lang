# Contract 4: typed tree → generic IR

Lowering turns each typed unit into a much smaller language. Three
analyses then run on it. Everything here is still generic: one result
per unit, not per instantiation.

## What crosses

```rust
fn lower_unit(typed: TypedUnit) -> IrUnit                 // no inference, no solver
fn analyse_unit(ir: IrUnit, inputs: AnalysisInputs) -> AnalysedUnit

struct IrUnit { body: Block, locals: List<IrLocal>, hidden_params: List<ValueParam>, closures: List<ClosureDef> }

enum Node {
    // control
    Block, If, Loop, Switch /* on an enum tag or a function-value set */, Continue, Return,
    Labelled { label, result: Type }, Break { label, value },
    // values
    Const, Local, Field, Element, MakeTuple, MakeStruct, MakeEnum, MakeClosure,
    // uses, always explicit
    Copy(Place), Move(Place), Lend(Place, Read | Write), Drop(Place, Always | IfInitialized),
    // calls
    Call { target: DeclId, type_args, args, region: Option<ClosureId> },
    CallTrait { param: TypeParam, method, args },          // resolved per instance
    CallValue { callee: Place, args },                     // through a function value
    Intrinsic { id: IntrinsicId, args },
    // checks and the rest
    Check { name: CheckName, operands, rule: SpecRule, status: CheckStatus },
    Trap { reason }, Event { point },
}
struct NodeMeta { source: (UnitId, NodeIx), origin: List<Origin> }   // desugaring and inlining chain
```

A **region** is how a lend and a generator are the same thing: the
caller passes the code that runs while the place is lent, or the loop
body, as a closure. It returns `Continue`, `Stop`, or `Leave(label,
value)`, where the label is a loop to break out of or the function to
return from. A loop's `else` runs on `Stop` from the callee, not on
`Leave`.

The exit rule belongs to the callee's point, not to the node. At a
`yield` point any result other than `Continue` unwinds the generator:
its remaining code does not run. At a `lend` point the callee's
remaining code runs first (a segment tree repairs its ancestors), then
the result is passed on. (Audit 4, finding 1.)

```rust
struct AnalysedUnit {
    ir:       IrUnit,                 // completed: moves, drops and lend ends placed, once
    facts:    FactTable,              // per point: ranges, relations, presence, variant; each with a tier
    summary:  EffectSummary,          // in parts; callers depend on the parts they read
    checks:   Map<NodeId, CheckStatus>,   // Needed | Proved(by which facts) | SwitchedOff(context)
    diagnostics: List<Diagnostic>,    // ownership errors, fact lints
}
struct EffectSummary { reads, writes, resizes: Set<OuterVar>, io: bool, may_fail: Set<CheckName>,
                       calls_params: Set<ParamIx>, placeholders: List<TraitMethodEffect> }
enum Tier { Proved, Checked, Promised, Trusted }
```

### The program as a whole (slice)

Beside its fns, lowering hands link two whole-program tables and the diagnostics it holds back, both in
`cpl-next-ir`, so link reads no typed tree (`link_reads_check` is 0, and
no crate past lowering depends on `cpl-next-check`):

```rust
struct SetAdds { params: Map<(FnId, Slot), SetTy>, calls: List<(FnRef, Slot)>, live: List<bool> }  // what `SetTy::AddedAt` resolves to
struct Whole {
    domains: List<Dom>,        // by DomId, what a `Ty::Dom` names: its size and start variables, its fn
    globals: List<Global>,     // by GlobalId, what a `Place::Global` names: name, top-level or in a block
    program_fns: Set<FnId>,    // the user's fns: where event points report entering and leaving
}
enum DomValue { Len(DomId), Lo(DomId) }   // a run-time value a domain carries, threaded by link
```

Each call passing a set gives it a `SetTy::AddedAt` member when its
body is checked, numbered by the call's place in that body, so a body
typed alone is final. `params` is each fn's summary: what its body adds
to its set parameters, a call's `AddedAt` included. `live` says which
calls' callees can add at all: a fixpoint over the summaries alone, no
body checked again. Link fills each live `AddedAt` from the callee's
summary per instance when it closes sets (2026-11-27).

```rust
struct Deferred(Map<FnId, List<Msg>>)  // in msg.rs: a library fn reached only through a trait, said if an instance runs it
```

`Deferred` is diagnostics, not a program fact, so it is not part of
`Whole`. Each deferred diagnostic names the layer that found it (checker
or lowering); link says those of each fn it makes an instance of, and the
driver merges nothing by hand.

## Producer guarantees

- **Expressions are pure.** Anything that can fail or has an effect
  (a call, checked arithmetic, a storage read, input) is a statement
  of its own that stores into a temporary. Statement order is then the
  only order, on every backend. (Slice: C++ leaves argument order
  open, so two nested checked operations could fail in either order.)
- **One traversal.** The IR ships one way to visit and rebuild a node's
  children. A new node kind is one edit, not one per pass.

- **The desugaring table is closed.** Each surface form has one row:
  the IR it becomes and the spec rule.
- Every copy, move, lend and drop is a node. Nothing is implicit.
- A callee's declared preconditions are `Check` nodes in the calling
  unit.
- A unit with a type error lowers each part it could not type to a
  `Check` that always fails (`CheckKind::Untyped`), the trap, and the
  rest of the program still runs. The trap is only the partial run: a
  build that reaches the unit reports its diagnostic and is not clean.
- Drops are placed once, by the ownership pass.
- **A range with a value-parameter end is `Ty::RangeEnds`.** `0..P` of
  `Mod<P>` (R03.5.1) has ends `End::Lit(i64)` or `End::Of(ty)`, where
  `ty` names one value: a static argument is its singleton
  `Ty::Range(k, k)` (`Mod<7>`), a run-time modulus `Mod(p)` a
  `Ty::RangeOf`. It is built and substituted only through
  `Ty::range_ends`, which gives `Ty::Range` once both ends are known
  values. Verify rejects a `RangeEnds` whose two ends are both known
  values, and any end still holding a `Param` past the generic stage. A
  `RangeOf`-ended one is legal and links to `CTy::Int`, exactly like
  `RangeOf`.
- Every fact names its tier; every removed check names the facts that
  removed it.

## Consumer may assume

- No surface construct survives: no comprehension, `match`, operator,
  method call or `for`.
- Facts are valid for every instance. Instance-specific facts are
  added in contract 5, never changed.

## Must never

- No consumer may compute a range, an effect or a lifetime itself.
- No consumer that could cause undefined behaviour may rely on a
  `Promised` or `Trusted` fact.

## Checked by

- Verifier: every IR unit is well-formed (each place initialized
  before use, every `Break` has its label, every lend ends).
- Compiler-held: lowering's crate does not depend on the checker's
  inference.
- Test builds: the reference interpreter tracks every lend and move
  and stops on a violation the ownership pass accepted.
- Scan: one module computes value facts.

## Open

- The region form for two overlapping lends, and what a trap inside a
  region leaves behind. The spec has no `lend` statement yet.
- Whether facts that depend on a value parameter (`i < n`) are stated
  against the hidden local or against the type.
- Fact lints that need another unit's facts are whole-program lints.
- **Audit 4 (interactions).** Ownership is a stage boundary, not an
  analysis: lowered generic IR goes in, completed generic IR comes
  out. For a call through a type parameter or a function value it
  cannot decide everything, so it also outputs **obligations** ("the
  callee bound here must not write `xs` between these points"), which
  the link step (contract 5) discharges. Today this hole is real:
  pushing to a list being walked, through `var f`, is accepted and
  prints garbage in release.
- A lend may need `lend_begin` / `lend_end` nodes in place of the
  region form (ownership places the end; overlap and early exit need
  no fusing), with the region form kept for generators only.
- Event points are not in generic IR. They are placed on raw instance
  IR and are not uses for ownership.
- Value facts are computed twice by one engine: here, and again per
  instance after inlining. A removed check names which run proved it.
- **Slice.** The region form was built and runs in both backends for
  generators and for a lender in the target of an assignment. A lend
  that spans statements needs the ownership pass to find its extent,
  and is not built.
- **Slice.** A lambda captures by value every binding that is not a
  file-level one, including those inside a top-level loop.

## Round 5 (2026-11-25, not yet reviewed)

- **Ownership runs on both sides of link** (2026-11-26 ruling, superseding "ownership
  finishes here" and decision 13). Lowering reads nothing of it: lends
  come from signatures (`plain::lends`) and passed places stay `Pass`.
  Own reads link's closed sets and each instance's resolved impls and
  does no set fixpoint of its own. It runs on both sides of link
  (2026-11-27): `own::lent` marks on generic IR which values are lent
  rather than moved, because whether an argument lends is chosen at its
  call (one in a place lends, a fresh one is moved), not by the
  signature alone, and link keys an instance by those marks. Own also
  gives link the rule for a slot a value is lent in (a plain value
  lent is copied). Everything else is decided after link. What it
  finds wrong in a fn reached only through a trait is said where an
  instance of that fn runs. Its diagnostics come from a walk of link's
  instances (`own::check`): link also makes an instance of
  every uncalled closed fn, its result sets being what its body puts
  there, so each one is checked once. A `var` made from an element
  read is its own copy (R05.3.1). The same walk decides which `Pass`
  places are copies, keyed by instance and node, and `place_copies`
  settles them; own reads nothing of the typed tree. It also marks the
  params each instance reads where the caller has them (`Func::views`,
  printed as `const&`), and says a closure that holds a `var` of its
  maker escapes (`Func::maker` names the maker).
- **`Witness(type, field)`** replaces "the size of this domain".
- **`lend_begin` / `lend_end`** for a lend that lasts several
  statements. Regions stay for generators and lenders.
- **`AddrOf(var)` and `Deref`** for a closure's reference to a `var`.
  Built in the slice.
- **A passed place is `Pass(place, viewable, site)`** in generic IR:
  a heap place given to a call by value. Lowering does not decide its
  mode; ownership's copies, by site, settle it on the linked program
  before canonicalization (`own::place_copies`): a copy taken in its
  turn where the call also writes the place (R4), else a view from the
  call's start (R11) or a read. No later IR has a `Pass`.
- **Value facts exist twice by design:** per generic unit (shown by
  hover and diagnostics, labelled generic) and per instance (shown by
  `explain`, used for the C++). A removed check names which run
  removed it.

### From the slice review (round 5, third report)

- **An operand is one of three things, not two:** a constant, a copy
  of a place, or a **borrow of a place**. Hoisting a failing operation
  into its own statement must not turn an earlier by-value argument
  into a copy. In the slice `xs[i + 1]` copied `xs`, and a loop of
  them was quadratic (2.31 s at n = 80,000; today 0.00 s). A borrow is
  legal only when ownership has shown nothing writes the place before
  its use. So the borrow-or-copy choice is made by the ownership pass
  and recorded on the operand; no later stage decides it, and the C++
  `const&` is printed from it. A copy the user did not write is an
  error or a cost note, never silent.
- **What ownership must cover** (each was accepted by the slice and
  then differed between interpreter and C++): the body of a region
  counts as running inside the callee, so body writes are checked
  against what the callee reads or writes, globals included; two
  arguments of one call are checked against each other; a callee that
  reaches a global by name holds it for the call.
- **A compound assignment evaluates its place once.** `xs[f()] += v`
  lowers to: place operands, then the value, then one read and one
  write. The slice evaluated `f()` twice.
- **`Int` is `-(2^63-1)..=2^63-1`.** The checked operations trap on
  `INT64_MIN`. The slice let it through.
- **A lender reaches exactly one `lend` on every path.** Checked on the
  typed tree, not left to the engines.
