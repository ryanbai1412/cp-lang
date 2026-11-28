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
  type. Blanket impls are allowed (`impl<T: Add> Add for ?T`); an impl is
  blanket when its head is a bare type parameter. An impl applies to a type
  when its head fits it and, for a blanket impl, its bounds hold. Among the
  impls that apply, the top rank (R08.1.9) is chosen; two of one rank whose
  heads may meet are E0202 where declared, whatever their bounds. A chosen
  impl with a written head whose bounds fail is E0606, never a fall back to
  an impl of a lower rank. Its body holds only the members of the trait
  and its supertraits: any other name is E0203 (`Sized` has no fn `totl`), with the closest member or
  a move to `impl T:` as help. (R08.1.2)
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
  the other way, so `self: Range<Int>` beats `self: Range<D>` and
  `[V]List<V>` beats `[D]T`. A concrete `self` beats a bare `self: S`
  whose bounds it meets: `self: Str` beats `self: S` with
  `S: Indexed<D, T>`, and `self: Range<Int>` beats `self: S` with `S: Seq<T>`.
  Two bare `self: S` are never more or less specific. Two equally specific
  candidates are an ambiguity error in different modules (E0701); in one
  module the second declaration is E0702 when the two overlap, that is
  when one call could take both: their full signatures unify, parameter
  by parameter after `self`, with every bound met (`x: T` with `T: Eq`
  takes no lambda, so it and `p: T -> Bool` don't overlap). The relation
  is the same within a module and across modules. One precedence order
  picks among the candidates: a member of the receiver's type, inherent
  or a method of a trait it has an impl of, beats any extension fn; at equal specificity, a fn the
  calling module declares beats an imported one, from std or any other
  library; two candidates that tie at one level are E0701. Members are found
  before extension fns, so an extension fn with a `self` of its own
  shape that a member of the receiver's type always wins over is never
  called; that call is E0701, naming the member (`fn sum(self: List<Int>)`
  beside `List.sum`), with help to rename the fn or call it as a plain
  fn. The error belongs to the module that declares the fn: a file that
  imports it calls the member with no error. So `fn sum(self: Range<Int>)`
  beside std's `Seq.sum` is E0701: a closed-form `Range` sum is the
  `impl Seq` override of `Seq.sum`, not an extension fn. To give a type its own cost, override the trait method instead
  (decision 486). A `@free_form` method's plain call `f(x, ..)` is the
  method call `x.f(..)` and picks by these same rules (R01.10.1). (R08.1.4)
- Operators are trait methods (`Add`, `Sub`, `Mul`, `Div`, `Rem` with
  `Rhs` and `Out`, `Neg`, `BitAnd`, ..., `Index`, `By`, `Ord`, `Eq`, `Hash`,
  `Show`, `Read`, `Unwrap`). `Eq<Rhs = Self>` and `Ord<Rhs = Self>` take
  the right side's type the way `Add` does, so `impl Eq for X` is
  `Eq<X>` and `a == b` with `b` of another type needs an
  `impl Eq<B> for A` (no other mixed comparison type-checks: `1 == "a"`
  is E0606). std's one walk-order impl over `Seq` is how a view and a
  list compare (R07.4.2). A `?T` compared with the `None` value is a
  presence test and needs no `Eq` (R02.4.5). Compound assignment comes
  from the operator. (R08.1.5)
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
  when a field meets each member the trait and its supertraits leave
  without a body, and then has the trait's default methods as its own:
  `(3, 4).rev()` is `Reversible`'s identity. A named struct satisfies it
  only through an `impl`, bodiless when its fields meet those members.
  One relation decides a bound, a parameter typed by the trait, `if T:
  Trait` and a default method's receiver (decision 361). (R08.1.7)
- Library-authored diagnostics: a trait may attach a message to "not
  implemented" and to each law, so `SegTree(f, a)` with a non-associative
  `f` says what to write. `@on_unimplemented(code: E0311, "...{T}...")`
  on a trait gives a missing impl that code in place of E0606 and the
  message as its help, `{T}` the type shown; each part is optional. A
  fn's `@reject(when: T = Type, "..", code: C)` on a defaulted parameter
  says the same of a default that is wrong for some types (R06.4.17).
  (R08.1.8)
- **An impl chooses each associated type** its trait declares, with
  `type Name = T` in its body, and impls of one trait for different kinds
  of type may choose differently: `impl<T> Op<T> for Monoid<T>` chooses
  `type Fold = T` and the impl for a bare `(T, T) -> T` chooses `?T`, so one
  generic `SegTree(op, a)` folds `T` over a monoid and `?T` over a closure
  (`max`/`min` are monoids with `Fold = T` over a bounded `T`, and `?T`
  otherwise, chosen by one impl with `if T: Bounded:`; decision 602)
  (decision 456). A bare operator takes the impl for a struct with a
  `@from_op` static fn (`Monoid.of` in std) and becomes that fn's call.
  A fn type is a type an impl may be for, and an impl may
  cover every domain (`impl<domain D> Endpoint for D`) beside one for a
  concrete type (`impl Endpoint for F64`). A generic fn names the choice as
  `M.Name` (R08.2.7), and its body is checked against the bound alone
  (R08.2.2): it can't assume `M.Name` is one type, so `fn f<B: Box>(b: B) ->
  Int = b.get()` is E0302. A trait may give an associated type a default
  (`type Item = Int`): an impl that leaves it out gets the default, and
  `type Item = T` in the impl overrides it, so adding a defaulted
  associated type breaks no existing impl (decision 469). An impl that
  leaves out one with no default is E0606 naming it. Two impls that apply to one type are
  E0202 (R08.1.2), so a type has one choice. (R08.1.9)
- **Each fn item has a type of its own**, written `type_of(f)` for a
  `fn` declared anywhere: a type of size zero that coerces to the fn's
  type wherever one is wanted, so `var g = mn` then `g = mx`, `[mx, mn]`
  and `if c: mx else: mn` all hold the fn type, and a fn-typed parameter
  takes any fn. A generic fn has one item, `type_of(mx)`, not one per
  instantiation. `type_of(x)` of anything else, a `let` holding a
  closure, a parameter or a type, is E0324; of a name nothing defines,
  E0201. (R08.1.10)
- **An impl may be for `type_of(f)`**, with the fn declared in the impl's
  module or the trait in its own: `impl Op<Int> for type_of(mx)`. Where
  the fn itself is passed at a bound, its impl is the one chosen, while a
  closure or another fn of the same type is not. Two impls of one trait
  for one item are E0202. (R08.1.11)
- **Inside a fn a trait owns**, a default, a law or one without `self`, a
  bare name is looked up in the locals and parameters first, then in the
  trait's own members without `self`, a supertrait's included, as
  `Self.name` would find them, then in the scopes around the trait. So a
  law or default writes `op(a, b)` for `Self.op(a, b)`, a parameter named
  `op` is still the parameter, and a module's `fn op` is hidden inside the
  trait (decision 684). (R08.1.12)

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
  implementing it" (decision 146). A method's bounds, its own and its
  impl's (`impl<F: Tag> S<F>`), are checked at each call as a free fn's
  are: `s.apply(x)` with a `NoTags` for `F` is E0606. (R08.2.1)
- Generic bodies are checked once against their bounds, at definition;
  errors point at the generic or the call, never inside an
  instantiation. (R08.2.2)
- Everything is monomorphized; there are no vtables and no `dyn`. (R08.2.3)
- A generic fn may not reach itself, directly or mutually, at a type that
  grows on every call: `wrap([x], n - 1)` inside `wrap<T>` would need
  `wrap<List<T>>`, `wrap<List<List<T>>>` and so on (E0319). (R08.2.10)
- A fn value with unwritten parameter types (an operator such as `+`, an
  overload set such as `max`, or `(a, b) => a * b + 1`) given to a type
  parameter nothing else fixes takes `Int` parameters, as a literal does:
  `ap(+)` for `fn ap<X>(op: X) -> X` is an `(Int, Int) -> Int`, and so is
  `SegTree(max, xs)`'s op. When its body does not check with `Int`s it is
  E0806, never an unknown type (decision 592). (R08.2.11)
- `domain` is a kind on type parameters; `const N: Int` parameters take
  compile-time values (`Mod<998244353>`, `SegTree<0..=4e9>`); range
  types are types and may be arguments. (R08.2.4)
- Type-parameter defaults fill omitted trailing arguments. Explicit arguments on a static call, `SegTree<V, Int>.of(max, f)` or `Box<Int>.empty()`, fix those parameters before inference, with the defaults filling the rest, so they decide the result type as they do on a constructor. A generic `fn` or type named as a value with its arguments, `id<Int>` or `Pair<Int>`, is the closure that calls it with them, as `id` alone is; a name without type parameters takes none (E0302). (R08.2.5)
- **Trait aliases** name a bundle of bounds:
  `trait WeightedNext<V, W> = Next<V> where Arc: Target<V> + Weighted<W>`. (R08.2.6)
- **Associated types** are reached through the implementing type or a
  value: `S.Item`, `g.Arc`. (R08.2.7)
- Return-type dispatch: a call may be resolved by its expected type
  (`read<T>()`). (R08.2.8)
- **Conditional code** on a bound: `if T: Commutative: ... else: ...`
  chooses at compile time, per instance, with no run-time test; it is how
  laws select a specialization (`lib/ops-ranges.md`). A trait's args are
  part of the bound tested: inside `fn pick<T, M: Op<T>>`,
  `if M: Selective<T>:` holds only when `M` implements `Selective` at this
  `T`, and the branch it guards sees `M: Selective<T>`. `if const cond:` branches on a compile-time
  value; `while const` is an error, since a `while` test runs every round. (R08.2.9)

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
- A struct that declares `law`s has a derived member
  `@get laws(self) -> List<(name: Str, sizes: S -> List<Int>, holds: (S, List<Int>) -> Bool)>`,
  one entry per law in declaration order, so a library checks laws with an
  ordinary body. `S` is the samples: `List<T>` when every law parameter has the
  one type `T`, else a record of one list per parameter type, named by the
  type, `(T: List<T>, F: List<F>)`. `sizes(s)` is the length of each
  parameter's list, and `holds(s, picks)` is the law with parameter `k` bound
  to element `picks[k]` of its list. A trait that declares laws derives the
  same member as a default, so each impl's copy binds the trait's parameters to
  the impl's args and calls the impl's fns: `impl Op<Int> for S` gives
  `S(0).laws` over `List<Int>`. A struct or impl that writes its own `laws`
  keeps it. (R08.3.5)
- Laws are never assumed, but each law's body is type-checked like a trait
  default: once, for every instance, as a `Bool`. What it needs of a type
  parameter it states, `law select(a: T, b: T) where T: Eq = ...`, so `==`
  on a `T` with no `Eq` bound is E0606, and a body that is not a `Bool` is
  E0302 (decision 683). (R08.3.6)

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
- `impl<V, R: (to: V, ..)> Tg<V> for R` is an impl for every record with
  those fields: a record meeting a blanket impl of the same trait too takes
  this one, and two such impls of one trait overlap, E0202. `impl Tr for _..=_`
  is an impl for every range type, whose body names it `Self`: a range type
  takes it before an impl for `Int`, so `(1..=3).at(9)` is `None` and
  `read<List<1..=3>>()` reads each item as `1..=3`. (R08.4.5)

## 8.5 What is deliberately missing

- No inheritance, subclassing or default-method overriding chains. (R08.5.1)
- No overloading by type, no implicit conversions, no orphan impls. (R08.5.2)
- No higher-kinded types; capability traits on sequences
  (`07-sequences.md`) and associated types cover the stdlib's needs. (R08.5.3)
- No user-defined patterns (decision 132) and no macros. (R08.5.4)
