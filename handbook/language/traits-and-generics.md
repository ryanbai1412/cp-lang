# Traits and generics

| | |
|---|---|
| Kind | Language design |
| Owner | spec |
| Leads | `spec/core/08-traits-generics.md` |
| Describes | `spec/core/08-traits-generics.md`, `spec/lib/ops-ranges.md` §1 to §2, `spec/conformance/core-08/` |

Why CP Lang abstracts with traits, monomorphized generics and laws, and
why it leaves out inheritance, overloading, `dyn` and custom patterns. A
*law* is a Boolean property a trait promises; *operation value* means an
operator or function passed as a value, such as `+` or `max`
(`spec/GLOSSARY.md`).

## The problem

A contest library is reused under time pressure, and generic code in the
usual languages fails in one of three ways:

```cpp
// 1. Templates check nothing until instantiation, then report from inside the library.
template <class T> T fold(vector<T>& a) { T s{}; for (auto& x : a) s = s + x; return s; }
fold(points);   // 40 lines of "no match for operator+" inside fold, none in your code

// 2. Wrapping an operation to fit a library: a struct per monoid.
struct MaxOp { static int id() { return INT_MIN; } static int op(int a, int b) { return max(a, b); } };
segtree<int, MaxOp::op, MaxOp::id> st(n);

// 3. std::function inside a structure: one indirect call per node visit.
SegTree st(n, [](int a, int b) { return max(a, b); });   // stores std::function: 1.6x to 4.4x slower
```

Python and Java pay the third cost everywhere through dynamic dispatch.
Rust traits fix the first, but the second survives as newtype wrappers
(`Reverse`, `Wrapping`), and an operation's algebra (associative?
commutative? idempotent?) lives in comments, so a sparse table built on
`+` gives wrong answers with no warning.

## The model

<!-- given:
  struct Frac: p: Int, q: Int
  impl Eq for Frac:
      fn eq(a, b) = a.p * b.q == b.p * a.q
  let a = [3, 1, 2] -->
```cp
trait Ord(Eq):
    fn cmp(self, other: Self) -> Ordering
impl Ord for Frac:
    fn cmp(self, o: Frac) -> Ordering = (self.p * o.q).cmp(o.p * self.q)
fn argmax<S: Keyed + Seq, K: Ord>(s: S, by: S.Item -> K = x => x) -> ?S.Key = todo
let st = SegTree(max, a)        // the operation is a value; its laws are known
```

What a contestant holds in their head:

1. A trait names a capability; an `impl` gives a type that capability.
2. Generic code bounds its type parameters by traits, is checked once at
   its definition, and is copied per type at compile time
   (monomorphized), so it runs as fast as hand-written code.
3. Operations are passed as values (`+`, `max`, `gcd`, a lambda), and a
   trait's laws say what algebra they promise, so a library can pick the
   fast algorithm the laws allow.
4. Records satisfy field traits by shape; structs opt in with an `impl`.
5. One name, one meaning: no overloading, no inheritance, no hidden
   conversions.

## Key decisions

### Operation values, not wrapper types (decision 3.1)

**Chosen.** Range structures and folds take bare operators and functions:
`SegTree(max, a)`, `a.scan(+)`, `xs.fold(gcd)`, `desc(x)` to flip an
order. An identity, when needed, comes from the operation's trait
(`Monoid`), or from an `(id, op)` pair. `Action` is the one extra shape,
for lazy segment trees.

**Lost.** The algebraic wrapper zoo (`Sum`, `Max`, `Order`, `Semiring`
as contestant-facing types), which the language had and deleted: every
use was a struct written to satisfy a library, the C++ `MaxOp` problem
again. The detailed surface is `spec/lib/ops-ranges.md` §1 to §2.

### Monomorphize everything; no `dyn` (decision 3.3, R08.2.3)

**Chosen.** Every generic function, struct and closure is specialized per
type. A lambda passed to `SegTree` is a type parameter, so the call
inlines.

**Lost.** `dyn Trait` as an escape hatch: removed after zero uses across
299 ports. Runtime polymorphism in contest code is almost always an enum,
which CP Lang has. Decision 3.2 settles specialization per type and per
function identity: code chosen at run time is an error, not a fallback.
The cost is compile time on very generic code.

### Generic bodies are checked once (R08.2.2)

**Chosen.** A generic body is type-checked against its bounds at its
definition. Errors point at the generic or the call, never inside an
instantiation, which fixes the C++ template error wall above.

**Lost.** C++ templates (check on instantiation) and C++20 concepts as an
opt-in: when bounds are optional, library code skips them and the error
wall returns.

### Laws are promises the library uses and the tools test (decision 3.4, R08.3.1 to R08.3.4)

**Chosen.** A trait states its algebra as `law` declarations:
`law assoc(a, b, c) = op(op(a, b), c) == op(a, op(b, c))`. Implementing
the trait promises the laws. The type checker never assumes them; they
select documented, answer-preserving specializations, such as a segment
tree without push-down when tags commute, or a sliding fold that
subtracts when the operation is a group. `cpl explain` names each
selection. `cpl stress` samples laws on the values a program builds,
`laws_hold(op, samples)` checks them directly, and a false law gives a
wrong answer, never undefined behaviour.

**Lost.** Proving laws (dependent types, Lean-style): far beyond what a
contestant will write in a contest. Leaving algebra in comments (C++,
Rust): nothing then stops `SparseTable(+)`, and nothing lets the library
use commutativity safely. A trait may attach a message to each law, so
`SegTree(f, a)` with a non-associative `f` says what to write (R08.1.8).

### Operators are trait methods, heterogeneous where needed (decision 59, R08.1.5)

**Chosen.** `a + b` calls `Add`; `Add`, `Sub`, `Mul`, `Div` and `Rem`
take an optional `Rhs` type and an `Out` associated type, so
`Vec * F64` and `Mat2 * Vec` coexist with `Mat2 * Mat2` on one struct.
An operator is declared like any function: `fn (a: Frac) + (b: Frac) ->
Frac`. Compound assignment comes from the operator.

**Lost.** Overloading by type, which C++ uses for the same job (R08.5.2,
decision 127). With `Rhs` the trait, not an overload set, picks the
implementation, so errors and hover name one candidate.

### Derived impls are structural (R08.1.6)

**Chosen.** Tuples, records, structs and enums get `Eq`, `Ord`
(lexicographic in declaration order), `Hash`, `Show` and `Read` when every
field has them. Tuples and records also get fieldwise `+`, `-` and `Zero`,
so a rerooting DP over `(cnt: Int, sum: Int)` passes `+` as its
operation.

**Lost.** Opt-in `#[derive(...)]` (Rust): one more line per struct, and
forgetting it is a compile error at the use site. Structs don't get `+`
because their fields may be private.

### One trait-satisfaction relation: records by shape, structs by `impl` (decision 361, R08.1.7)

**Chosen.** A trait may require fields (`trait HasW: w: Int`), and a
bodiless `@get fn` member is the same requirement. A record or tuple
satisfies such a trait when its fields meet the members; a named struct
satisfies it only through an `impl`, which may be bodiless. The same
relation decides a bound, a parameter typed by a trait, and
`if T: Trait`.

**Lost.** Two relations, which the implementation once had: `if A: HasW`
was false for a record that the bound `A: HasW` accepted. And structural
satisfaction for structs too (Go interfaces): structs keep their fields
private, so they opt in.

### Row bounds, not record subtyping (decision 146 rules, R08.4.1 to R08.4.4)

**Chosen.** `R: (u: V, v: V, ..)` accepts any record with at least those
fields, and `..R without (u, v)` names the rest, so `undirected` keeps an
edge's extra fields on its arcs. There is no subtyping: a record with
more fields fits only where the parameter has a row bound. Each row
function is monomorphized per record type, and nine safety rules keep
rows from changing layout or hiding copies. This is a trial.

**Lost.** Structural subtyping (TypeScript), which makes "which fields
does this value have?" a whole-program question.

### Graphs are a trait with arc capabilities (decision 146)

**Chosen.** Algorithms take `Next<V>`: `fn out(self, u: V) -> yields Arc`.
`Graph`, any function `V -> Seq` and any array of sequences implement it.
What an arc carries is a trait on the arc type (`Target<V>`,
`Weighted<W>`, `Edged<E>`), and trait aliases bundle them:
`WeightedNext<V, W>`. So `dijkstra` runs on an implicit grid graph with
no adapter.

**Lost.** Arcs stored in a side table: rarely worth it, since `Next` carries arc capabilities.

### Conditional code and type arguments (R08.2.5, R08.2.9, decision 116)

**Chosen.** `if T: Commutative: ... else: ...` picks a branch at compile
time; it is how laws select a specialization. Explicit type arguments
fill parameters before inference: `SegTree<V, Int>.of(max, f)`. They
never select a different function (decision 116).

**Lost.** C++ SFINAE and specialization by partial ordering, where a type
argument can silently pick another overload.

### What is deliberately missing (R08.5.1 to R08.5.4)

| Missing | Why |
|---|---|
| Inheritance, subclassing, override chains | Contest code models data, not taxonomies; traits plus default bodies cover reuse |
| Overloading by type, implicit conversions, orphan impls | One name, one meaning; errors name one candidate |
| Higher-kinded types | Sequence capabilities and associated types cover the stdlib |
| User-defined patterns, macros (decision 132) | A custom pattern runs code while looking like a field read, and defeats exhaustiveness; a view is a method returning an enum: `match t.node(v): Leaf => ...` |

Method lookup is inherent methods first, then traits in scope; two
candidates at one step is an error naming both, resolved with
`Trait.m(x)` (R08.1.4). Generic code calls the receiver's own impl, so a
default a type overrides is overridden everywhere. A type that wants its
own cost for a fold overrides the trait method (decision 486), which is
how `Range.sum` becomes a closed form.

## Worked examples

A trait default, overridden, seen through a generic:

```cp
trait Tot:
    fn tot(self) -> Int = 0
struct A(x: Int)
struct B(x: Int)
impl Tot for A
impl Tot for B:
    fn tot(self) -> Int = 5
fn g<T: Tot>(v: T) -> Int = v.tot()
print(g(A(1)), g(B(1)), B(1).tot())
```

`Ord` on a fraction, so `sorted` works with no comparator:

```cp
struct Frac: p: Int, q: Int
impl Ord for Frac:
    fn cmp(self, o: Frac) -> Ordering = (self.p * o.q).cmp(o.p * self.q)
let xs = [Frac(1, 2), Frac(1, 3), Frac(2, 3)].sorted()
print(xs[0].p, xs[0].q, xs[2].p, xs[2].q)
```

An operator is a function:

```cp
struct Frac:
    n: Int
    d: Int
fn (a: Frac) + (b: Frac) -> Frac = Frac(n: a.n * b.d + b.n * a.d, d: a.d * b.d)
let c = Frac(n: 1, d: 2) + Frac(n: 1, d: 3)
print(c.n, c.d)
```

Derived `Ord` and `Eq` on a struct:

```cp
struct P: a: Int, b: Int
let xs = [P(2, 1), P(1, 5), P(1, 3)].sorted()
print(xs[0].a, xs[0].b, yes(P(1, 2) == P(1, 2)))
```

**An impl chooses its associated types (decision 456).** `SegTree(f, a)`
should fold to `?T` when `f` has no identity, since an empty range has no
answer, and to `T` over a monoid. `(lo..hi).bsearch` should return a
`Cut<D>` on a domain and a `?F64` on reals, and `split(sep:)` should take a
value or a predicate. Fns don't overload by parameter type (R07.1.7), so
each of these needed a second name (`bsearch_real`, a `Str`-only `split`) or
a wrapper at every call (`f.opt()` and `Some` cells). An associated type
answers all three with one mechanism: `Op<T>` declares `type Fold`, the
monoid impl chooses `T`, the impl for a bare closure chooses `?T`, and
`SegTree`'s folds return `M.Fold`. Generic code sees only `M.Fold`, never one
impl's choice, so it is still checked once against its bound (R08.1.9). The
cost is one concept a contestant meets mostly in library signatures, and
Rust programmers know it already.

**An associated type can have a default (decision 469).** A trait may
write `type Item = Int`; an impl that says nothing gets `Int`, and one that
writes `type Item = List<Int>` gets its own choice. This is what lets a
trait grow: `Seq` can gain `type Taken`, so `take(k)` is a slice on a list
and a lazy walk on a generator, without every existing `impl Seq` having
to name a type it never uses. The alternative, a second `take` chosen by
which trait is more specific, would be overloading by type, which CP Lang
leaves out (R08.5.2).

Associated types and trait aliases:

```cp
trait Box:
    type Item
    fn get(self) -> Self.Item
struct IntBox: v: Int
impl Box for IntBox:
    type Item = Int
    fn get(self) -> Int = self.v
fn both<B: Box>(b: B) -> (B.Item, B.Item) = (b.get(), b.get())
let (x, y) = both(IntBox(4))
print(x + y)

trait Sh:
    fn area(self) -> Int
trait Nm:
    fn name(self) -> Str
trait Both = Sh + Nm
struct Sq: s: Int
impl Sh for Sq:
    fn area(self) -> Int = self.s * self.s
impl Nm for Sq:
    fn name(self) -> Str = "sq"
fn show<T: Both>(v: T) = print(v.name(), v.area())
show(Sq(3))
```

Laws checked directly, and code conditional on a bound:

```cp
print(yes(laws_hold(+, [1, 2, 3])), yes(laws_hold(-, [1, 2, 3])))
trait Fancy
struct A: x: Int
struct B: x: Int
impl Fancy for A
fn kind<T>(x: T) -> Str:
    if T: Fancy:
        return "fancy"
    return "plain"
print(kind(A(1)), kind(B(1)))
```

Field traits (a record by shape, a struct by `impl`) and a row bound:

```cp
trait HasW: w: Int
fn weight<T: HasW>(x: T) -> Int = x.w
struct Road: w: Int, len: Int
impl HasW for Road
print(weight((w: 3, z: 1)), weight(Road(4, 9)))
fn total<R: (w: Int, ..)>(xs: List<R>) -> Int = xs.map(r => r.w).sum()
let a = [(w: 2, u: 9), (w: 3, u: 1)]
print(total(a), a[0].u)
```

# Reference

## Where the precise rules are

| Topic | Spec |
|---|---|
| The model | `spec/core/08-traits-generics.md` "The model in one paragraph" |
| Traits, impls, lookup, operators, derived impls, field traits | §8.1, R08.1.1 to R08.1.8 |
| Generics, bounds, monomorphization, aliases, associated types, `if T: Trait` | §8.2, R08.2.1 to R08.2.9 |
| Laws | §8.3, R08.3.1 to R08.3.4 |
| Record row types (trial) | §8.4, R08.4.1 to R08.4.4 |
| What is deliberately missing | §8.5, R08.5.1 to R08.5.4 |
| Operation traits, built-in operation values, your own operation | `spec/lib/ops-ranges.md` §1 to §2 |
| Library powers (no privileged stdlib) | `spec/core/12-powers.md` |
| Conformance cases | `spec/conformance/core-08/` |

## Changes

- 2026-11-23: first version.
