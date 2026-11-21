# Powers: what a user library can define

A power is a mechanism that lets library code feel built in. The stdlib has
no power a user library lacks: if a stub in `spec/std/` needs something, it
is listed here, or it is core syntax in `SYNTAX.md`. A power earns its place
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
works in `for`, comprehensions and every sequence method (`count`, `first`,
`sum`, `sort(by:)`). `Sized` adds `.len`; `Indexed<K>` adds `xs[k]`.

    pub trait Seq<T>:      fn iter(self) -> yields T
    pub trait Sized:       len: Int                    // property, no parens
    pub trait Indexed<K, T>: fn at(self, k: K) -> T

A view is a struct declared `lends S`: its field of type `S` borrows the
source instead of owning it. There is no `View` trait; a view is a `Seq`
(and `Indexed` when reads stay O(1)) whose struct lends. `lends` gives it
the place rules of `mut a[i]` with no lifetime annotations:

- it lives at most as long as the source's scope, and while it lives the
  source can't change length; a `mut` view excludes overlapping access;
- it can't go in a field, container or global, or be captured by a closure
  that escapes; `.collect()` or `List(w)` copies it out;
- a fn returns one only by saying which parameter it lends:
  `fn window(xs: [V]Int, r: Range<V>) lends xs = xs[r]`.

    pub struct Filter<S, P> lends S:
        src: S
        keep: P                      // the closure's own type: no boxing
    impl Seq<T> for Filter<S: Seq<T>, P: T -> Bool>:
        fn iter(self) -> yields T =
            for x in self.src:
                if self.keep(x): yield x

A `for` over a generator pastes its body at the loop and each `yield x`
becomes the loop body, so `for y in a.filter(p).map(f): b(y)` is
`for x in a: if p(x): b(f(x))`, and `break`, `continue` and `return` act on
that one loop. Recursive generators pass the body as a closure that
returns continue or stop. `Slice`, `Rev`, `Map`, `Filter`, `TakeWhile` and
`Chain` are each a few lines like this, with no stdlib privilege.

An index space is a domain: a declared or fresh finite type with `D.count`,
`D.cuts` (its cuts) and `D.at(i)`. Generic code takes `D: Domain` and
stores `[D]T`; runtime-sized domains read their declaration-site slot
(decision 124 and the summary in `decisions.md`).

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

    pub enum Visit<V>: Enter(v: V), Exit(v: V)
    pub fn dfs<V>(g: Graph<V, _>, s: V) -> yields Step<V>   // Step has .v, .prune()

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
Moves: returning any value is O(1); `.take()` moves out of a place that
stays live; parameters are lends, `mut` lends allow draining
(`a.absorb(mut b)`). Overlapping `mut` places are a compile error.
Needs: `SegTree`, `Fenwick`, `Map`, `Counter`, small-to-large.
Status: partial (`Map` places check; `IndexSet` desugaring does not).

## W9. Operators on user types (P9)

One spelling: `fn (a: T) op (b: T) -> R` defines `op`, unary `fn -(a: T)`,
comparison `fn (a: T) < (b: T) -> Bool` (or the derived order). It is the
`Add`/`Mul`/... protocol impl written the way the operator reads, so generic
code bounded by `T: Add` accepts it. `impl Add for T` is no longer written
by hand (decision 133). Status: partial (`impl Add for` checks; the
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
use `none`; a user type supplies them as constants (`const zero = Low(...)`).
Anything else passes `id:`: `SegTree(op: f, id: e)`. A missing identity is
an error naming the constant to define. Lazy propagation takes an action
value `act: (f, x) => ...` with `compose:`. Const generics: `Mod<P>`,
`[n]T`, `Bitset<N>`.
Needs: `SegTree`, `LazySegTree`, `SparseTable`, `Fenwick`, `Matrix`, scans,
ordered containers (`Ord`), hashing (`Hash`). Status: partial (traits check;
laws, `T.zero` identities and `Self` at run time do not).

## W11. Reading and printing (P11)

`read<T>()` and `print(x)` go through `Read` and `Print`, derived for
records (W3) and defined by hand for anything else. Format options are
named arguments: `read<[n]Int>(base: 1)`, `print(xs, sep: "\n")`.
`read<Graph<V, E>>(directed: false, weighted: true)` is ordinary library
code. Status: partial (`read` is still compiler-special for some types).

## W12. Costs and library lints (P12)

Every public signature's first doc line states its cost: `/// O(log n).`
`cpl explain` and hover show it. Libraries may declare lints with a code,
message and codemod (`#lint(L7001, "...")`) and `#on_unimplemented("...")`
messages on traits. `assert` is debug-only. Status: doc convention only.

## W13. Modules, pins and codemods (P13)

`use std.graph.{bfs, Graph}`; prelude names need no `use`. A library may
mark a name deprecated with a replacement that `cpl fix` applies:
`#deprecated(use: "xs.sort(by: f)")`. Status: partial (codemods are
compiler-side).

## W14. Inline C++ (P14)

`@cpp("...")` bodies are allowed in any library, with the signature checked
at the boundary and the C++ included once in the bundle. Status: partial.

## W15. Precomputation (P15)

A top-level `let` is computed once before the first test (`let C =
Comb<Mod<P>>(N)`). Compile-time evaluation is not part of the spec:
contest tables are cheap at start-up, and baking them would grow the bundle.
Status: checks.

## W16. Contexts (P16): deferred

`with io(base: 1)` and friends wait for the program-shape design. Until
then a dream passes the option where it is used. Five dreams use contexts;
they are listed in `SYNTAX.md`.
