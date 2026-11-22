# 8. Traits, generics, laws and row types

Status: draft for 0.3.0. Sources: archived `design/spec/14`; decisions
59, 116, 127, 142, 146 (row types, trial), round 5 (`lib/ops-ranges.md`)
for laws; `12-powers.md` for what user libraries can define.

## 8.1 Traits and impls

```
trait Ord(Eq):
    fn cmp(self, other: Self) -> Ordering

trait Next<V>:
    type Arc: Target<V>
    fn next(self, v: V) -> yields Arc

trait Monoid<T>(Op<T>):
    fn id(self) -> T
    law left_id(x: T) = self.op(self.id, x) == x

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
  both, resolved by `Trait.m(x, ...)`. No auto-ref or auto-deref. (R08.1.4)
- Operators are trait methods (`Add`, `Sub`, `Mul`, `Div`, `Rem` with
  `Rhs` and `Out`, `Neg`, `BitAnd`, ..., `Index`, `Ord`, `Eq`, `Hash`,
  `Print`, `Read`, `Unwrap`). Compound assignment comes from the
  operator. (R08.1.5)
- **Derived impls are structural**: tuples, records, structs and enums
  get `Eq`, `Ord` (lexicographic in declaration order, variants in
  declaration order), `Hash`, `Print` and `Read` when every field has
  them, unless the type writes its own impl. (R08.1.6)
- **Field traits**: a trait may require fields (`trait Target<V>: to:
  V`). Records satisfy field-only traits structurally; named structs opt
  in with a bodiless `impl Target<V> for Edge`. (R08.1.7)
- Library-authored diagnostics: a trait may attach a message to "not
  implemented" and to each law, so `SegTree(f, a)` with a non-associative
  `f` says what to write. (R08.1.8)

`cmp(a, b)` is the three-way comparison of two `Ord` values, an
`Ordering` (`Less`, `Equal`, `Greater`); `desc(x)` flips an order.

## 8.2 Generics

```
fn argmax<S: Keyed + Seq, K: Ord>(s: S, by: S.Item -> K = id) -> ?S.Key
fn dijkstra<V, W>(g: WeightedNext<V, W>, from: Seq<(V, W)>) -> Paths<V, W, g.Arc>
fn f<domain I, domain J>(a: [I, J]Int)
struct Pair<A, B = Int>: a: A, b: B
```

- Bounds go inline (`T: Ord + Hash`), in a `where` clause, or directly as
  a parameter type: `g: WeightedNext<V, W>` means "some type
  implementing it" (decision 146). (R08.2.1)
- Generic bodies are checked once against their bounds, at definition;
  errors point at the generic or the call, never inside an
  instantiation. (R08.2.2)
- Everything is monomorphized; there are no vtables and no `dyn`. (R08.2.3)
- `domain` is a kind on type parameters; `const N: Int` parameters take
  compile-time values (`Mod<998244353>`, `SegTree<0..=4e9>`); range
  types are types and may be arguments. (R08.2.4)
- Type-parameter defaults fill omitted trailing arguments. (R08.2.5)
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

```
trait Op<T>:
    fn op(self, a: T, b: T) -> T
    law assoc(a: T, b: T, c: T) = self.op(self.op(a, b), c) == self.op(a, self.op(b, c))
trait Commutative<T>(Op<T>):
    law comm(a: T, b: T) = self.op(a, b) == self.op(b, a)
```

- A `law` is a Boolean property of a trait's operations. Implementing a
  trait promises its laws (trust tier "promised", chapter 9). (R08.3.1)
- Laws are never assumed by the type checker. They select documented,
  answer-preserving specializations (a segment tree without push-down
  when tags commute), and `cpl explain` names each selection. (R08.3.2)
- `cpl stress` samples laws on the values a program actually builds;
  `laws_hold(op, samples)` checks them directly; `--laws=off` skips. (R08.3.3)
- A false law gives wrong answers, never undefined behaviour. (R08.3.4)

## 8.4 Record row types (trial)

```
fn undirected<V, R: (u: V, v: V, ..)>(n: V.count, edges: Seq<R>)
    -> Graph<V, (to: V, ..R without (u, v))>
let a2 = (..a, to: u)                 // record update
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
  access through a row is O(1), and rows never change layout of an
  existing value. (R08.4.4)

## 8.5 What is deliberately missing

- No inheritance, subclassing or default-method overriding chains. (R08.5.1)
- No overloading by type, no implicit conversions, no orphan impls. (R08.5.2)
- No higher-kinded types; capability traits on sequences
  (`07-sequences.md`) and associated types cover the stdlib's needs. (R08.5.3)
- No user-defined patterns (decision 132) and no macros. (R08.5.4)
