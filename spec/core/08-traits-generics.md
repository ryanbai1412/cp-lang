# 8. Traits, generics, laws and row types

Status: draft for 0.3.0. Sources: archived `design/spec/14`; decisions
59, 116, 127, 142, 146 (row types, trial), round 5 (`lib/ops-ranges.md`)
for laws; `12-powers.md` for what user libraries can define.

## The model in one paragraph

A trait names a capability (`Ord`, `Next<V>`, `Monoid<T>`), an `impl`
gives a type that capability, and generic code takes type parameters
bounded by traits. A law (`law left_id(x: T) = ...`) states the algebra
a trait promises, and the library relies on it: commutative tags need
no push-down in a segment tree (`lib/ops-ranges.md`). User libraries
declare traits, laws and impls exactly as the stdlib does
(`12-powers.md`).

## 8.1 Traits and impls

<!-- signatures: std/graph.cp -->
```cp
trait Next<V>:
    type Arc: Target<V>
    fn out(self, u: V) -> yields Self.Arc
```

A user library declares its own the same way:

<!-- given: struct Frac: p: Int, q: Int -->
```cp
trait Rank(Eq):
    fn rank(self, other: Self) -> Ordering

trait Unit<T>(Op<T>):
    fn unit(self) -> T
    law left_unit(x: T) = self.op(self.unit(), x) == x

impl Ord for Frac:
    fn cmp(self, o: Frac) -> Ordering = (self.p * o.q).cmp(o.p * self.q)
```

- A trait declares methods (with optional default bodies), associated
  types, field requirements and laws. Supertraits go in parentheses. (R08.1.1)
- `impl Trait for T` implements it; `impl A + B for T` is shorthand for
  two impls. One impl per (trait, type), declared with the trait or the
  type. Blanket impls are allowed (`impl<T: Add> Add for ?T`). (R08.1.2)
- Inherent methods (`impl T:` or methods in a struct body) live in the
  type's module only. (R08.1.3)
- Method lookup: inherent first, then traits in scope that the type
  implements; two candidates at one step is an ambiguity error naming
  both, resolved by `Trait.m(x, ...)`. No auto-ref or auto-deref. Generic
  code calls the method of the receiver's own `impl`, so a trait default
  a type overrides is overridden everywhere. A free extension function's
  candidates are the fns of the name whose `self` the receiver fits and
  whose parameters take the call's arguments; a built-in member whose
  parameters can't take them is no candidate, so `a.contains(i, x)`
  calls a `fn contains(self: [0..2]Int, i: Int, x: Int)`. Of the
  candidates, one is chosen when it is more specific than every other:
  `A` is more specific than `B` when `B`'s `self` binds to `A`'s and not
  the other way, so `self: Range<F64>` beats `self: Range<D>` and
  `[V]List<V>` beats `[D]T`. A bare `self: S` (with `S: Seq`) has no
  shape and is never more or less specific, so it and `self: Range<Int>`
  are the same ambiguity error, as are two equally specific candidates
  in different modules (E0701) or one module (E0702). The relation is the
  same within a module and across modules. To give a type its own cost,
  override the trait method instead (decision 486). (R08.1.4)
- Operators are trait methods (`Add`, `Sub`, `Mul`, `Div`, `Rem` with
  `Rhs` and `Out`, `Neg`, `BitAnd`, ..., `Index`, `Ord`, `Eq`, `Hash`,
  `Show`, `Read`, `Unwrap`). Compound assignment comes from the
  operator. (R08.1.5)
- **Derived impls are structural**: tuples, records, structs and enums
  get `Eq`, `Ord` (lexicographic in declaration order, variants in
  declaration order), `Hash`, `Show` and `Read` when every field has
  them, unless the type writes its own impl. Tuples and records (not
  structs, whose fields may be private) also get `+`, `-` and `Zero`
  fieldwise when every field has them, so a rerooting DP over
  `(cnt: Int, sum: Int)` passes `+` as its operation. (R08.1.6)
- **Field traits**: a trait may require fields (`trait Target<V>: to:
  V`); a bodiless `@get fn` member is the same requirement, met by a
  field of that name. A record or tuple satisfies a trait structurally
  when a field meets each member the trait leaves without a body; a named
  struct satisfies it only through an `impl`, bodiless when its fields
  meet those members. One relation decides a bound, a parameter typed by
  the trait and `if T: Trait` (decision 361). (R08.1.7)
- Library-authored diagnostics: a trait may attach a message to "not
  implemented" and to each law, so `SegTree(f, a)` with a non-associative
  `f` says what to write. (R08.1.8)
- **An impl chooses each associated type** its trait declares, with
  `type Name = T` in its body, and impls of one trait for different kinds
  of type may choose differently: `impl<T> Op<T> for Monoid<T>` chooses
  `type Fold = T` and the impl for a bare `(T, T) -> T` chooses `?T`, so one
  generic `SegTree(op, a)` folds `T` over a monoid and `?T` over a closure
  (decision 456). A fn type is a type an impl may be for, and an impl may
  cover every domain (`impl<domain D> Search for D`) beside one for a
  concrete type (`impl Search for F64`). A generic fn names the choice as
  `M.Name` (R08.2.7), and its body is checked against the bound alone
  (R08.2.2): it can't assume `M.Name` is one type, so `fn f<B: Box>(b: B) ->
  Int = b.get()` is E0302. A trait may give an associated type a default
  (`type Item = Int`): an impl that leaves it out gets the default, and
  `type Item = T` in the impl overrides it, so adding a defaulted
  associated type breaks no existing impl (decision 469). An impl that
  leaves out one with no default is E0606 naming it. Two impls that apply to one type are
  E0202 (R08.1.2), so a type has one choice. (R08.1.9)

`cmp(a, b)` is the three-way comparison of two `Ord` values, an
`Ordering` (`Less`, `Equal`, `Greater`); `desc(x)` flips an order.

## 8.2 Generics

<!-- given: use std.graph.{WeightedNext, Paths} -->
```cp
fn best<S: Indexed<K, T>, K, T, B: Ord>(s: S, by: T -> B = x => x) -> ?K = todo
fn shortest<V, W>(g: WeightedNext<V, W>, from: Seq<(V, W)>) -> Paths<V, W, g.Arc> = todo
fn f<domain I, domain J>(a: [I, J]Int) = todo
struct Pair<A, B = Int>: a: A, b: B
```

- Bounds go inline (`T: Ord + Hash`), in a `where` clause, or directly as
  a parameter type: `g: WeightedNext<V, W>` means "some type
  implementing it" (decision 146). (R08.2.1)
- Generic bodies are checked once against their bounds, at definition;
  errors point at the generic or the call, never inside an
  instantiation. (R08.2.2)
- Everything is monomorphized; there are no vtables and no `dyn`. (R08.2.3)
- A generic fn may not reach itself, directly or mutually, at a type that
  grows on every call: `wrap([x], n - 1)` inside `wrap<T>` would need
  `wrap<List<T>>`, `wrap<List<List<T>>>` and so on (E0319). (R08.2.10)
- `domain` is a kind on type parameters; `const N: Int` parameters take
  compile-time values (`Mod<998244353>`, `SegTree<0..=4e9>`); range
  types are types and may be arguments. (R08.2.4)
- Type-parameter defaults fill omitted trailing arguments. Explicit arguments on a static call, `SegTree<V, Int>.of(max, f)` or `Box<Int>.empty()`, fix those parameters before inference, with the defaults filling the rest, so they decide the result type as they do on a constructor. (R08.2.5)
- **Trait aliases** name a bundle of bounds:
  `trait WeightedNext<V, W> = Next<V> where Arc: Target<V> + Weighted<W>`. (R08.2.6)
- **Associated types** are reached through the implementing type or a
  value: `S.Item`, `g.Arc`. (R08.2.7)
- Return-type dispatch: a call may be resolved by its expected type
  (`read<T>()`). (R08.2.8)
- **Conditional code** on a bound: `if T: Commutative: ... else: ...`
  chooses at compile time; it is how laws select a specialization
  (`lib/ops-ranges.md`). `if const cond:` branches on a compile-time
  value. (R08.2.9)

## 8.3 Laws

<!-- signatures: std/algebra.cp -->
```cp
trait Op<T>:
    fn op(self, a: T, b: T) -> T
trait Commutative<T>: Op<T>:
    law comm(a: T, b: T) = self.op(a, b) == self.op(b, a)
```

- A `law` is a Boolean property of a trait's operations. Implementing a
  trait promises its laws (trust tier "promised", chapter 9). (R08.3.1)
- Laws are never assumed by the type checker. They select documented,
  answer-preserving specializations (a segment tree without push-down
  when tags commute), and `cpl explain` names each selection. (R08.3.2, permits)
- `cpl stress` samples laws on the values a program actually builds;
  `laws_hold(op, samples)` checks them directly; `--laws=off` skips. (R08.3.3)
- A false law gives wrong answers, never undefined behaviour. (R08.3.4, permits)

## 8.4 Record row types (trial)

<!-- given:
  domain V = 0..3
  let (a, u) = ((to: V(0), w: 5), V(1)) -->
```cp
fn arcs<V, R: (u: V, v: V, ..)>(edges: List<R>) -> List<(to: V, ..R without (u, v))> = todo
let a2 = (..a, to: u)  // record update
```

- `(f: T, ..)` is a bound: any record with at least those fields, in any
  order. `R` names the whole record; `..R without (u, v)` is its other
  fields as a record. (R08.4.1)
- There is no subtyping: a record with more fields is not usable where
  fewer are expected unless the parameter is written with a row bound.
  Dropping fields is explicit (`keep:` on graphs, `without` in types). (R08.4.2)
- Row-polymorphic functions are monomorphized per record type. (R08.4.3)
- Nine safety rules (decision 146): no hidden heap copies, no field
  reordering visible to the program, row variables only on records,
  duplicate fields an error, `without` of a missing field an error,
  rows not inferred across statements, no rows on named structs, field
  access through a row is \(O(1)\), and rows never change layout of an
  existing value. (R08.4.4)

## 8.5 What is deliberately missing

- No inheritance, subclassing or default-method overriding chains. (R08.5.1)
- No overloading by type, no implicit conversions, no orphan impls. (R08.5.2)
- No higher-kinded types; capability traits on sequences
  (`07-sequences.md`) and associated types cover the stdlib's needs. (R08.5.3)
- No user-defined patterns (decision 132) and no macros. (R08.5.4)
