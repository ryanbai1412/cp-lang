# Representations and summaries

**Status:** 2026-11-25. Written after review round 5 (contracts
review F1 to F5). Not yet reviewed. Only the closure capture mode is
built in the slice.

Round 5 found four mechanisms that are the same kind of fact: decided
by how a value flows, not written in the source, and visible in how
it is stored. They were spread over three contracts as `lend_modes`,
`fn_sets`, "the size of this domain" and the hash-map order choice.
This page gives them one name, **representation**, and one home.

It also states the rule that separates them from user errors.

## The rule

**Whole-program steps choose representations. They do not reject
programs.** Every error a user can cause is decided from one unit plus
the summaries of the declarations it names, so the editor can show it
from the statement being typed.

Two lists are exempt, and each is closed:

1. **Budgets against the judge profile**: peak memory, source size, a
   128-bit integer on a judge without one. These run off the typing
   path and are reported as their own kind.
2. **A function value that would contain itself** (decision 2, Ryan's
   ruling). It needs the whole program today. Review round 5 proposes
   a heap-allocated fallback that removes this entry; that is an open
   question for Ryan, not a decision.

## Summaries: what a signature says beyond its types

Contract 2 already derives result types and capture lists on demand.
The same mechanism carries everything a caller needs to check a call
without reading the callee's body:

```rust
struct Summary {                      // derived, per declaration, part of contract 2
    effects:  List<Effect>,           // top-level variables read and written, input, output
    lends:    List<(Param, Result)>,  // which parameters the result borrows from
    calls:    List<Param>,            // function-typed parameters it calls, and how (read or write)
    modes:    List<ModeVar>,          // see below: one variable per heap-holding slot in the signature
    witnesses: List<WitnessUse>,      // run-time type values it needs from, or returns to, the caller
}
```

- A summary is computed from the body once, by demand, and cached with
  the declaration. A cycle of mutually recursive functions is solved
  as one group.
- A summary changes only when the body's behaviour toward its caller
  changes. Editing a body without changing its summary re-checks no
  caller.
- A call through a function value uses the function type's own
  summary (what any member may do), which is part of the type.

**What this moves.** "Is this value changed while it is lent" was
answered in the link step (decision 13). It is now answered at the
call site from the callee's summary and the closure's capture modes.
The slice's whole-program check stays only as a test that the
per-unit answer is never weaker.

## Mode: owned or lent

Every slot that can hold heap storage has a mode:

```rust
enum Mode {
    Own,                              // this slot frees its storage
    Lend(Access, Sources),            // it does not; `Sources` are the variables it borrows from
}
enum Access { Read, Write }
```

- The mode is part of the **concrete** type. `Win<List<Int>>` built
  from a fresh list and `Win<List<Int>>` built from a parameter are
  one written type and two concrete types, with different drop code.
- A generic signature has a mode variable per heap-holding slot, so
  one function serves both, and instances are keyed by
  `(types, modes)`.
- A lent value cannot be returned, or stored anywhere that outlives
  its sources. That is a per-unit error.
- A lend that lasts several statements is `lend_begin` and `lend_end`
  in the IR. Regions (code run inside a callee) are kept for
  generators and lenders only.

**Closures.** A closure records how it holds each capture:

```rust
struct Capture { var: Local, how: Copy | Ref(Access) }
```

A `let` of a plain value, a parameter without a heap part and a
hidden run-time value are copied. A local `var` and a local heap
value are held by reference (spec R05.10.1). A closure with any `Ref`
capture is lent, with those variables as its sources.

Built in the slice: `Ref` captures for `var`, and the error for
returning such a closure. Not built: heap values held by reference,
the error for storing one through a `mut` parameter, and the borrow
check seeing writes made through a reference.

## Witness: the run-time values a type carries

A type declared in a body can depend on values known only at run
time. The earlier contract had one node, "the size of this domain".
The general form:

```rust
struct Witness {                      // one per fresh type, declared with the type
    fields: List<(Name, Type, StaticBound)>,   // count, modulus, derived constants, a library index type's lets
}
// generic IR
Witness(type, field)                  // replaces "the size of this domain"
```

- The link step passes a witness **down** as a hidden parameter, has
  closures copy it when they are made, and passes it **up** as a
  hidden result when a function returns a type it declared
  (`fn comps(xs) -> (domain P, List<P>)`).
- Opening such a result binds the witness to a local.
- The static bound of each field comes from the declaration and gives
  storage widths and overflow proofs. A returned type with no declared
  range has the bound its signature states, or the widest.
- `@!multitest` makes the witness per test case, not per file.
- This reverses spec R02.9 and R13.2 (one static slot per declaration,
  saved and restored on re-entry). The slot gives the wrong answer for
  a domain (slice t14) and for a modulus (`mod_rt.cp`: 400 is correct,
  today's compiler prints 200). The spec changes with it.

## Function-value slots

How a slot of function type is stored is a representation:

| Representation | When |
|---|---|
| The closure's own type | one closure reaches the slot |
| A tagged set of closures | several reach it, and the set is finite |

Review round 5 proposes two changes, both open:

- **Take sets out of the type.** `(A) -> R` would be one type in the
  typed tree, and the set would be chosen at link. Then typing a
  statement never depends on later statements, which the spec
  requires ("Inference never looks at later statements"). The slice
  has sets in the type, and a top-level `var f` gains members from
  statements below it.
- **A third, heap-allocated representation** as the fallback that
  always exists, reported by `explain` and flagged by a lint in a hot
  loop. This would accept four programs that run today and that the
  design rejects. It conflicts with the ruling that stored function
  values never cost a heap allocation or an indirect call.

The first is adopted as the direction and not built. The second waits
for Ryan.

## Map order

Decision 6 has `HashMap` iterate in insertion order. Today's compiler
picks a plain hash table when nothing reads the order. With the map
in library source this is a choice between two library types, made
where the program is whole, and reported. It is one more row in the
same table, not a separate mechanism.

## What `explain` and hover show

Every representation choice is reported in the user's terms: "`f`
holds one of 3 functions", "`w` borrows `a`", "`Z` carries its
modulus". Hover in generic code shows the generic answer and says it
is generic; `explain` on an instance shows the instance's.
