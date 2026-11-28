# Powers: what a user library can define

A power is a mechanism that lets library code feel built in. The stdlib has
no power a user library lacks: if a stub in `std/` needs something, it
is listed here, or it is core syntax in `spec/core/`. A power earns its place
by the stdlib families it unlocks; one family means look for another design.

Each entry: the decision, the smallest example, what needs it, and whether
`scripts/spec-check.sh` already accepts it (**checks**), accepts part of it
(**partial**) or not yet (**no**). Lowering for every power is monomorphized,
inlined C++; any `std::function` or hidden allocation is logged debt.

## W1. Numeric types (P1)

A type becomes a number by defining operators, `Int` literal conversion and
`read`/`print`. Compound assignments (`+=`, `min=`) come from the operators.

    struct Frac: p: Int, q: Int
    impl Num for Frac:
        fn from_int(n: Int) -> Frac = Frac(n, 1)
        fn (a: Frac) + (b: Frac) -> Frac = norm(a.p * b.q + b.p * a.q, a.q * b.q)
        fn (a: Frac) * (b: Frac) -> Frac = norm(a.p * b.p, a.q * b.q)
    let half: Frac = 1 / 2          // literals convert through from_int

Range facts (what the overflow checker may assume about a result) are
declared on the return type: `fn (a: Mod<P>) + (b: Mod<P>) -> 0..P`.
Needs: `Mod<P>`, `Frac`, `Matrix`, `Poly`, `Point`, `I128`. Status: partial
(operators check; literal conversion through `from_int` does not).

## W2. Niches (P2)

`?T` costs no space when `T` has an unused value. Ints, domains, ranges,
references and enums have one; a struct has one when any field has one, and
`?S` uses the first such field. Automatic, never worse (it majorizes), shown
by `cpl explain`. No syntax. Needs: every optional record (`?Edge`,
`?Pair`), `[V]?Int` distance arrays. Status: checks (layout is codegen debt).

## W3. Reflection and derives (P3)

Every struct and enum gets, by field order: `==`, `<` (lexicographic), hash,
`print`, `read`, and, when every field is a finite index, an index domain
(decision 2.16). A library derives more with a compile-time loop over fields:

    pub fn show<T: Record>(x: T) -> Str = ", ".join(for f in T.fields: f"{f.name}={x.(f)}")

`T.fields`, `T.variants` and `x.(f)` are resolved at compile time, so the
loop unrolls. Needs: sorting records, hashing tuples, `[Sign]Path`, trace
printing. Status: partial (built-in derives check; `T.fields` does not).

## W4. Sequences, views and index spaces (P4)

One iteration protocol: anything with `iter` (a generator) is a `Seq` and
works in `for`, comprehensions and every sequence method. Five more
capabilities layer on it; each operation asks for the weakest one that
keeps it at its stated cost (decision 142):

    pub trait Seq<T>:                    fn iter(self) -> yields T          // walk in order
    pub trait Keyed<K, T>: Seq<T>:       fn items(self) -> yields (K, T)    // each element knows its key
    pub trait Sized:                     len: Int                           // \(O(1)\) length
    pub trait Bidir<T>: Seq<T>:          fn iter_rev(self) -> yields T      // walk backwards
    pub trait Indexed<K, T>: Keyed<K, T>, Sized, Bidir<T>:
        fn at(self, k: K) -> T                                              // \(O(1)\) s[k]
        keys: Range<K>                                                      // contiguous keys
    pub trait IndexedMut<K, T>: Indexed<K, T>:
        fn at(mut self, k: K) -> lends mut T                                   // s[k] is a place
    pub trait Contains<T>:               fn contains(self, x: T) -> Bool    // `x in s`, sub-linear only

| Needs | Operations |
|---|---|
| `Seq` | `map`, `filter`, `take_while`, `chain`, folds, `min`/`max`, `any`/`all`, `first(p)`, `tally`, `runs`, `pairs`, `keyed()`, `collect()`, `sorted()` (R12.4.1) |
| `Keyed` | `.items`, `find`, `find_all`, `argmax`/`argmin`: walk and remember keys (R12.4.2) |
| `Sized` | `.len` (R12.4.3) |
| `Bidir` | `.rev`, `.last` as an \(O(1)\) property (R12.4.4) |
| `Indexed` | `s[k]`, slices, `nth`, `bsearch`, `first_ge`, string algorithms, `zip` (R12.4.5) |
| `IndexedMut` | `sort()`, `reverse()`, `swap`, `select`, `next_perm`, and `rng.shuffle(mut a)` from `std.rand`, slice assignment (R12.4.6) |
| `Contains` | `x in s`: sets, maps (keys), bitsets, ranges; a linear search is `s.any(x)` (R12.4.7) |

`at(k)` takes a key, the same key as `s[k]`; it is never a walk offset.
On `w = s[2..]`, `w.at(2)` is `s[2]` and `w.at(0)` traps. Walking by
position is `nth(k)`, which is `s.at(s.keys.nth(k))`, so generic code that
counts from 0 over any `Indexed` (`zip`, `select`, `next_permutation`)
reads with `nth` (see R04b.1.4). (R12.4.13)

Generators are `Seq` only: a yield number is not an identity, so
`dfs(s).argmax()` is an error suggesting `.keyed(by: v => v)` or `.max()`.
`s.keyed(keys)` makes any `Seq` `Keyed`, pairing by walk order (`keys`
defaults to `0..`, a counter); `s.keyed(by: f)` takes each key from its
value. There is no `enumerate`: `for (i, x) in g.keyed().items` (decision 60). Sortedness is a
precondition the checked build tests, not a trait. `Map` is `Keyed`,
`Sized` and `Contains` with its own `[]`; there is no `Lookup` trait.

Every value is owned or lent (read or `mut`); the mode is not part of the
type. Five rules, with no lifetime annotations (decisions 141, 144; the
full model with exercises is `spec/core/05-ownership.md`):

1. Parameters are lent: a fn never owns its arguments. (R12.4.8)
2. A value is lent if any part is: a struct's fields, a closure's captures,
   a generator value's arguments. It is `mut`-lent if any part is, and it
   borrows every source its parts borrow. (R12.4.9)
3. A lent value lives until its last use. It may be kept in a `let`, `var`,
   field or container that dies no later than its sources (same or inner
   block, declared after them); never returned except by a fn declared
   `lends`, stored through a parameter, or stored in a top-level `var`. (R12.4.10)
4. While it lives its sources are lent: a read lend forbids writing,
   resizing or moving them, and a `mut` lend excludes every other access.
   A closure's captures are borrowed per call, not for its life. (R12.4.11)
5. Owned values move: `let b = a` and `S(a)` move `a`; `a.clone()` copies. (R12.4.12)

A view is a `Seq` (and `Indexed` when reads stay \(O(1)\)) built from a lent
source. There is no `View` trait and no marker on the struct: the same
`Filter` owns its source when built from an owned value (`Filter(xs, p)`
moves `xs`) and borrows it when built from a lent one (`arr.filter(p)`,
since `self` is lent). A fn that returns a lent value says so in its
signature; by default the result borrows every lent parameter that is not
a plain value (`Int`, `Bool`, ranges, positions), and `lends(xs)` narrows
it. The body is checked against the signature, so call sites and `= todo`
stubs need nothing more:

    pub struct Filter<S, P>:
        src: S
        keep: P                      // the closure's own type: no boxing
    impl Seq<T> for Filter<S: Seq<T>, P: T -> Bool>:
        fn iter(self) -> yields T =
            for x in self.src:
                if self.keep(x): yield x
    impl Keyed<K, T> for Filter<S: Keyed<K, T>, P: T -> Bool>:
        fn items(self) -> yields (K, T) =       // keys survive filtering
            for (k, x) in self.src.items():
                if self.keep(x): yield (k, x)
    impl Bidir<T> for Filter<S: Bidir<T>, P: T -> Bool>:
        fn iter_rev(self) -> yields T =
            for x in self.src.rev:
                if self.keep(x): yield x
    pub fn filter<S: Seq<T>, T, P: T -> Bool>(self: S, keep: P) -> lends Filter<S, P> =
        Filter(self, keep)
    fn window(xs: [V]Int, r: Range<V>) -> lends [V]Int = xs[r]
    pub fn at(mut self, i: D) -> lends mut T       // a writable place: the same keyword

A `for` over a generator pastes its body at the loop and each `yield x`
becomes the loop body, so `for y in a.filter(p).map(f): b(y)` is
`for x in a: if p(x): b(f(x))`, and `break`, `continue` and `return` act on
that one loop. An adapter implements each capability it can keep \(O(1)\):
`Filter` keeps `Keyed` and `Bidir` but not `Sized` or `Indexed`, so
`a.filter(p).argmax()` walks `a.items()` once and answers with a key of `a`.
Recursive generators pass the body as a closure that
returns continue or stop. `Slice`, `Rev`, `Map`, `Filter`, `TakeWhile` and
`Chain` are each a few lines like this, with no stdlib privilege.

An index space is a domain: a declared or fresh finite type with `D.count`,
`D.cuts` (its cuts) and `D.at(i)`, or a library type, std or user, that
implements `Index` with `count`, `ord` and `nth` (R04.2.26). Generic code
takes `D: Domain`, which both kinds meet, walks `for d in D` and stores
`[D]T`; runtime-sized domains read their declaration-site slot (decision 124
and the summary in `decisions.md`).

    pub struct Fenwick<D: Domain, T: Group>: tree: [D]T
    pub fn fold(self, r: Range<D>) -> T = ...       // a cut range converts here

Needs: every container, `SegTree<D>`, `Fenwick<D>`, `Graph<V, E>`, `t.Pre`,
`SortedSet.range`, `Window`. Status: partial (domains and `[D]T` check;
`Seq` is informal, so some sequence methods exist only on `List`).

## W5. Patterns (P5): closed

Values destructure by their stored shape only. A view is a method returning
an enum: `match t.node(v): Leaf => ... Inner(l, r) => ...`. Decision 132,
with its reopening condition. Status: checks.

## W6. Generators and steering (P6)

`fn f(...) -> yields T` is a generator; `yield` produces, plain `return`
ends. A `for` over it supports `break`, labelled `break`, `return` and loop
`else`, including through recursive generators (decision 126). The item
type may carry methods that talk back to the generator; it checks a flag on
its own item after each `yield`:

    pub enum Visit<V, A>: Enter(v: V, via: ?A), Exit(v: V, via: ?A), Again(from: V, via: A)
    pub fn dfs<V, G: Next<V>>(g: G, from: Seq<V>, seen: ?mut [V]Bool = None) -> yields Visit<V, G.Arc>   // step.prune() after Enter

Closures passed to a library are guaranteed to inline when they do not
escape (fn parameters of non-recursive generics). Non-local `return` from a
closure is deferred until ports need it. Lockstep over two recursive
generators: materialize with `List(g)`.
Needs: `bfs`, `dfs`, `t.Pre`, `submasks`, `divisors`, `reach`, every search.
Status: partial (generators and `break` check and run; item steering and
recursive-generator `return` are implementation debt).

## W7. Functions as values (P7)

Operators and named functions are values: `SegTree(+)`, `keep: max`,
`by: <`, `xs.sort(by: p => p.x)`. Named and default arguments work in user
functions exactly as in the stdlib; options are named arguments, never `_by`
variants. Recursive `let` lambdas lower to a self-passing generic lambda
(decision 127). No overloading: a function is its name plus its labels.
Status: partial (operator values and defaults check; recursive `let`
lambdas do not parse yet).

## W8. User places (P8)

A type defines what `x[k]` is as a place with one of two protocols:

    pub trait IndexMut<K, T>: fn at_mut(mut self, k: K) -> mut T   // a real place
    pub trait IndexSet<K, T>: fn set(mut self, k: K, v: T)          // write-through

With `at_mut`, `mut x[k]`, `let p = mut x[k]` and `x[k] op= v` are ordinary
places. With only `set`, `x[k] = v` calls `set` and `x[k] op= v` is
`set(k, at(k) op v)`, with `k` evaluated once; a named alias is an error
pointing at the method. `SegTree` uses `set` (an update must recompute
ancestors); `Map` uses `at_mut` and inserts the default (decision 128).
Moves: returning any value is \(O(1)\); `.take()` moves out of a place that
stays live; parameters are lends, `mut` lends allow draining
(`a.absorb(mut b)`). Overlapping `mut` places are a compile error.
Needs: `SegTree`, `Fenwick`, `Map`, `Counter`, small-to-large.
Status: partial (`Map` places check; `IndexSet` desugars `x[k] = v` and `x[k] op= v` on a
struct with `set` and no `at_mut`; a computed `k` in `x[k] op= v` must be bound first).

## W9. Operators on user types (P9)

One spelling: `fn (a: T) op (b: T) -> R` defines `op`, unary `fn -(a: T)`,
comparison `fn (a: T) < (b: T) -> Bool` (or the derived order). It is the
`Add`/`Mul`/... protocol impl written the way the operator reads, so generic
code bounded by `T: Add` accepts it. `impl Add for T` is no longer written
by hand. Status: partial (`impl Add for` checks; the
`fn (a) + (b)` spelling does not parse yet).

## W10. Protocols, laws and operation values (P10)

Traits have `Self`, associated types and constants, and may state laws that
`laws_hold` tests and `cpl stress` samples:

    pub trait Add:
        fn (a: Self) + (b: Self) -> Self
        law assoc(a, b, c) = (a + b) + c == a + (b + c)

Algebra stays operation values (decision 3.1): a structure takes a binary fn
(`SegTree(+)`, `SparseTable(min)`, `scan(xs, gcd)`). Its identity comes from
the element type: `+` uses `T.zero`, `*` uses `T.one`, `min`/`max` over `?T`
use `None`; a user type supplies them as constants (`const zero = Low(...)`).
Anything else passes a `Monoid(id: e, op: f)`, as in
`SegTree(Monoid(id: e, op: f), init)`. A missing identity is
an error naming the constant to define. Lazy propagation takes an action
value `act: (f, x) => ...` with `compose:`. Const generics: `Mod<P>`,
`[n]T`, `Bitset<N>`.
Needs: `SegTree`, `LazySegTree`, `SparseTable`, `Fenwick`, `Matrix`, scans,
ordered containers (`Ord`), hashing (`Hash`). Status: partial (traits check;
laws, `T.zero` identities and `Self` at run time do not).

## W10a. Records, associated types and inline bounds (P10)

Graphs (decision 146) need five mechanisms; each is open to user code.

- **Associated types.** A trait names a type its implementor picks, read
  as `Self.Arc` or `G.Arc`:

      pub trait Next<V>:
          type Arc: Target<V>
          fn out(self, u: V) -> yields Self.Arc

- **Trait aliases.** `pub trait WeightedNext<V, W> = Next<V> where
  Self.Arc: Weighted<W>` names a bound; it has no members and no impls. (R12.11.1)
- **Inline bounds.** A trait in a parameter's type is a fresh hidden type
  parameter, static per caller, like `P: T -> Bool` for fns:
  `fn dijkstra<V, W>(g: WeightedNext<V, W>, from: Seq<(V, W)>)`. Name it
  (`<G: Next<V>>(g: G)`) when a return type or a second parameter mentions
  it. (R12.11.2)
- **Field traits.** A trait whose members are all `@get` is satisfied by
  any record with fields of those names and types: `(to: V, w: Int)` is a
  `Target<V> + Weighted<Int>`. A named struct opts in with a bodiless
  `impl Weighted<Int> for Road`. (R12.11.3)
- **Row bounds and record update.** `R: (u: V, v: V, ..)` bounds a type
  parameter to records with at least those fields; `(to: V, ..R without
  (u, v))` is R's other fields after new ones, in R's order; `(..a, to: u)`
  is `a` with `to` replaced (heap fields move), and `(to: e.v, ..e without
  (u, v))` is its value mirror: `e`'s other fields after the new ones, the
  value of the row type `(to: V, ..R without (u, v))`. No record subtyping: rows
  only bound type parameters, matched by monomorphization. A generic body
  sees only the named fields. (R12.11.4)

Needs: `Graph.directed`/`undirected` over any edge record, `keep:`,
`transpose`, every graph algorithm's signature, `.weighted(f)`.
Status: no (stubs load in spec mode; outside it the parser rejects row
bounds and `without` with E0108, record update E0102, aliases E0103, and
an inline bound is E0204).

## W11. Reading and printing (P11)

`read<T>()` and `print(x)` go through `Read` and `Show`, derived for
records (W3) and defined by hand for anything else. Format options are
named arguments: `read<[n]Int>(base: 1)`, `print(xs, sep: "\n")`.
A graph is read as its edge records and built from them; there is no
`Graph.read` (R10.2.5). Status: partial (`read` is still compiler-special for some types).

## W12. Costs and library lints (P12)

Every public signature's doc comment states its cost as LaTeX math:
`/// cost: \(O(\log n)\)` (the math convention in `spec/README.md`).
`cpl explain` and hover show it. Libraries declare lints with
`@lint(name, from: "...", to: "...")` on the fn they suggest (R06.4.9),
and `@on_unimplemented("...")` messages on traits. `assert` runs in checked
builds and, opted in as `asserts`, in release (chapter 9). Status: `@lint`
implemented; costs are a doc convention; `@on_unimplemented` implemented (decision 28.11).

## W13. Modules, pins and codemods (P13)

`use std.graph.{bfs, Graph}`; prelude names need no `use`. A library may
mark a name deprecated with a replacement that `cpl fix` applies:
`#deprecated(use: "xs.sort(by: f)")`. Status: partial (codemods are
compiler-side).
A std item states the names it replaced, `@was(Prefix1, PrefixN)` above
`pub struct Prefix`; a program writing an old one gets E0201 or E0204
naming the new one, with the fix (and a word built on the type, the
snake-case forms: `prefix_n`). Status: checks
(`std-prefix/was-diff1-names-diff.cp`). (R12.13.1)
A fn states its older names the same way, `@was(bsearch_last)` above
`pub fn last_true`; a program calling `r.bsearch_last(..)` gets E0503
naming the fn, with the fix. Status: checks
(`std-search/last-true-was-bsearch-last.cp`).
A member states its older names the same way, on its own shape only:
`@was(prefix)` above `pub fn cut` in `Prefix<T>`; a program calling
`p.prefix(k)` on a `Prefix<T>` gets E0503 naming `cut`, with the fix, while
`Prefix<D, T>` keeps its own `prefix`. Status: checks
(`std-prefix/list-shape-prefix-was-cut.cp`).
A fn states other languages' names for it, `@aka(upper, toUpperCase)` above
`pub fn to_upper(self: Str)`; a program calling `s.upper()` on that
receiver type gets E0203 naming the fn, with the fix. `@aka` is an
ordinary decorator: the checker reads it from std's declarations, so no
compiler table spells a std name (decision 633). Status: checks
(`core-12/aka-upper-names-to-upper.cp`, `core-12/aka-on-a-user-fn-runs.cp`).
On an `@operator` trait, `@aka` names other languages' names for the fn its
operator calls: `@aka(pow)` above `trait Pow` makes `a.pow(e)`, where
`pow` is found as `Pow`'s fn, E0203 with the fix `a ** e` (R01.11.1), while
a type's own `pow` is still its method. Status: checks
(`lints/pow-method.cp`, `core-12/aka-on-an-operator-trait-keeps-own-methods.cp`). (R12.13.2)
`@stress(kind)` above a type alias says how `cpl stress` makes and shrinks
its values: `perm`, `tree`, `simple`, `connected`, `dag` or `digraph`
(decision 25.32). std writes it above `Perm`, `Tree` and the graph
aliases; stress reads it on the declaration a written type resolves to, so
`type P = Perm<V>` is a permutation and a user's own undecorated `Tree` is
not, and a weighted tree is `@stress(tree) type Tree = [0..n - 1](u: V,
v: V, w: W)`. Anything but one bare kind, or `@stress` on anything but a
type alias, is E0325. `cpl run` and `cpl build` ignore it. Status: checks
(`core-12/stress-not-on-a-type.cp`, `core-12/stress-unknown-kind.cp`). (R12.13.3)

## W14. Inline C++ (P14)

`@cpp("...")` bodies are allowed in any library, with the signature checked
at the boundary and the C++ included once in the bundle. Status: partial.

`@intrinsic("id")` is the body form with a VM twin: it names one entry of the
compiler's intrinsics table, which holds that primitive's interpreter
implementation and C++ snippet. Like `@cpp`, any library may write it.

1. `fn f(params) -> R = @intrinsic("id")` is allowed in any module; `id`
   must name an entry of the intrinsics table, else E0210. (R12.14.1)
2. The fn's parameter and result types must be the entry's signature,
   else E0210 names the entry's signature. (R12.14.2)
3. A call of such a fn is the primitive itself: both engines run the entry,
   and the emitted C++ is the entry's snippet inline, with no call of `f`.
   `f` is still a fn value. (R12.14.3)
4. A module copied under another name, with the same `@intrinsic` bodies,
   behaves and emits C++ exactly as the original. (R12.14.4)
5. The table grows only by a decision entry; `cpl explain` lists each
   intrinsic a program uses. (R12.14.5)

The table's primitives are the public set of primitive operations, by id
(`cpl-prim` holds them): `popcount`, `abs`, `sqrt`, `floor`, `ceil`, `round`,
`trunc`, `ctz`, `clz`, `bit_width`, `bit_ceil`, `acos`, `asin`, `atan2`,
`cos`, `sin`, `tan`, `hypot`, `exp`, `ln`.

## W15. Precomputation (P15)

A top-level `let` is computed once before the first test (`let C =
Comb<Mod<P>>(N)`). Compile-time evaluation is not part of the spec:
contest tables are cheap at start-up, and baking them would grow the bundle.
Status: checks.

## W16. Contexts (P16)

`with io(base: 1)` (R10.2.2) and user contexts (chapter 9) are core
features a library can declare and read. Status: checks
(`core-10/counted-domain-base.cp`, `core-09/context-constant-value.cp`).
