# The standard library: the complete design

Status: the design the stdlib lane builds toward. `plan/20-stdlib.md`
keeps the catalogue of what contests need and the reasoning behind the
v1 scope; this file is the surface: every module, every public item with
its signature and cost, the naming rules they follow, and the core
features each one relies on. `plan/DOMAINS.md` section 4 says where we
stand against it. The 0.3 spec chapters 20 to 29 and 40 were the seed;
where this file departs from them it says why, and `decisions.md` logs
the call.

Every item is marked:

- **[have]** exists in `stdlib/` today with tests;
- **[new]** designed here, not written yet;
- **[rename]** exists under another name, which keeps working as a
  deprecated alias until a codemod moves the ports;
- **[local]** lives in `examples/local-lib/` today and is promoted here.

## Who owns what

Ten area owners each own their part of section 4, write its design in
`design/stdlib/<area>.md` and build it; the coordinator owns sections 1
to 3 (principles, naming, core features), the prelude budget (4.1), `std.spec`
(4.15) and section 5, reviews every area design before it lands against the
checklist in `design/stdlib/README.md`, and routes each stdlib friction
in `research/frictions/` to the owner below. An item that spans two
areas belongs to the area of the type it returns; a new intrinsic or
core feature goes through section 3 and the core lane.

| Area | Owns | Handed over with its queue |
|---|---|---|
| sequences | 4.2 except the searches; the walk folds' library side; a `Seq`/`Indexed` trait | `runs`/`chunk_by` over any `Seq` incl. `Str` (`cf2084h`, `cf2069e`); `enumerate`; the `_by` renames (`argmax_by`, `min_by` to `by:`); the `Seq`/`Indexed` trait |
| DP/search | 4.2 searches (`design/stdlib/search.md`); `std.dp` | `cf2183h` onto `bsearch_first` + `aliens` (blocked: `Range<I128>` E0302, core lane); a gated port for Li Chao or Knuth when one appears |
| number theory/combinatorics | 4.3 to 4.7 (`math`, `mod`, `ntt`, `matrix`, `rand`, hashing) | `Comb<M>` keyed on the Mod type in place of `Comb<P>` (14 friction reports); `Mod<P>` in 32 bits via ranged-field narrowing (with codegen); exact-`Int` convolution, cyclic correlation, `Mod<P>`→`Int` (`cf2201e`); RNG `pick`/`perm`/`sample` |
| containers | 4.8, 4.9 (`bitset`, `bag`, `deque`, `heap`, `dsu`, `marks`, `window`), `List` methods | `Bits<D>` unifying `Bitset`/`Subset` (`cf2147h`); `List.truncate`/`extend` (`cf2084h`, `cf2178h`); offline divide and conquer over time, `offline_over` (`cf2069f`, `cf2222f`, `cf2237h`) |
| range structures | 4.10 | `LazySegTree` over `Bounds<D>` with constant init (`cf2207f`) |
| graphs | 4.11 `std.graph`, 4.15 `std.grid` | `design/stdlib/graphs.md`; `std.grid` stays a thin module over `(R, C)` product domains (decision 46.9); product/enum state domains wait on the core lane |
| trees | 4.11 `std.tree`, `std.tour` | link-cut tree (`cf2222f`, `cf2237h`) |
| flows/matching | 4.12 | `FlowGraph.reset()` (`cf2147h`); `max_matching`, `CostFlow` |
| strings | 4.13; O(1) slice and reverse views | `manacher`, `AhoCorasick`; one `lcp` (merge `lcp_between`/`RollingHash.lcp`, `lcp_with` to a named argument); O(1) slice/reverse views |
| geometry | 4.14 | `convex_hull`; `locate` returns an enum; `rot_by` to a named option |

## 1. Principles

The stdlib is ordinary CP Lang. A module may use only what a user library
may use: the public language, the documented intrinsics (section 3), and
other stdlib modules. If a module needs a power the language lacks, the
power becomes a core feature for everyone or the module waits.

What belongs here, in the end, is everything ever asked at the hardest
contests (ICPC World Finals, IOI, AtCoder WTF, Codeforces Div. 1), each
as fast as the best known C++ library implementation on a matched
benchmark. The order of arrival follows the ports: `examples/local-lib/`
is the staging area, written on top of this library the way a
contestant's own would be, and a module is promoted when it meets the
bar here (documented costs, tests, a port or example, the naming rules)
and the ports reach for it.

Each public item has a `///` comment saying what it does, its cost in
time and, where it matters, in memory. `cpl doc` prints them; the cost is
part of the contract, and a change that makes a cost worse is a breaking
change.

The obvious call is the correct and fast one. A function that looks O(1)
is O(1); anything O(n) or allocating says so in its name or its type
(`sorted` returns a new list, `sort` sorts in place).

Nothing is chosen secretly. `Bitset<D>` is a type you pick, never what a
`[D]Bool` silently becomes; `convolve` names NTT in its doc and its cost,
and a different algorithm is a different function.

## 2. Naming conventions

One name for one idea, everywhere. These rules decide every name below;
a name that breaks one is a bug in this file.

**Types are nouns in `UpperCamel`, functions and methods verbs or nouns
in `snake_case`.** A type's constructor is its name called like a
function: `Sieve(n)`, `Dsu<V>()`, `Rng(seed)`. `T.new(...)` is the
declaration form and the explicit spelling.

**A fold or sort of one sequence is a method.** `xs.sum()`, `xs.max()`,
`xs.count(p)`, `xs.all(p)`, `xs.first(p)`, `xs.sorted(by: k)`,
`xs.sort(by: k)`. The free form is for math: a generator
(`sum(f(x) for x in xs)`) or two or more values (`min(a, b)`). `find(x)`
is the index of a value; the element a test holds for is `first(p)`.
Lint L0509 (`free_fold`) rewrites the rest (decisions.md §53).

**Options are named arguments, not `_by` variants.** `runs(a, by: key)`,
`a.sort(by: key)`, `bsearch_first(r, ok)`. Today's `runs_by`,
`first_ge_by` and `last_le_by` are [rename]s to `runs(a, by:)`,
`first_ge(xs, x, by:)` and `last_le(xs, x, by:)`.

**`bsearch_first`/`bsearch_last` are the O(log n) monotone form of
`first`/`last`.** `first(xs, p)` scans: O(n) calls of any `p`.
`bsearch_first(r, ok)` asks O(log n) questions and needs `ok` false then
true along `r` (`bsearch_last`: true then false); both answer `?` a
position of the space searched. Reach for the scan unless the predicate
is monotone and the range is large.

**Fallible operations return `?T`; the trapping twin has the short
name.** `m[k]` traps when `k` is absent and `m.get(k)` is `?V`;
`x.inv` traps on a non-invertible residue and `x.try_inv()` is
`?Mod<P>`. A fallible twin is named `try_<verb>` unless a established
name exists (`get`, `pop`, `first`, `find`).

**Collection verbs.** Every container speaks the same verbs:

```
s.add(x)        // insert into a set-like container; returns whether it was new
s.remove(x)     // returns whether it was there (maps: the removed ?V)
s.toggle(x)     // flips membership
x in s          // membership; s.contains(x) is the method form
s.len           // size, a getter, never len()
s.is_empty
s.clear()       // O(1) or O(capacity), documented per type
s.take()        // moves the contents out and leaves s empty
```

Sequences `push` and `pop` at the back, `push_front` and `pop_front` at
the front; `first` and `last` read the ends as `?T`.

**Range structures.** `t.get(i)`, `t.set(i, x)`, `t.update(i, f)` for a
point; `t.fold(r)` for a range under the structure's monoid, `t.sum(r)`
where the operation is addition and nothing else makes sense
(`Fenwick`, `Prefix1`); `t.apply(r, f)` for a lazy action; `t.all` for
the fold of everything; `t.max_right(from, ok)` and `t.min_left(to, ok)`
for searches, returning boundaries.

**Sorted lookup.** One four-name family everywhere a sorted order exists
(sorted lists, `[D]T`, `Map`, `SortedBag`, `Compressed`):

```
first_ge(x)    // first element >= x
first_gt(x)    // first element >  x
last_le(x)     // last element  <= x
last_lt(x)     // last element  <  x
```

Sequences return the position, ordered collections the element. The 0.3
spec spells these `first(from: x)`, `first(after: x)`, `last(upto: x)`
and `last(before: x)`; we keep the four names because they read the
same in every container, are greppable, and need no overloading on
argument labels, which the core does not have (decision 26.1).

**Number-theory and algorithm names are the textbook names.**
`gcd`, `ext_gcd`, `crt`, `pow_mod`, `is_prime`, `factor`, `phi`,
`convolve`, `z_function`, `prefix_function`, `suffix_array`,
`manacher`, `dijkstra`, `scc`, `max_flow`, `convex_hull`. A contestant
types the name from the editorial.

**Costs in the name when two costs exist.** `factor(x)` is trial
division, O(sqrt x); `Sieve(n).factor(x)` is O(log x) after an O(n)
sieve; `factor_big(x)` is Pollard's rho for x up to 4e18. Each doc says
which to reach for.

**Costs are written one way.** A trailing `// O(...)` on the signature
here and the same in the `///` comment: time first, then memory when
it is not O(1) extra (`// O(n log n), O(n) memory`). Amortized and
expected bounds say so in words after the bound (`// O(alpha(n)) amortized`,
`// O(n) expected`); `n` is the structure's size unless the comment
names another variable.

**Index spaces.** A function over positions takes and returns the
domain (`?D`, `Bounds<D>`, `Range<D>`), never a raw `Int`, when its
input is a `[D]T`; over a `List<T>` it uses `Int`. A function that
creates a new index space returns it (`scc` returns `domain C`).

## 3. Core features the stdlib relies on

The stdlib needs no compiler knowledge beyond the following, each a
public feature user libraries get too.

Intrinsics (functions not written in the language): `popcount`, `ctz`,
`bit_width`, `bit_ceil`, `isqrt`, `sqrt`, `atan2`, `floor`, `trunc`,
`ceil`, `round`, `rand`, `flush`, `read`, `print`, `debug`, `sort`,
`sorted`, `swap`, `min`, `max`, `abs`, `sum`, `prod`, `count`, `all`,
`any`, `first`. The folds take a walk, `count(x > 2 for x in xs)`; the
four that fold a test (`count`, `all`, `any`, `first`) also take one
after the sequence, `count(xs, p)` and the method `xs.count(p)`, which
is the same walk with `p`'s body as its body (an inline lambda costs
nothing, a named function is one call per element). `sum`, `prod`,
`min` and `max` of a function stay `sum(f(x) for x in xs)`: one
spelling per idea (spec 22.2). The 0.3 spec adds
`mulhi`, `checked_*`, `wrapping_*`, `overflowing_*` and `clock`
[new, core lane].

Language features, with the modules that need them:

- generics with type parameters that are values (`Mod<P>` with a
  constant or a read `P`): `mod`, `ntt`, `matrix`, `spec`;
- operator traits (`Add`, `Sub`, `Mul`, `Div`, `Neg`, `Show`) and the
  associated constants `T.zero`, `T.one`: `mod`, `matrix`, `fenwick`,
  `prefix`, `geometry`;
- operation values (`Monoid`, `Action`, functions as values): `segtree`,
  `lazy`, `sparse`, `algebra`;
- domains, `Bounds<D>`, `Range<D>` and fresh domains returned from a
  function (existential returns): `graph`, `tree`, `seq.compress`,
  `strings.suffix_array` [0.3: fresh `Rank`];
- generators (`yields`): `bitset.each`, `graph.adj`, `tree.tour`,
  `seq.windows`;
- interior mutability in a `let` value (`Comb`), `lends mut`
  projections (`matrix.at`), `@get` getters;
- `@test` functions run by `cpl test stdlib`, and `laws_hold` for
  algebraic laws.

Still missing from the core and wanted here, owned by the core lane:
variadic `print`/`read` for the `Read`/`Print` derives, a niche trait so
user optionals are 8 bytes, writable places through methods
(`seg.at(i) += 1`), and overloading on argument labels (would let the
four-name family become `first(after:)` if we ever want it).

## 4. The modules

Each module below opens with what it is for, then its full public
surface as declared (the [have] block is generated from `stdlib/*.cp`
by reading each `pub` line and the `O(...)` phrases in its `///`
comment, so it matches the source at the commit that last touched this
file), then the [new] and [rename] items it still owes.

### 4.1 `std.prelude`

The names every program sees without a `use`: the 80-name budget is a
test (`prelude_stays_under_budget`). A name earns a slot when ports
reach for it in more than one file in ten. Everything else is one `use`
away, and the compiler's missing-name hint names the module (it is read
off the files by `build.rs`, so a new `pub fn` needs no second edit).

### 4.2 `std.seq`: sequences, sorted lookup, search on answers

Free functions over `List<T>`, `[D]T`, and `Str`, the four-name sorted-lookup
family, prefix/suffix/exclusive scans, permutations, monotone-stack and
sliding-window helpers, coordinate compression, and binary/ternary searches.
Full design and contest survey in `design/stdlib/seq.md`. The searches return
the boundary, not a found flag: `bsearch_first(lo..hi, ok)` is the first `i`
with `ok(i)` for a monotone `ok`, or `none`.

```cp
type Perm<V> = [V]V
fn scan<D, T>(xs: [D]T, f: (T, T) -> T) -> [D]T // O(n)
fn bsearch_first<I>(r: Range<I>, ok: I -> Bool) -> ?I // O(log n)
fn bsearch_last<I>(r: Range<I>, ok: I -> Bool) -> ?I // O(log n)
fn min_by<S, T, K>(it: S, key: T -> K) -> ?T
fn max_by<S, T, K>(it: S, key: T -> K) -> ?T
fn reverse<T>(mut a: List<T>)
fn dedup<T>(mut a: List<T>)
fn deduped<T>(self: List<T>) -> List<T>
fn partition<T>(self: List<T>, keep: T -> Bool) -> (List<T>, List<T>)
fn windows<T>(self: List<T>, k: Int) -> yields List<T> // O(k)
fn pairs<T>(self: List<T>) -> yields (T, T)
fn with_prev<T>(self: List<T>) -> yields (?T, T)
fn cyclic_pairs<T>(self: List<T>) -> yields (T, T)
fn zip<A, B>(a: List<A>, b: List<B>) -> List<(A, B)>
fn get_or_insert<K, V>(mut self: Map<K, V>, k: K, v: V) -> V // O(log n)
// O(log n)
fn get_or_insert_with<K, V>(mut self: Map<K, V>, k: K, f: fn() -> V) -> V
fn update<K, V>(mut self: Map<K, V>, k: K, init: V, f: fn(V) -> V) // O(log n)
fn rotate_left<T>(mut a: List<T>, k: Int)
fn next_permutation<T>(mut a: List<T>) -> Bool // O(n), O(1)
fn prev_permutation<T>(mut a: List<T>) -> Bool // O(n), O(1)
fn runs<T>(a: List<T>) -> List<(val: T, len: Int)>
fn runs_by<T, K>(a: List<T>, by: T -> K) -> List<(key: K, items: List<T>)>
fn excl_fold<T, M>(self: List<T>, m: M) -> yields (x: T, rest: T) // O(n)
struct Compressed<D, T>(vals: [D]T)
    fn get<K>(self, i: K) // O(1), O(r.len)
    fn slice(self, r: Range<D>) -> List<T> // O(r.len)
    @get fn len(self) -> Int
    fn at(self, val: T) -> D // O(log n)
    fn first_ge(self, val: T) -> ?D // O(log n)
    fn last_le(self, val: T) -> ?D // O(log n)
fn compress<T>(all: List<T>) // O(n log n)
@get fn l<D>(self: Range<D>) -> Bounds<D>
@get fn r<D>(self: Range<D>) -> Bounds<D>
@get fn ends<D>(self: Range<D>) -> (Int, Int)
fn first_ge_by<T, K>(xs: List<T>, target: K, by: T -> K) -> ?Int // O(log n)
fn last_le_by<T, K>(xs: List<T>, target: K, by: T -> K) -> ?Int // O(log n)
fn swap_at<D, T>(mut self: [D]T, i: D, j: D)
fn resized<T>(self: List<T>, n: Int, fill: T) -> List<T> // O(n)
fn take<T>(mut self: List<T>) -> List<T> // O(1)
fn insert<T>(mut self: List<T>, i: Int, x: T) // O(n)
fn remove<T>(mut self: List<T>, i: Int) -> T // O(n)
@get fn argmax<D, T>(self: [D]T) -> ?D // O(n)
@get fn argmin<D, T>(self: [D]T) -> ?D // O(n)
```

[new] Designed in `design/stdlib/seq.md`:

```cp
// Reductions and predicates
fn prod<S, T>(it: S) -> T // O(n)
fn is_sorted<D, T>(self: [D]T) -> Bool // O(n)
fn is_sorted_by<D, T, K>(self: [D]T, by: T -> K) -> Bool // O(n)
fn argmax_by<D, T, K>(self: [D]T, by: T -> K) -> ?D // O(n)
fn argmin_by<D, T, K>(self: [D]T, by: T -> K) -> ?D // O(n)

// Selection, merging, and O(1)-space rotation
fn nth_element<T>(mut a: List<T>, k: Int) // expected O(n), O(1) space
fn merge_sorted<T>(a: List<T>, b: List<T>) -> List<T> // O(n + m)
fn rotate_right<T>(mut a: List<T>, k: Int) // O(n), O(1) space

// Zero-allocation runs/chunks and domain bucketing
fn chunk_by<D, T>(self: [D]T, same: (T, T) -> Bool) -> yields Range<D> // O(n), O(1) space
fn run_ranges<D, T>(self: [D]T) -> yields Range<D> // O(n), O(1) space
fn run_ranges_by<D, T, K>(self: [D]T, by: T -> K) -> yields Range<D> // O(n), O(1) space
fn buckets<D, K>(key: [D]K) -> [K]List<D> // O(D.count + K.count)
fn chunks<T>(self: List<T>, k: Int) -> yields List<T> // O(n)

// Prefix and suffix scans over elements (cut-indexed prefix_fold/suffix_fold live in std.prefix)
fn suffix_scan<D, T>(xs: [D]T, f: (T, T) -> T) -> [D]T // O(n)

// Coordinate compression completeness
fn Compressed.contains(self, val: T) -> Bool // O(log n)
fn Compressed.try_at(self, val: T) -> ?D // O(log n)
fn Compressed.first_gt(self, val: T) -> ?D // O(log n)
fn Compressed.last_lt(self, val: T) -> ?D // O(log n)

// Permutations
fn inverse<I, V>(p: [I]V) -> [V]I // O(n)
fn compose<A, B, C>(f: [A]B, g: [B]C) -> [A]C // O(n)
fn perm_cycles<V>(p: [V]V) -> List<List<V>> // O(n)
fn perm_pow<V>(p: [V]V, k: Int) -> [V]V // O(n)
fn inversions<T>(a: List<T>) -> Int // O(n log n)

// Binary/ternary search and four-name sorted lookup
// (search on answers: design/stdlib/dp.md; each returns a position of its
// range's index space or none, never a sentinel)
fn bsearch_real(lo: F64, hi: F64, ok: F64 -> Bool, steps: Int = 100) -> ?F64
    // steps + 1 calls of ok; the least x in [lo, hi] with ok(x), to within
    // (hi - lo) / 2^steps; none when not ok(hi)
fn ternary_max<I, K>(r: Range<I>, f: I -> K) -> ?I // O(log n) calls of f
fn ternary_min<I, K>(r: Range<I>, f: I -> K) -> ?I // O(log n) calls of f
    // a position of the optimum of f on r, for f strictly rising then
    // falling (plateaus only at the optimum); none for an empty r
fn ternary_max_real(lo: F64, hi: F64, f: F64 -> F64, steps: Int = 100) -> F64
fn ternary_min_real(lo: F64, hi: F64, f: F64 -> F64, steps: Int = 100) -> F64
    // steps + 2 calls of f (golden section); within (hi - lo) * 0.618^steps
fn parallel_bsearch<I>(queries: Int, r: Range<I>, reset: () -> (),
                       apply: I -> (), ok: Int -> Bool) -> List<?I>
    // O((r.len + queries) log r.len) calls; out[q] is the first t with
    // ok(q) after apply on every t' <= t, ok monotone in the prefix
fn first_gt<D, T>(self: [D]T, target: T) -> ?D // O(log n)
fn first_gt_by<D, T, K>(self: [D]T, target: K, by: T -> K) -> ?D // O(log n)
fn last_lt<D, T>(self: [D]T, target: T) -> ?D // O(log n)
fn last_lt_by<D, T, K>(self: [D]T, target: K, by: T -> K) -> ?D // O(log n)

// Monotone stack and fixed-window sliding extrema
fn prev_less<D, T>(self: [D]T, strict: Bool = true) -> [D]?D // O(n)
fn next_less<D, T>(self: [D]T, strict: Bool = true) -> [D]?D // O(n)
fn prev_greater<D, T>(self: [D]T, strict: Bool = true) -> [D]?D // O(n)
fn next_greater<D, T>(self: [D]T, strict: Bool = true) -> [D]?D // O(n)
fn sliding_min<D, T>(self: [D]T, k: Int) -> List<T> // O(n)
fn sliding_max<D, T>(self: [D]T, k: Int) -> List<T> // O(n)
```
```

[rename] `runs_by(a, key)` is superseded in hot paths by zero-allocation
`a.run_ranges(by: key)` (and ultimately `runs(a, by: key)`);
`first_ge_by`, `last_le_by`, `argmax_by`, `argmin_by` collapse into
`first_ge(.., by:)`, `last_le(.., by:)`, `argmax(by:)`, `argmin(by:)` once
builtins and getters accept optional `by:`.

### 4.3 `std.math`: number theory

Integer helpers and number theory with every cost in the name or the
type. Full design and contest survey: `design/stdlib/math.md`. Three
ways to factor, from cheapest setup to largest input:
`Sieve(n).factor(x)` for many x up to n, `factor(x)` by trial division
for one x up to about 1e12, `factor_big(x)` by Pollard's rho for x up
to 4e18.

```cp
fn gcd(a: Int, b: Int) -> Int // O(log min(a, b))
fn lcm(a: Int, b: Int) -> Int
fn fixed(num: Int, den: Int, k: Int) -> Str
fn choose(n: Int, k: Int) -> Int // O(min(k, n - k))
fn ext_gcd(a: Int, b: Int) -> (g: Int, x: Int, y: Int) // O(log min(a, b))
fn ceil_div(a: Int, b: Int) -> Int
struct Sieve // O(n)
    fn new(n: Int) -> Sieve
    fn is_prime(self, x: Int) -> Bool
    fn factor(self, x: Int) -> List<(Int, Int)> // O(log x)
    fn phi(self, x: Int) -> Int // O(log x)
    fn mobius(self, x: Int) -> Int // O(log x)
fn bit_len(x: Int) -> Int
fn digits(self: Int, base: Int = 10) -> List<Int>
@get fn sign(self: Int) -> Int
fn bit(self: Int, i: Int) -> Bool
@get fn is_pow2(self: Int) -> Bool
fn splitmix64(x: U64) -> U64
fn factor(x: Int) -> List<(Int, Int)> // O(sqrt x)
fn divisors(x: Int) -> List<Int> // O(sqrt x)
fn crt(r1: Int, m1: Int, r2: Int, m2: Int) -> ?(Int, Int)
fn mul_mod(a: Int, b: Int, m: Int) -> Int // O(1), through I128 if needed
fn pow_mod(a: Int, e: Int, m: Int) -> Int // O(log e); 1 <= m
fn is_prime(x: Int) -> Bool // O(log^2 x); deterministic Miller-Rabin, every Int
fn factor_big(x: Int) -> List<(Int, Int)> // expected O(x^(1/4) log x)
fn phi(x: Int) -> Int // O(sqrt x)
fn primitive_root(p: Int) -> Int // expected O(p^(1/4) log p), prime p
fn floor_sum(n: Int, m: Int, a: Int, b: Int) -> Int
    // O(log m); sum of (a*i + b) / m for i in 0..n, floor division
```

[new] Additions from `design/stdlib/math.md`:

```cp
fn choose2(n: Int) -> Int // O(1); n * (n - 1) / 2, 0 for n < 2
fn inv_mod(a: Int, m: Int) -> ?Int // O(log m); m >= 1, none if gcd(a, m) != 1
fn crt_all(eqs: List<(r: Int, m: Int)>) -> ?(r: Int, m: Int) // O(k log M)
fn mod_log(a: Int, b: Int, m: Int) -> ?Int // O(sqrt m); BSGS discrete log
fn mod_sqrt(a: Int, p: Int) -> ?Int // O(log^2 p); Tonelli-Shanks, prime p
fn divisors_from_factors(fs: List<(Int, Int)>) -> List<Int> // O(d(x) log d(x))
fn Sieve.divisors(self, x: Int) -> List<Int> // O(d(x) log d(x)), sorted
fn Sieve.phi_table(self) -> List<Int> // O(n)
fn Sieve.mobius_table(self) -> List<Int> // O(n)
fn sieve_multiplicative<T>(
    n: Int,
    one: T,
    mul: (T, T) -> T,
    prime_pow: (Int, Int, Int) -> T,
) -> List<T> // O(n) time and memory
```

Relies on: `I128` (inside `mul_mod`), the builtin `isqrt`.

### 4.4 `std.mod`: modular arithmetic

`Mod<P>` is an integer reduced mod `P`, where `P` is a constant or a
value read at startup; `Comb<M>` is keyed on the `Mod` type (`type Z = Mod<P>;
let comb = Comb<Z>(limit)`) so the modulus is written once and factorial tables
grow on demand (decision 52).

```cp
struct Mod<P>(v: 0..P) // O(1), O(log P)
    fn new(x: Int) -> Mod<P>
    fn pow(self, e: Int) -> Mod<P> // O(log e)
    fn try_inv(self) -> ?Mod<P> // O(log P)
    @get fn inv(self) -> Mod<P>
struct Comb<M> // O(1)
    fn new(limit: Int = 0) -> Comb<M>
    fn fact(self, n: Int) -> M
    fn inv_fact(self, n: Int) -> M
    fn inv(self, i: Int) -> M // O(1)
    fn choose(self, n: Int, k: Int) -> M
    fn arrange(self, n: Int, k: Int) -> M
    fn multichoose(self, n: Int, k: Int) -> M
    fn multinomial(self, ks: List<Int>) -> M // O(ks.len)
    fn catalan(self, n: Int) -> M
```

### 4.5 `std.ntt` [new]: convolution

The number-theoretic transform, polynomial products (linear and cyclic),
and bitwise/SOS convolutions, promoted from
`examples/local-lib/math/poly.cp` (imported as `lib.math.poly` by nine
port files) and `examples/local-lib/math/sos.cp`. Full design:
`design/stdlib/math.md`. The formal power-series operations stay in the
local library until wider corpus demand justifies promotion.

```cp
fn ntt<P>(mut a: List<Mod<P>>, inverse: Bool = false)
    // O(n log n) in place; a.len a power of two dividing P - 1
fn convolve<P>(a: List<Mod<P>>, b: List<Mod<P>>, cyclic: Bool = false) -> List<Mod<P>>
    // O((n + m) log(n + m)); P - 1 divisible by a power of two at least
    // n + m - 1 (998244353, 167772161, 469762049, 754974721 qualify);
    // schoolbook below 64 terms
fn convolve_mod(a: List<Int>, b: List<Int>, m: Int, cyclic: Bool = false) -> List<Int>
    // three NTT primes and Garner CRT; any 1 <= m < 2^31
fn convolve_int(a: List<Int>, b: List<Int>, cyclic: Bool = false) -> List<Int>
    // exact signed integer convolution when every output fits in about +-2^62
enum SosOver = Sub | Sup
fn sos<T>(mut a: List<T>, over: SosOver = Sub, inverse: Bool = false)
    // O(n log n); subset (Sub) or superset (Sup) zeta/Mobius transform in place
fn convolve_or<T>(a: List<T>, b: List<T>) -> List<T> // O(n log n)
fn convolve_and<T>(a: List<T>, b: List<T>) -> List<T> // O(n log n)
fn convolve_xor<T>(a: List<T>, b: List<T>) -> List<T> // O(n log n); requires / 2
```

Relies on: `Mod<P>` with a type-parameter modulus, `primitive_root`,
`bit_ceil`, `ctz`.

### 4.6 `std.matrix` [new]: matrices, Gaussian elimination and linear basis

`Mat<T>` is a dense flat row-major matrix over any `T` with `T.zero`,
`T.one`, `+`, `-`, `*` (`Int`, `Mod<P>`, `F64`). Exact field elimination
(`det`, `rank`, `solve`, `solve_space`, `inverse`) lives on `Mat<T>` for
exact fields such as `Mod<P>` with prime `P`; partial-pivoted `F64`
helpers (`det_real`, `rank_real`, `solve_real`, `inverse_real`) take a
named tolerance `eps: F64 = 1e-9`. `XorBasis` covers GF(2) linear bases
with a `Monoid` for segment trees. Full design: `design/stdlib/math.md`.

```cp
struct Mat<T>(n: Int, m: Int, a: List<T>)
    fn zero(n: Int, m: Int = n) -> Mat<T>          // O(n m)
    fn identity(n: Int) -> Mat<T>                  // O(n^2)
    fn of(rows: List<List<T>>) -> Mat<T>           // O(n m)
    fn get(self, i: Int, j: Int) -> T              // O(1)
    fn at(self, i: Int, j: Int) lends mut T        // O(1)
    fn pow(self, e: Int) -> Mat<T>                 // O(n^3 log e)
    fn apply(self, v: List<T>) -> List<T>          // O(n m)
    fn transpose(self) -> Mat<T>                   // O(n m)
    fn rows(self) -> List<List<T>>                 // O(n m)
    fn det(self) -> T                              // O(n^3), T: Div
    fn rank(self) -> Int                           // O(n m min(n, m))
    fn solve(self, b: List<T>) -> ?List<T>         // O(n m min(n, m))
    fn solve_space(self, b: List<T>) -> ?(x0: List<T>, basis: List<List<T>>)
    fn inverse(self) -> ?Mat<T>                    // O(n^3)
    fn product(n: Int) -> Monoid<Mat<T>>           // for segment trees
fn det_real(a: Mat<F64>, eps: F64 = 1e-9) -> F64   // O(n^3)
fn rank_real(a: Mat<F64>, eps: F64 = 1e-9) -> Int  // O(n m min(n, m))
fn solve_real(a: Mat<F64>, b: List<F64>, eps: F64 = 1e-9) -> ?List<F64>
fn inverse_real(a: Mat<F64>, eps: F64 = 1e-9) -> ?Mat<F64>
struct XorBasis
    fn new() -> XorBasis                           // O(1)
    @get fn rank(self) -> Int                      // O(1)
    fn add(mut self, x: Int) -> Bool               // O(log x)
    fn reduce(self, x: Int) -> Int                 // O(62)
    fn contains(self, x: Int) -> Bool              // O(62)
    fn max_xor(self, init: Int = 0) -> Int         // O(62)
    fn min_xor(self, init: Int) -> Int             // O(62)
    fn kth(self, k: Int) -> ?Int                   // O(62^2)
    fn merge(mut self, o: XorBasis)                // O(rank * 62)
    fn monoid() -> Monoid<XorBasis>                // for segment trees
```

Relies on: operator traits, `T.zero` and `T.one`, `lends mut`.

### 4.7 `std.rand` and hashing

A seeded SplitMix64 stream: deterministic, so a failing stress case
replays. Hashing of values is the language's (`Hash` on every key
type); string hashing is `std.strings.RollingHash`.

```cp
struct Rng(s: U64) // O(1)
    fn new(seed: Int) -> Rng
    fn next(mut self) -> U64
    fn int(mut self, lo: Int, hi: Int) -> Int
    fn float(mut self) -> F64
    fn shuffle<T>(mut self, mut a: List<T>) // O(n)
```

[new]:

```cp
fn pick<T>(mut self, xs: List<T>) -> T           // O(1); traps on empty
fn chance(mut self, p: F64) -> Bool              // O(1)
fn perm(mut self, n: Int) -> List<Int>           // O(n), uniform
fn sample(mut self, n: Int, k: Int) -> List<Int> // O(k log k), distinct,
                                                 // sorted, from 0..n
fn hash_mix(h: U64, x: U64) -> U64               // O(1), splitmix finalizer
                                                 // over h ^ x; for keyed hashes
```

### 4.8 `std.bitset`: bitsets

`Bitset<D>` packs a set of positions of `D` into 64-bit words, the type
for O(n/64) set algebra and shift DP. It is chosen by name; a `[D]Bool`
is never silently packed. The 0.3 spec's open name `Bits<D>` would
clash with the existing `Bits` trait for integer widths, so `Bitset`
stays (decision 26.3).

```cp
struct Bitset<D> // O(1), O(K / 64)
    fn new() -> Bitset<D>
    fn full() -> Bitset<D>
    fn from(a: [D]Bool) -> Bitset<D>
    fn contains(self, i: D) -> Bool
    fn add(mut self, i: D)
    fn remove(mut self, i: D)
    fn toggle(mut self, i: D)
    @get fn len(self) -> Int
    @get fn count(self) -> Int
    @get fn is_empty(self) -> Bool
    fn union(mut self, o: Bitset<D>)
    fn intersect(mut self, o: Bitset<D>)
    fn subtract(mut self, o: Bitset<D>)
    fn symmetric(mut self, o: Bitset<D>)
    fn bitor(self, o: Bitset<D>) -> Bitset<D>
    fn bitand(self, o: Bitset<D>) -> Bitset<D>
    fn bitxor(self, o: Bitset<D>) -> Bitset<D>
    fn meets(self, o: Bitset<D>) -> Bool
    fn shifted(self, k: Int) -> Bitset<D>
    fn each(self) -> yields D
    fn next(self, i: D) -> ?D
```

[new] (see `design/stdlib/containers.md`):

```cp
fn with(self, i: D) -> Bitset<D>      // O(K / 64), non-mutating copy
fn without(self, i: D) -> Bitset<D>   // O(K / 64), non-mutating copy
fn clear(mut self)                    // O(K / 64)
@get fn first(self) -> ?D             // O(K / 64)
@get fn last(self) -> ?D              // O(K / 64)
fn prev(self, i: D) -> ?D             // O(K / 64); last member < i
fn count_below(self, i: D) -> Int     // O(K / 64); count < i
fn shift_up(mut self, k: Int)         // O(K / 64) in place, k >= 0
fn shift_or(mut self, k: Int)         // O(K / 64) in place: self |= self << k
fn find(self, pred: D -> Bool) -> ?D  // O(K / 64 + visited)
fn subset_sums(weights: List<Int>) -> Bitset<D> // O(K sqrt(S) / 64) (associated on Bitset<D>)
```

### 4.9 Containers: `std.bag`, `std.interval`, `std.deque`, `std.heap`, `std.dsu`, `std.marks`, `std.hashmap`

See `design/stdlib/containers.md` for the full pattern survey, laws, and
worked contest examples. `SortedBag<K>` is an ordered multiset with order
statistics, monotone search, recycled treap nodes, and O(1)-amortized
cursors (`BagCursor`, `entries`, `entries_rev`); `OrderedSet<K>` is its
unique-key twin; `IntervalMap<D, T>` (`std.interval`) maintains a maximal-run
partition of domain `D` under range assignment; `Deque<T>`, `MonoDeque<T>`,
and `TwoStackFold<T>` cover ring buffers and sliding-window queries;
`MinHeap`/`MaxHeap`, `ErasableMinHeap`/`ErasableMaxHeap`, and
`IndexedMinHeap`/`IndexedMaxHeap` cover binary, lazy-delete, and domain-keyed
heaps; `Dsu<V>`, `DsuOf<V, T>`, `ParityDsu<V>`, `PotentialDsu<V>`, and
`RollbackArray<D, T>` share nominal `Mark` rollback; `Marks<D>`,
`MarkMap<D, T>`, and `Interner<K>` cover O(1)-clear epoch sets/maps and
online interning; `FlatMap<V>` (`std.hashmap`) provides open-addressing
salted hashing for `Int` keys.

The union-finds and `RollbackArray` step back in time the same way: `mark()`
returns a `Mark`, its own type, and `undo_to(mark)` undoes every change since,
values and parities included, O(1) per change. One mark serves several
returns to the same point (`cf2222f` undoes both halves of its divide and
conquer to one `snap`). `DsuOf`'s join is `join(value of a's set, value
of b's set)` for `unite(a, b)` whatever root union by size keeps, so
`(_, b) => b` is a directional "b wins" (`cf2237h`, skip-to-previous
bucket) and `(a, b) => a ?? b` keeps the first representative
(`cf2222f`). Scoped rollback, so an early `return` cannot skip
`undo_to`, belongs to the core's proposed `defer`/`undo` block (`plan/14`);
a closure helper would need the body to mutate the structure while it is
lent, which lexical borrowing rejects. Frictions:
`research/frictions/cf2222f.md`, `research/frictions/cf2237h.md`.

```cp
struct BagCursor(id: Int)
struct SortedBag<K> // O(log n), O(1), O(n)
    fn new() -> SortedBag<K>
    fn of(xs: List<K>) -> SortedBag<K> // O(n log n)
    @get fn len(self) -> Int // O(1)
    @get fn distinct(self) -> Int // O(1) [new]
    @get fn is_empty(self) -> Bool
    fn clear(mut self)
    fn take(mut self) -> SortedBag<K> // O(1)
    fn add(mut self, x: K) -> Bool // O(log n)
    fn add_many(mut self, x: K, c: Int) // O(log n) [new]
    fn remove(mut self, x: K) -> Bool // O(log n)
    fn remove_all(mut self, x: K) -> Int // O(log n) [new]
    fn count(self, x: K) -> Int // O(log n)
    fn contains(self, x: K) -> Bool // O(log n)
    @get fn first(self) -> ?K // O(log n)
    @get fn last(self) -> ?K // O(log n)
    fn pop_first(mut self) -> ?K // O(log n)
    fn pop_last(mut self) -> ?K // O(log n)
    fn first_ge(self, k: K) -> ?K // O(log n)
    fn first_gt(self, k: K) -> ?K // O(log n)
    fn last_le(self, k: K) -> ?K // O(log n)
    fn last_lt(self, k: K) -> ?K // O(log n)
    fn rank(self, k: K) -> Int // O(log n)
    fn count_range(self, lo: K, hi: K) -> Int // O(log n) [new]
    fn nth(self, i: Int) -> ?K // O(log n)
    fn first_where(self, ok: K -> Bool) -> ?K // O(log n) [new]
    fn cursor_first(self) -> ?BagCursor // O(log n) [new]
    fn cursor_last(self) -> ?BagCursor // O(log n) [new]
    fn cursor_at(self, k: K) -> ?BagCursor // O(log n) [new]
    fn cursor_ge(self, k: K) -> ?BagCursor // O(log n) [new]
    fn cursor_gt(self, k: K) -> ?BagCursor // O(log n) [new]
    fn cursor_le(self, k: K) -> ?BagCursor // O(log n) [new]
    fn cursor_lt(self, k: K) -> ?BagCursor // O(log n) [new]
    fn key_of(self, c: BagCursor) -> K // O(1) [new]
    fn count_of(self, c: BagCursor) -> Int // O(1) [new]
    fn next_cursor(self, c: BagCursor) -> ?BagCursor // O(1) amort [new]
    fn prev_cursor(self, c: BagCursor) -> ?BagCursor // O(1) amort [new]
    fn entries(self) -> yields (key: K, count: Int) // O(n) [new]
    fn entries_rev(self) -> yields (key: K, count: Int) // O(n) [new]
    fn to_list(self) -> List<K> // O(n)
struct OrderedSet<K> // [new] unique-key twin of SortedBag<K>
    fn new() -> OrderedSet<K>
    fn of(xs: List<K>) -> OrderedSet<K> // O(n log n)
    @get fn len(self) -> Int // O(1)
    @get fn is_empty(self) -> Bool
    fn clear(mut self)
    fn take(mut self) -> OrderedSet<K> // O(1)
    fn add(mut self, x: K) -> Bool // O(log n)
    fn remove(mut self, x: K) -> Bool // O(log n)
    fn contains(self, x: K) -> Bool // O(log n)
    @get fn first(self) -> ?K // O(log n)
    @get fn last(self) -> ?K // O(log n)
    fn pop_first(mut self) -> ?K // O(log n)
    fn pop_last(mut self) -> ?K // O(log n)
    fn first_ge(self, k: K) -> ?K // O(log n)
    fn first_gt(self, k: K) -> ?K // O(log n)
    fn last_le(self, k: K) -> ?K // O(log n)
    fn last_lt(self, k: K) -> ?K // O(log n)
    fn rank(self, k: K) -> Int // O(log n)
    fn count_range(self, lo: K, hi: K) -> Int // O(log n)
    fn nth(self, i: Int) -> ?K // O(log n)
    fn first_where(self, ok: K -> Bool) -> ?K // O(log n)
    fn each(self) -> yields K // O(n)
    fn each_rev(self) -> yields K // O(n)
    fn to_list(self) -> List<K> // O(n)
struct IntervalMap<D, T> // [new] std.interval; O(log runs) amortized
    fn new(init: T) -> IntervalMap<D, T> // O(1)
    fn from(a: [D]T) -> IntervalMap<D, T> // O(D.K)
    fn at(self, i: D) -> T // O(log runs)
    fn run_at(self, i: D) -> (range: Range<D>, val: T) // O(log runs)
    fn assign(mut self, r: Range<D>, val: T) -> List<(range: Range<D>, val: T)>
    fn assign_point(mut self, i: D, val: T) -> T // O(log runs)
    fn each(self) -> yields (range: Range<D>, val: T) // O(runs)
    @get fn runs(self) -> Int // O(1)
struct Deque<T> // O(1)
    fn new() -> Deque<T>
    fn of(xs: List<T>) -> Deque<T> // O(n)
    @get fn len(self) -> Int
    @get fn is_empty(self) -> Bool
    fn clear(mut self)
    fn take(mut self) -> Deque<T> // O(1)
    fn push(mut self, x: T)
    fn push_front(mut self, x: T)
    fn pop(mut self) -> ?T
    fn pop_front(mut self) -> ?T
    @get fn first(self) -> ?T
    @get fn last(self, k: Int
    @get fn items(self) -> List<(Int, T)> // O(n)
    fn get(self, i: Int) -> T
    fn set(mut self, i: Int, x: T) // O(1) [new]
    fn to_list(self) -> List<T> // O(n) [new]
struct MonoDeque<T> // O(1) amortized sliding min/max (std.window)
    fn new(by: (T, T) -> T = min) -> MonoDeque<T> // O(1)
    @get fn len(self) -> Int // O(1)
    @get fn is_empty(self) -> Bool // O(1)
    fn clear(mut self) // O(1)
    fn push(mut self, at: Int, val: T) // O(1) amortized
    fn drop_before(mut self, min_at: Int) // O(1) amortized
    fn drop_after(mut self, max_at: Int) // O(1) amortized
    @get fn best(self) -> ?T // O(1)
    @get fn best_at(self) -> ?Int // O(1)
struct TwoStackFold<T> // [new] O(1) amortized queue monoid fold
    fn new<M>(op: M) -> TwoStackFold<T>
    @get fn len(self) -> Int
    @get fn is_empty(self) -> Bool
    fn clear(mut self)
    fn push(mut self, x: T)
    fn pop(mut self) -> ?T
    @get fn fold(self) -> T
struct MinHeap<T> // O(log n), O(1)
    fn new() -> MinHeap<T>
    fn of(xs: List<T>) -> MinHeap<T> // O(n) [new]
    @get fn len(self) -> Int
    @get fn is_empty(self) -> Bool
    @get fn peek(self) -> ?T
    fn clear(mut self)
    fn take(mut self) -> MinHeap<T> // O(1)
    fn push(mut self, x: T)
    fn pop(mut self) -> ?T
struct MaxHeap<T>
    fn new() -> MaxHeap<T>
    fn of(xs: List<T>) -> MaxHeap<T> // O(n) [new]
    @get fn len(self) -> Int
    @get fn is_empty(self) -> Bool
    @get fn peek(self) -> ?T
    fn clear(mut self)
    fn take(mut self) -> MaxHeap<T> // O(1)
    fn push(mut self, x: T)
    fn pop(mut self) -> ?T
struct ErasableMinHeap<T> // [new] O(log n) amortized lazy-delete heap
    fn new() -> ErasableMinHeap<T>
    @get fn len(self) -> Int
    @get fn is_empty(self) -> Bool
    fn clear(mut self)
    fn take(mut self) -> ErasableMinHeap<T> // O(1)
    fn push(mut self, x: T)
    fn remove(mut self, x: T)
    fn peek(mut self) -> ?T
    fn pop(mut self) -> ?T
struct ErasableMaxHeap<T> // [new] largest-first twin of ErasableMinHeap
struct IndexedMinHeap<D, T> // [new] O(log n) domain-keyed decrease/update-key
    fn new() -> IndexedMinHeap<D, T> // O(D.K)
    @get fn len(self) -> Int
    @get fn is_empty(self) -> Bool
    fn clear(mut self)
    fn contains(self, k: D) -> Bool // O(1)
    fn get(self, k: D) -> ?T // O(1)
    fn set(mut self, k: D, val: T) // O(log n)
    fn remove(mut self, k: D) -> ?T // O(log n)
    @get fn peek(self) -> ?(key: D, val: T) // O(1)
    fn pop(mut self) -> ?(key: D, val: T) // O(log n)
struct IndexedMaxHeap<D, T> // [new] largest-first twin of IndexedMinHeap
struct Mark(at: Int)
struct Dsu<V> // O(alpha(n)), O(log n)
    fn new() -> Dsu<V> // O(n)
    fn find(mut self, v: V) -> V
    fn mark(mut self) -> Mark // O(1)
    fn undo_to(mut self, snap: Mark) // O(1) per undone unite
    fn unite(mut self, a: V, b: V) -> Bool
    fn merge(mut self, a: V, b: V) -> ?(kept: V, merged: V)
    fn same(mut self, a: V, b: V) -> Bool
    fn size(mut self, v: V) -> Int
    @get fn components(self) -> Int
    fn groups(mut self) -> List<List<V>> // O(n alpha(n))
struct DsuOf<V, T> // as Dsu, plus one join per merge; three [V] arrays
    fn new(init: (V) -> T, join: (T, T) -> T) -> DsuOf<V, T> // O(n)
    fn find(mut self, v: V) -> V
    fn get(mut self, v: V) -> T
    fn set(mut self, v: V, x: T)
    fn unite(mut self, a: V, b: V) -> Bool
    fn merge(mut self, a: V, b: V) -> ?(kept: V, merged: V)
    fn same(mut self, a: V, b: V) -> Bool
    fn size(mut self, v: V) -> Int
    @get fn components(self) -> Int
    fn groups(mut self) -> List<List<V>> // O(n alpha(n)) [new]
    fn mark(mut self) -> Mark // O(1)
    fn undo_to(mut self, snap: Mark) // O(1) per undone merge or set
struct ParityDsu<V> // O(log n), no path shortening; three [V] arrays
    fn new() -> ParityDsu<V> // O(n)
    fn find(self, v: V) -> (root: V, flip: Bool)
    fn unite(mut self, a: V, b: V, differ: Bool = true) -> Bool // false: contradiction
    fn diff(self, a: V, b: V) -> ?Bool
    fn same(self, a: V, b: V) -> Bool
    fn size(self, v: V) -> Int
    @get fn consistent(self) -> Bool
    fn mark(self) -> Mark // O(1)
    fn undo_to(mut self, snap: Mark) // O(1) per undone unite
struct PotentialDsu<V> // [new] O(alpha(n)) / O(log n) after mark
    fn new() -> PotentialDsu<V> // O(n)
    fn find(mut self, v: V) -> (root: V, pot: Int)
    fn unite(mut self, a: V, b: V, diff: Int) -> Bool
    fn diff(mut self, a: V, b: V) -> ?Int
    fn same(mut self, a: V, b: V) -> Bool
    fn size(mut self, v: V) -> Int
    @get fn components(self) -> Int
    @get fn consistent(self) -> Bool
    fn mark(mut self) -> Mark // O(1)
    fn undo_to(mut self, snap: Mark) // O(1) per undone unite
struct RollbackArray<D, T> // [new] O(1) get/set/mark, O(changes) undo
    fn new(init: [D]T) -> RollbackArray<D, T> // O(D.K)
    fn filled(val: T) -> RollbackArray<D, T> // O(D.K)
    fn get(self, i: D) -> T // O(1)
    fn set(mut self, i: D, val: T) // O(1)
    fn mark(self) -> Mark // O(1)
    fn undo_to(mut self, snap: Mark) // O(1) per undone set
struct Marks<D> // O(1)
    fn new() -> Marks<D> // O(D.K), O(1)
    fn add(mut self, v: D) -> Bool // O(1)
    fn remove(mut self, v: D) -> Bool // O(1)
    fn contains(self, v: D) -> Bool // O(1)
    fn clear(mut self) // O(1)
    @get fn len(self) -> Int // O(1)
    @get fn is_empty(self) -> Bool
struct MarkMap<D, T> // [new] O(1) epoch-stamped sparse array over D
    fn new(default: T) -> MarkMap<D, T> // O(D.K)
    fn get(self, v: D) -> T // O(1)
    fn set(mut self, v: D, x: T) // O(1)
    fn contains(self, v: D) -> Bool // O(1)
    fn remove(mut self, v: D) -> Bool // O(1)
    fn clear(mut self) // O(1)
    @get fn len(self) -> Int // O(1)
    @get fn is_empty(self) -> Bool // O(1)
struct Interner<K> // [new] online key <-> 0..len id
    fn new() -> Interner<K> // O(1)
    @get fn len(self) -> Int // O(1)
    @get fn is_empty(self) -> Bool // O(1)
    fn id(mut self, k: K) -> Int // O(log n)
    fn get(self, k: K) -> ?Int // O(log n)
    fn key(self, idx: Int) -> K // O(1)
    @get fn keys(self) -> List<K> // O(1)
struct FlatMap<V> // [new] std.hashmap; O(1) expected open-addressing Int map
    fn new(cap: Int = 16) -> FlatMap<V> // O(cap)
    @get fn len(self) -> Int // O(1)
    @get fn is_empty(self) -> Bool // O(1)
    fn clear(mut self) // O(table_size)
    fn contains(self, k: Int) -> Bool // O(1) expected
    fn get(self, k: Int) -> ?V // O(1) expected
    fn set(mut self, k: Int, v: V) // O(1) expected
    fn remove(mut self, k: Int) -> ?V // O(1) expected
    fn each(self) -> yields (key: Int, val: V) // O(table_size)
```

### 4.10 Range structures

Modules `std.algebra`, `std.fenwick`, `std.prefix`, `std.segtree`,
`std.lazy`, `std.sparse`, plus [new] `std.window`, `std.mo`, and advanced
structures (`std.persistent`, `std.lichao`, `std.beats`, `std.mergesort`).
Full design, contest survey, laws and exemplars: `design/stdlib/ranges.md`.

The algebra is values, not traits: a `Monoid<T>` is an identity and an
operation, an `Action<T, F>` a monoid of tags acting on one of values.
`laws_hold` checks the laws on samples in tests. The 0.3 spec moves
these to traits; that waits for trait-associated constants on
user-declared traits and stays a [rename] candidate (decision 26.4).

```cp
struct Monoid<T>(id: T, op: (T, T) -> T)
    fn of<M>(m: M) -> Monoid<T>
struct Action<T, F>(id: F, compose: (F, F) -> F,
                    apply: (F, T, Int) -> T, over: Monoid<T>)
    fn new<M>(id: F, compose: (F, F) -> F,
              apply: (F, T, Int) -> T, over: M) -> Action<T, F>
struct Desc<T>(x: T)
fn desc<T>(x: T) -> Desc<T>
trait Reversible
fn fold<T, M>(m: M, xs: List<T>) -> T
struct MaxSub(has: Bool, best: Int, total: Int, pre: Int, suf: Int) // [new] O(1), Reversible
    const empty: MaxSub
    fn of(v: Int) -> MaxSub
    fn join(a: MaxSub, b: MaxSub) -> MaxSub           // `+`
    fn scale(self, k: Int) -> MaxSub                  // `*`, k >= 0
    @get fn try_best(self) -> ?Int
    fn monoid() -> Monoid<MaxSub>
struct Fenwick<D, T> // O(n), O(log n)
    fn new(a: ?[D]T = none) -> Fenwick<D, T>
    fn add(mut self, i: D, x: T)
    fn prefix(self, b: Bounds<D>) -> T
    fn sum(self, r: Range<D>) -> T
    fn lower_bound(self, target: Int) -> ?Bounds<D> // O(log n)
    fn kth(self, k: Int) -> ?D // O(log n)
struct DualFenwick<D, T> // [new] range add, point get; O(n), O(log n)
    fn new(a: ?[D]T = none) -> DualFenwick<D, T>
    fn add(mut self, r: Range<D>, x: T)
    fn get(self, i: D) -> T
struct RangeFenwick<D> // [new] range add, range sum (Int); O(n), O(log n)
    fn new(a: ?[D]Int = none) -> RangeFenwick<D>
    fn add(mut self, r: Range<D>, x: Int)
    fn prefix(self, b: Bounds<D>) -> Int
    fn sum(self, r: Range<D>) -> Int
struct XorFenwick<D, T> // O(n), O(log n)
    fn new(a: ?[D]T = none) -> XorFenwick<D, T>
    fn flip(mut self, i: D, x: T)
    fn prefix(self, b: Bounds<D>) -> T
    fn xor(self, r: Range<D>) -> T
struct Fenwick2D<R, C, T> // O(log R * log C)
    fn new() -> Fenwick2D<R, C, T>
    fn add(mut self, r: R, c: C, x: T)
    fn sum(self, rows: Range<R>, cols: Range<C>) -> T
struct Prefix1<D, T>
    fn new(a: [D]T) -> Prefix1<D, T>
    fn prefix(self, b: Bounds<D>) -> T
    fn sum(self, r: Range<D>) -> T
struct Prefix2<R, C, T>
    fn new(a: [R, C]T) -> Prefix2<R, C, T>
    fn sum(self, rows: Range<R>, cols: Range<C>) -> T
struct PrefixN<T>
    fn new(a: List<T>) -> PrefixN<T>
    fn prefix(self, k: Int) -> T
    fn sum(self, r: Range<Int>) -> T
fn prefix_n<T>(a: List<T>) -> PrefixN<T>
fn prefix1<D, T>(a: [D]T) -> Prefix1<D, T>
fn prefix2<R, C, T>(a: [R, C]T) -> Prefix2<R, C, T>
fn prefix_fold<D, T, M>(op: M, a: [D]T) -> [D.bounds]T // [new] O(n)
fn suffix_fold<D, T, M>(op: M, a: [D]T) -> [D.bounds]T // [new] O(n)
struct Diff1<D, T> // [new] offline 1D range add; O(1) add, O(n) finish
    fn new() -> Diff1<D, T>
    fn add(mut self, r: Range<D>, x: T)
    fn finish(self) -> [D]T
struct Diff2<R, C, T> // [new] offline 2D rect add; O(1) add, O(r * c) finish
    fn new() -> Diff2<R, C, T>
    fn add(mut self, rows: Range<R>, cols: Range<C>, x: T)
    fn finish(self) -> [R, C]T
struct SegTree<D, T>(m: Monoid<T>) // O(log n)
    fn new<M>(op: M, a: ?[D]T = none, fill: ?T = none) -> SegTree<D, T> // O(n)
    fn of<M>(op: M, f: D -> T) -> SegTree<D, T> // O(n)
    fn refresh(mut self, i: D) // O(log n)
    fn get(self, i: D) -> T
    fn set(mut self, i: D, x: T)
    fn update(mut self, i: D, f: T -> T) // O(log n)
    fn fold(self, r: Range<D>) -> T
    @get fn all(self) -> T // O(1)
    @get fn monoid(self) -> Monoid<T>
    fn max_right(self, from: Bounds<D>, ok: T -> Bool) -> Bounds<D>
    fn min_left(self, to: Bounds<D>, ok: T -> Bool) -> Bounds<D>
struct LazySegTree<D, T, F>(act: Action<T, F>) // O(log n), O(1)
    fn new(act: Action<T, F>, a: ?[D]T = none, fill: ?T = none) -> LazySegTree<D, T, F> // O(n)
    fn set(mut self, i: D, x: T)
    fn update(mut self, i: D, f: T -> T) // O(log n)
    fn get(mut self, i: D) -> T
    @get fn all(self) -> T // O(1)
    @get fn monoid(self) -> Monoid<T>
    fn fold(mut self, r: Range<D>) -> T
    fn apply(mut self, r: Range<D>, f: F)
    fn max_right(mut self, from: Bounds<D>, ok: T -> Bool) -> Bounds<D>
    fn min_left(mut self, to: Bounds<D>, ok: T -> Bool) -> Bounds<D>
struct DualSegTree<D, T, F> // [new] range apply, point get; no monoid or pulls; O(log n)
    fn new(id: F, compose: (F, F) -> F, apply: (F, T) -> T,
           a: ?[D]T = none, fill: ?T = none) -> DualSegTree<D, T, F> // O(n)
    fn apply(mut self, r: Range<D>, f: F)
    fn get(mut self, i: D) -> T
    fn set(mut self, i: D, x: T)
    fn update(mut self, i: D, f: T -> T)
const AddSum = Action<Int, Int>(0, +, (f, x, len) => x + f * len, over: +)
const AddMin = Action<Int, Int>(0, +, (f, x, len) => x + f, over: min)
const AddMax = Action<Int, Int>(0, +, (f, x, len) => x + f, over: max)
const AssignSum: Action<Int, ?Int>   // assign over sum
const AssignMin: Action<Int, ?Int>   // assign over min
const AssignMax: Action<Int, ?Int>   // assign over max
const AffineSum: Action<Int, (Int, Int)> // [new] (mul, add) over sum
fn affine_sum<T>() -> Action<T, (T, T)>  // [new] generic (Int, Mod<P>)
struct SparseTable<D, T>(op: (T, T) -> T) // O(n log n), O(1) idempotent
    fn new(op: (T, T) -> T, a: [D]T) -> SparseTable<D, T>
    fn fold(self, r: Range<D>) -> T // O(1)
struct DisjointSparseTable<D, T>(op: (T, T) -> T) // [new] O(n log n), O(1) any semigroup
    fn new(op: (T, T) -> T, a: [D]T) -> DisjointSparseTable<D, T>
    fn fold(self, r: Range<D>) -> T // O(1)
fn idempotent<T>(op: (T, T) -> T, xs: List<T>) -> Bool
struct MonoDeque<T> // [new] O(1) amortized sliding-window min/max
    fn new(pick: (T, T) -> T) -> MonoDeque<T>
    fn by(better: (T, T) -> Bool) -> MonoDeque<T>
    @get fn len(self) -> Int
    @get fn is_empty(self) -> Bool
    @get fn best(self) -> ?T
    fn clear(mut self)
    fn push(mut self, at: Int, val: T)
    fn drop_before(mut self, min_at: Int)
struct WindowFold<T>(m: Monoid<T>) // [new] O(1) amortized inverse-free queue fold
    fn new<M>(op: M) -> WindowFold<T>
    @get fn len(self) -> Int
    @get fn is_empty(self) -> Bool
    @get fn fold(self) -> T
    fn push(mut self, x: T)
    fn pop(mut self) -> ?T
fn mo_order<D>(queries: List<Range<D>>) -> List<Int> // [new] O(q log q)
fn mo_steps<D>(from: Range<D>, to: Range<D>) -> yields (pos: D, entering: Bool) // [new]
```

Phase 2 additions (see `design/stdlib/ranges.md` §3.8):
`MergeSortTree<D, T>` and `SparseSegTree<T>` (`std.segtree`),
`PersistentSegTree<D, T>` + `Version` + `RangeKth<D, T>` (`std.persistent`),
`BeatsSegTree<D>` (`std.beats`: `chmin`/`chmax`/`add`/`sum`/`min_val`/`max_val`),
`TimeTree<E>` + `TimeStep` (`std.timetree`: offline time-interval divide-and-conquer),
and `LiChao` + `LiChaoTag<V>` (coordinated with the DP lane).

### 4.11 `std.graph`, `std.tree`, `std.tour`: graphs and trees

`Graph<V>` is a compressed adjacency list over a vertex domain, built by
`undirected`/`directed`; `Rooted<V>` a rooted tree with parents, depths,
heavy-first preorder positions `t.Pre` in which every subtree and every heavy
chain is a range, so one range structure serves subtree and path queries;
`Centroids<V>` a centroid decomposition. The design, laws and worked problems
are in `design/stdlib/trees.md`.

```cp
type Tree<V> = [0..V.len - 1](u: V, v: V)
type SimpleGraph<V, E> = [E](u: V, v: V)
type ConnectedGraph<V, E> = [E](u: V, v: V)
type Dag<V, E> = [E](u: V, v: V)
type Digraph<V, E> = [E](u: V, v: V)
trait Neighbours
trait Target
struct Graph<V, E> // O(n + m)
    fn directed(edges: [E](u: V, v: V)) -> Graph<V, E>
    fn undirected(edges: [E](u: V, v: V)) -> Graph<V, E>
    fn out(self, u: V) -> yields (to: V, e: E)
    fn get(self, u: V) -> List<(to: V, e: E)>
    fn deg(self, u: V) -> Int // O(1)
    fn transpose(self) -> Graph<V, E> // O(n + m)
// list-form hooks behind `Graph.directed(pairs)` etc., never called by name
fn directed_of_pairs<V>(edges: List<(V, V)>) -> [V]List<V> // O(n + m)
fn undirected_of_pairs<V>(edges: List<(V, V)>) -> [V]List<V> // O(n + m)
fn directed_of_triples<V, W>(edges: List<(V, V, W)>) -> [V]List<(V, W)> // O(n + m)
fn undirected_of_triples<V, W>(edges: List<(V, V, W)>) -> [V]List<(V, W)> // O(n + m)
struct Paths<V, W, A> // dist, parent, via: [V]?_; order: List<V>
    fn get(self, v: V) -> ?W // O(1)
    fn reached(self, v: V) -> Bool // O(1)
    fn path(self, t: V) -> ?List<V> // O(len)
    fn arcs(self, t: V) -> ?List<A> // O(len)
fn bfs<V, N>(next: N, from: List<V>) -> Paths<V, Int, A> // O(n + m)
// O(n + m) deque; traps unless every weight is 0 or 1
fn bfs01<V, N, A>(next: N, from: List<V>, w: A -> Int) -> Paths<V, Int, A>
// O((n + m) log m)
fn dijkstra<V, N, A, W>(next: N, from: List<(V, W)>, w: A -> W)
    -> Paths<V, W, A>
// O(|S| |moves|): one FIFO per move, no heap
fn fixed_cost_paths<S, M, W>(from: List<S>, moves: List<M>, cost: M -> W,
    step: (M, S) -> ?S) -> Paths<S, W, M>
// O(n m); none on a reachable negative cycle
fn bellman_ford<V, N, A, W>(next: N, from: List<(V, W)>, w: A -> W)
    -> ?Paths<V, W, A>
fn floyd_warshall<V, W>(w: [V, V]?W) -> ?[V, V]?W // O(n^3)
fn topo<V, N>(next: N) -> ?List<V> // O(n + m)
fn peel<V, N>(next: N) -> List<V> // O(n + m)
fn find_cycle<V, N>(next: N) -> ?List<V> // O(n + m)
fn components<V, N>(next: N) -> (domain C, comp: [V]C) // O(n + m)
fn scc<V, N>(next: N) -> (domain C, comp: [V]C) // O(n + m), C topological
fn condense<V, N, C>(next: N, comp: [V]C) -> [C]List<(to: C)>
fn two_edge_components<V, N>(next: N) -> (domain C, comp: [V]C) // O(n + m)
fn bridges<V, E, N>(next: N) -> [E]Bool // O(n + m)
fn articulation_points<V, N>(next: N) -> [V]Bool // O(n + m)
fn biconnected_components<V, N>(next: N) -> List<List<V>> // O(n + m)
fn two_color<V, N>(next: N) -> ?[V]Bool // O(n + m)
struct Walk<V, E> // start: V, arcs: List<(to: V, e: E)>
    @get fn vertices(self) -> List<V> // O(len)
// O(n + m) Hierholzer
fn euler_walk<V, E>(edges: [E](u: V, v: V), directed: Bool) -> ?Walk<V, E>
struct Lit<X>(x: X, value: Bool)
    @get fn flip(self) -> Lit<X>
struct TwoSat<X>
    fn new() -> TwoSat<X>
    fn add_or(mut self, a: Lit<X>, b: Lit<X>) // O(1)
    fn implies(mut self, a: Lit<X>, b: Lit<X>) // O(1)
    fn must(mut self, a: Lit<X>) // O(1)
    fn at_most_one(mut self, lits: List<Lit<X>>) // O(k)
    fn solve(self) -> ?[X]Bool // O(|X| + clauses)
struct Functional<V> // O(n log n)
    fn new(next: [V]V) -> Functional<V>
    fn on_ring(self, v: V) -> Bool // O(1)
    fn ring_of(self, v: V) -> Int // O(1)
    fn steps_to_ring(self, v: V) -> Int // O(1)
    fn root(self, v: V) -> V // O(1)
    fn jump(self, v: V, k: Int) -> V // O(log n)
    fn dist(self, a: V, b: V) -> ?Int // O(log n)
    fn orbit(self, v: V) -> Int // O(1)
    fn jump_while(self, v: V, pred: V -> Bool, max_steps: Int = V.K)
        -> ?(V, Int) // O(log n)
struct Rooted<V, Pre> // O(n) time and memory; design/stdlib/trees.md
    fn new<N>(adj: N, root: V)
    fn from_parents(par: [V]?V) // O(n)
    fn weighted<N, A>(adj: N, root: V, len: A -> Int)
    fn forest<N>(adj: N, roots: List<V>) // [new] O(n)
    fn parent(self, v: V) -> ?V // O(1), like every shape query
    fn children(self, v: V) -> List<V> // in the order adj listed them
    heavy: [V]?V // O(1) field, like at and far
    fn light(self, v: V) -> yields V // [new] O(deg v)
    fn depth(self, v: V) -> Int
    far: [V]Int // weighted depth
    fn size(self, v: V) -> Int
    fn root_of(self, v: V) -> V // [new]
    @get fn subtree_sizes(self) -> [V]Int // O(n)
    fn pos(self, v: V) -> Pre // O(1)
    at: [Pre]V // inverse of pos
    fn sub(self, v: V) -> Range<Pre> // O(1)
    fn subtree(self, v: V) -> yields V // [new] O(size v)
    fn reorder<T>(self, a: [V]T) -> [Pre]T // O(n)
    @get fn preorder(self) -> List<V> // O(n)
    @get fn postorder(self) -> List<V> // O(n)
    @get fn tour(self) -> yields (v: V, entering: Bool) // O(n)
    fn is_anc(self, a: V, b: V) -> Bool // O(1)
    fn lower(self, u: V, v: V) -> V // O(1)
    fn lca(self, a: V, b: V) -> V // O(log n)
    fn dist(self, a: V, b: V) -> Int // O(log n)
    fn far_dist(self, a: V, b: V) -> Int // [new] O(log n)
    fn ancestor(self, v: V, k: Int) -> ?V // O(log n)
    fn step(self, a: V, b: V, k: Int) -> ?V // O(log n)
    fn lift_while(self, v: V, pred: V -> Bool) -> ?V // O(log^2 n)
    fn path(self, a: V, b: V) -> List<V> // O(|path|)
    fn chains( // O(log n)
        self, a: V, b: V, edges: Bool = false,
    ) -> yields (range: Range<Pre>, rising: Bool)
    // O(log n) range folds; path order when the monoid is Reversible
    fn fold_path<S, T>(self, a: V, b: V, ds: S, edges: Bool = false) -> T
    // O(log n) range applies
    fn apply_path<S, F>(self, a: V, b: V, mut ds: S, tag: F, edges: Bool = false)
    fn down<T>(self, init: T, f: (T, V) -> T) -> [V]T // [new] O(n)
    // [new] O(n): rooted DP, and the same DP for every root
    fn up<T, E, M>(self, m: M, lift: (T, V, V) -> E, finish: (E, V) -> T) -> [V]T
    fn reroot<T, E, M>(self, m: M, lift: (T, V, V) -> E, finish: (E, V) -> T) -> [V]T
    // O(k)
    fn excl_fold<T, M>(self, v: V, m: M, f: V -> T) -> yields (c: V, rest: T)
    fn diameter(self) -> (len: Int, u: V, v: V) // O(n)
    @get fn centroid(self) -> V // O(depth)
    @get fn centroids(self) -> List<V> // O(depth)
    fn shapes(self) -> (domain Shape, of: [V]Shape) // [new] O(n log n)
    // O(k log k + k log n)
    fn virtual_tree(self, keys: List<V>)
        -> (nodes: List<V>, edges: List<(u: V, v: V)>)
struct Centroids<V> // [new] O(n log n) time and memory
    fn new<N>(adj: N) -> Centroids<V>
    fn parent(self, c: V) -> ?V // O(1)
    fn depth(self, c: V) -> Int // O(1), at most log2 n
    fn ancestors(self, v: V) -> List<(c: V, dist: Int)> // O(1), v's own first
    fn piece(self, c: V) -> yields (v: V, dist: Int, branch: V) // O(|piece|)
```

[new], promoted from `examples/local-lib/graph/`:

```cp
fn mst<V>(edges: List<(u: V, v: V, w: Int)>) -> (total: Int, used: List<Int>)
    // O(m log m) Kruskal; indices of the chosen edges
```

### 4.12 `std.flow`: flows, cuts and matchings

The design and its reasons are in `design/stdlib/flows.md`. Two networks
share `Arc` and one accessor vocabulary (`flow`, `cap`, `source`,
`target`, `rev`); everything whose input is a table or an edge list is a
function returning a value. Objectives are named (`max_`/`min_`), never
encoded by negating weights; options are named arguments (`limit:`,
`cap: none`). Sides, vertices and input edges are domains: a matching is
`[L]?R`, a bounded flow is `[E]Int`. All of it ships in `stdlib/flow.cp`, with
matched-C++ timings in `bench/flow/`.

```cp
struct Arc(id: Int)
    @get fn rev(self) -> Arc                                  // O(1)
struct FlowGraph<V>                    // Dinic
    fn new() -> FlowGraph<V>                                  // O(V)
    fn add(mut self, u: V, v: V, cap: ?Int) -> Arc            // O(1)
    fn add_both(mut self, u: V, v: V, cap: Int) -> Arc        // O(1)
    fn flow(self, a: Arc) -> Int                              // O(1)
    fn cap(self, a: Arc) -> Int                               // O(1)
    fn source(self, a: Arc) -> V                              // O(1)
    fn target(self, a: Arc) -> V                              // O(1)
    fn max_flow(mut self, s: V, t: V, limit: ?Int = none) -> Int
        // O(V^2 E); O(E sqrt V) on unit networks
    fn min_cut(self, s: V) -> [V]Bool                         // O(V + E)
    fn reset(mut self)                                        // O(E)
struct CostFlow<V>                     // successive shortest paths
    fn new() -> CostFlow<V>                                   // O(V)
    fn add(mut self, u: V, v: V, cap: ?Int, cost: Int) -> Arc // O(1)
    fn flow(self, a: Arc) -> Int                              // O(1)
    fn cap(self, a: Arc) -> Int                               // O(1)
    fn cost(self, a: Arc) -> Int                              // O(1)
    fn source(self, a: Arc) -> V                              // O(1)
    fn target(self, a: Arc) -> V                              // O(1)
    fn min_cost_flow(mut self, s: V, t: V, limit: ?Int = none)
        -> (flow: Int, cost: Int)
        // O(F E log V); plus O(V E) once when a cost is negative
    fn slope(mut self, s: V, t: V, limit: ?Int = none)
        -> List<(flow: Int, cost: Int)>                       // same
fn max_closure<V, E>(profit: [V]Int, needs: [E](u: V, v: V))
    -> (value: Int, take: [V]Bool)               // one max flow
fn max_matching<L, R>(adj: [L]List<R>)
    -> (size: Int, mate: [L]?R, back: [R]?L)     // O(E sqrt V)
fn min_vertex_cover<L, R>(adj: [L]List<R>)
    -> (left: [L]Bool, right: [R]Bool)           // O(E sqrt V)
fn max_weight_matching<L, R>(weight: (L, R) -> Int)
    -> (total: Int, mate: [L]R)                  // O(L^2 R), L <= R
fn min_weight_matching<L, R>(weight: (L, R) -> Int)
    -> (total: Int, mate: [L]R)                  // O(L^2 R), L <= R
fn general_matching<V>(adj: [V]List<V>)
    -> (size: Int, mate: [V]?V)                  // O(V^3) Edmonds
fn circulation<V, E>(arcs: [E](u: V, v: V, lo: Int, hi: Int)) -> ?[E]Int
fn max_bounded_flow<V, E>(arcs: [E](u: V, v: V, lo: Int, hi: Int), s: V, t: V)
    -> ?(value: Int, flow: [E]Int)
fn min_bounded_flow<V, E>(arcs: [E](u: V, v: V, lo: Int, hi: Int), s: V, t: V)
    -> ?(value: Int, flow: [E]Int)
    // each two max flows on V + 2 vertices
fn gomory_hu<V, E>(edges: [E](u: V, v: V, w: Int))
    -> List<(u: V, v: V, w: Int)>                // V - 1 max flows
fn global_min_cut<V, E>(edges: [E](u: V, v: V, w: Int))
    -> (w: Int, side: [V]Bool)                   // Stoer-Wagner, O(V^3)
```

Staged (design section 7): weighted general matching,
`min_cost_circulation`, `set_cap`, matroid intersection. Network nodes
built from several domains wait on sum domains (spec §21.11, core lane);
until then `max_closure`, the bounded flows and the matchings keep their
source and sink inside the library.

### 4.13 `std.strings`: strings

ASCII byte strings, KMP/prefix and Z functions, borders and Lyndon words,
palindromes (Manacher and palindromic tree), suffix array/LCP and suffix
automaton, tries and Aho-Corasick, and double-mod rolling hashing (1D and 2D).
Full design and exemplars in `design/stdlib/strings.md`.

```cp
fn repeat(self: Str, n: Int) -> Str
fn starts_with(self: Str, p: Str) -> Bool // O(p.len)
fn ends_with(self: Str, p: Str) -> Bool // O(p.len)
fn find<P>(self: Str, p: P) -> ?Int // O(s.len + p.len)
fn count(self: Str, c: Char) -> Int // O(s.len)
fn rfind<P>(self: Str, p: P) -> ?Int // O(s.len + p.len)
@get fn lower(self: Char) -> Char
@get fn upper(self: Char) -> Char
@get fn digit(self: Char) -> Int
fn z_function<T>(s: List<T>) -> List<Int>
fn prefix_function<T>(s: List<T>) -> List<Int>
fn find_all<T>(self: List<T>, pat: List<T>) -> yields Int
fn period<T>(s: List<T>) -> Int // O(n)
fn suffix_array<T>(s: List<T>) -> List<Int>
fn lcp_array<T>(s: List<T>, sa: List<Int>) -> List<Int>
struct HashVal(h1: Int, h2: Int)
    @get fn hash_token(self) -> Int
trait Hashable
struct RollingHash // O(n), O(1)
    fn new<S>(seq: S, bases: ?(Int, Int)
    fn with_bases<S>(
    @get fn len(self) -> Int
    @get fn all(self) -> HashVal // O(1)
    fn slice(self, l: Int, r: Int) -> HashVal // O(1)
    fn get<R>(self, r: R) -> HashVal // O(1)
    fn combine(self, a: HashVal, b: HashVal, b_len: Int) -> HashVal // O(1)
```

[new], designed in `design/stdlib/strings.md` and promoted from
`examples/local-lib/str/`:

```cp
fn is_palindrome<T>(s: List<T>) -> Bool                         // O(n)
fn borders<T>(s: List<T>) -> yields Int                         // O(n)
fn prefix_matches<T>(self: List<T>, pat: List<T>) -> List<Int>  // O(|self| + |pat|)
fn overlap<T>(a: List<T>, b: List<T>) -> Int                    // O(|a| + |b|)
fn lyndon_factorization<T>(s: List<T>) -> List<Int>             // O(n)
fn min_rotation<T>(s: List<T>) -> Int                           // O(n)
struct Palindromes(odd: List<Int>, even: List<Int>)
    fn is_palindrome(self, l: Int, r: Int) -> Bool              // O(1)
    @get fn count(self) -> Int                                  // O(n)
    @get fn longest(self) -> (l: Int, r: Int)                   // O(n)
fn manacher<T>(s: List<T>) -> Palindromes                       // O(n)
struct PalindromicTree<A>
    fn new() -> PalindromicTree<A>                              // O(|A|)
    fn of(s: List<A>) -> PalindromicTree<A>                     // O(n * |A|) mem, O(n)
    fn push(mut self, a: A) -> Int                              // O(1) amortized
    @get fn len(self) -> Int                                    // O(1)
    @get fn distinct(self) -> Int                               // O(1)
    fn pal_len(self, state: Int) -> Int                         // O(1)
    fn link(self, state: Int) -> Int                            // O(1)
    fn step(self, state: Int, a: A) -> ?Int                     // O(1)
    fn diff(self, state: Int) -> Int                            // O(1)
    fn series_link(self, state: Int) -> Int                     // O(1)
    fn occurrences(self) -> List<Int>                           // O(n)
fn RollingHash.lcp(self, a: Int, b: Int) -> Int                 // O(log n)
fn RollingHash.lcs(self, a: Int, b: Int) -> Int                 // O(log n)
fn RollingHash.lcp_with(self, a: Int, other: RollingHash, b: Int) -> Int // O(log n)
struct Hash2D<R, C>
    fn new(code: (R, C) -> Int, bases: ?(Int, Int) = none) -> Hash2D<R, C> // O(R * C)
    fn rect(self, rows: Range<R>, cols: Range<C>) -> HashVal    // O(1)
struct SuffixArray<T>
    fn new(s: List<T>) -> SuffixArray<T>                        // O(n log n)
    @get fn len(self) -> Int                                    // O(1)
    fn lcp_between(self, i: Int, j: Int) -> Int                 // O(1)
    fn matches(self, pat: List<T>) -> Range<Int>                // O(|pat| log n)
    fn count(self, pat: List<T>) -> Int                         // O(|pat| log n)
    fn lcp_range(self, i: Int, len: Int) -> Range<Int>          // O(log n)
    fn compare(self, l1: Int, r1: Int, l2: Int, r2: Int) -> Int // O(1)
    @get fn distinct(self) -> Int                               // O(n)
struct SuffixAutomaton<A>
    fn new() -> SuffixAutomaton<A>                              // O(|A|)
    fn of(s: List<A>) -> SuffixAutomaton<A>                     // O(n * |A|) mem, O(n)
    fn push(mut self, a: A)                                     // O(1) amortized
    @get fn root(self) -> Int                                   // O(1)
    @get fn len(self) -> Int                                    // O(1)
    fn step(self, state: Int, a: A) -> ?Int                     // O(1)
    fn link(self, state: Int) -> ?Int                           // O(1)
    fn longest(self, state: Int) -> Int                         // O(1)
    fn shortest(self, state: Int) -> Int                        // O(1)
    fn find(self, word: List<A>) -> ?Int                        // O(|word|)
    fn contains(self, word: List<A>) -> Bool                    // O(|word|)
    @get fn distinct(self) -> Int                               // O(states)
    fn topo_order(self) -> List<Int>                            // O(states)
    fn occurrences(self) -> List<Int>                           // O(states)
struct Trie<A>
    fn new() -> Trie<A>                                         // O(|A|)
    @get fn root(self) -> Int                                   // O(1)
    @get fn len(self) -> Int                                    // O(1)
    fn step(self, node: Int, a: A) -> ?Int                      // O(1)
    fn find(self, word: List<A>) -> ?Int                        // O(|word|)
    fn add(mut self, word: List<A>) -> Int                      // O(|word|)
    fn remove(mut self, word: List<A>) -> Bool                  // O(|word|)
    fn count(self, word: List<A>) -> Int                        // O(|word|)
    fn count_prefix(self, prefix: List<A>) -> Int               // O(|prefix|)
    fn ends_at(self, node: Int) -> Int                          // O(1)
    fn pass_at(self, node: Int) -> Int                          // O(1)
struct AhoCorasick<A>
    fn new(patterns: List<List<A>>) -> AhoCorasick<A>           // O(sum |p| * |A|)
    @get fn root(self) -> Int                                   // O(1)
    @get fn len(self) -> Int                                    // O(1)
    fn step(self, state: Int, a: A) -> Int                      // O(1)
    fn link(self, state: Int) -> Int                            // O(1)
    fn hit(self, state: Int) -> Bool                            // O(1)
    fn words_at(self, state: Int) -> List<Int>                  // O(output)
    fn matches(self, text: List<A>) -> yields (at: Int, pattern: Int) // O(|text| + output)
    fn counts(self, text: List<A>) -> List<Int>                 // O(|text| + states)
```

### 4.14 `std.geometry`: 2D integer and floating-point geometry

Exact integer points (`Pt`) and careful floating-point vectors (`Vec`),
kept as distinct types so exact predicates never silently mix with
approximate arithmetic. Full design and contest examples live in
`design/stdlib/geometry.md`. Orientation predicates compute cross and dot
signs in `I128`, staying exact for coordinates up to `1e18`.

```cp
struct Pt(x: Int, y: Int) // Add, Sub, Neg, Eq, Ord (x, y), Hash, Read, Show
    @get fn norm2(self) -> Int // O(1)
    @get fn len(self) -> F64 // O(1)
    @get fn perp(self) -> Pt // O(1), 90 deg CCW
fn dot(a: Pt, b: Pt) -> Int // O(1)
fn cross(a: Pt, b: Pt) -> Int // O(1)
fn turn(a: Pt, b: Pt, c: Pt) -> Int // O(1), I128: 1 CCW, -1 CW, 0 collinear
struct ByAngle(p: Pt) // Ord: CCW in [0, 2*pi) from +x, zero first, I128
fn on_segment(p: Pt, a: Pt, b: Pt) -> Bool // O(1), I128
fn segments_meet(a: Pt, b: Pt, c: Pt, d: Pt) -> Bool // O(1), I128
fn area2(poly: List<Pt>) -> Int // O(n)
enum Location = Inside | Boundary | Outside
fn locate(poly: List<Pt>, p: Pt) -> Location // O(n), I128
```

[new]:

```cp
// Pt methods and exact integer helpers
    @get fn perp_cw(self) -> Pt // O(1), 90 deg CW
    @get fn angle(self) -> F64 // O(1), atan2(y, x)
    @get fn unit_dir(self) -> Pt // O(log min(|x|, |y|)), reduced by gcd
    @get fn to_vec(self) -> Vec // O(1)
    fn scale(self, k: Int) -> Pt // O(1)
    fn dot(self, o: Pt) -> Int // O(1)
    fn cross(self, o: Pt) -> Int // O(1)
    fn dist2(self, o: Pt) -> Int // O(1)
    fn dist(self, o: Pt) -> F64 // O(1)
fn dot128(a: Pt, b: Pt) -> I128 // O(1)
fn cross128(a: Pt, b: Pt) -> I128 // O(1)
fn dist2(a: Pt, b: Pt) -> Int // O(1)
fn dist2_128(a: Pt, b: Pt) -> I128 // O(1)
fn dist(a: Pt, b: Pt) -> F64 // O(1)
fn area2_128(poly: List<Pt>) -> I128 // O(n)
fn same_dir(a: Pt, b: Pt) -> Bool // O(1), I128
fn angle_less(a: Pt, b: Pt) -> Bool // O(1), I128
fn on_segment(p: Pt, a: Pt, b: Pt, strict: Bool = false) -> Bool // O(1), I128
fn segments_meet(a: Pt, b: Pt, c: Pt, d: Pt, strict: Bool = false) -> Bool // O(1)

// Convex hulls, chains, Minkowski sums, calipers, closest pair
fn lower_hull(pts: List<Pt>, collinear: Bool = false) -> List<Pt> // O(n), sorted
fn upper_hull(pts: List<Pt>, collinear: Bool = false) -> List<Pt> // O(n), sorted
fn convex_hull(pts: List<Pt>, collinear: Bool = false) -> List<Pt> // O(n log n)
fn convex_hull_idx<D>(pts: [D]Pt, collinear: Bool = false) -> List<D> // O(n log n)
fn minkowski_sum(a: List<Pt>, b: List<Pt>) -> List<Pt> // O(|a| + |b|)
fn minkowski_upper(a: List<Pt>, b: List<Pt>) -> List<Pt> // O(|a| + |b|)
fn locate_convex(hull: List<Pt>, p: Pt) -> Location // O(log n), I128
fn diameter2(hull: List<Pt>) -> Int // O(n) rotating calipers
fn hull_extreme(hull: List<Pt>, dir: Pt) -> Int // O(log n), I128
fn hull_tangents(hull: List<Pt>, p: Pt) -> (left: Int, right: Int) // O(log n), I128
fn closest_pair2(pts: List<Pt>) -> Int // O(n log n)
fn closest_pair<D>(pts: [D]Pt) -> (a: D, b: D, dist2: Int) // O(n log n)

// Careful floating-point geometry and half-plane intersection
fn close(a: F64, b: F64, eps: F64 = 1e-9) -> Bool // O(1)
fn sign_eps(x: F64, eps: F64 = 1e-9) -> Int // O(1)
struct Vec(x: F64, y: F64) // Add, Sub, Neg; no Show (precision required)
    @get fn norm2(self) -> F64 // O(1)
    @get fn len(self) -> F64 // O(1)
    @get fn unit(self) -> Vec // O(1)
    @get fn perp(self) -> Vec // O(1)
    @get fn angle(self) -> F64 // O(1)
    fn scale(self, k: F64) -> Vec // O(1)
    fn div(self, k: F64) -> Vec // O(1)
    fn rot(self, by: Vec) -> Vec // O(1), rotate by unit direction vector
fn vdot(a: Vec, b: Vec) -> F64 // O(1)
fn vcross(a: Vec, b: Vec) -> F64 // O(1)
fn vdist(a: Vec, b: Vec) -> F64 // O(1)
fn vturn(a: Vec, b: Vec, c: Vec, eps: F64 = 1e-9) -> Int // O(1)
fn project(p: Vec, a: Vec, b: Vec) -> Vec // O(1)
fn reflect(p: Vec, a: Vec, b: Vec) -> Vec // O(1)
fn line_dist(p: Vec, a: Vec, b: Vec) -> F64 // O(1)
fn seg_dist(p: Vec, a: Vec, b: Vec) -> F64 // O(1)
fn line_intersect(a: Vec, b: Vec, c: Vec, d: Vec, eps: F64 = 1e-9) -> ?Vec // O(1)
fn circle_line_intersect(c: Vec, r: F64, a: Vec, b: Vec, eps: F64 = 1e-9) -> List<Vec>
fn circle_intersect(c1: Vec, r1: F64, c2: Vec, r2: F64, eps: F64 = 1e-9) -> List<Vec>
fn circumcenter(a: Vec, b: Vec, c: Vec, eps: F64 = 1e-9) -> ?Vec // O(1)
struct HalfPlane(p: Vec, dir: Vec)
fn half_plane_intersect(planes: List<HalfPlane>, eps: F64 = 1e-9) -> List<Vec> // O(n log n)
fn polygon_area(poly: List<Vec>) -> F64 // O(n)
fn polygon_cut(poly: List<Vec>, a: Vec, b: Vec, eps: F64 = 1e-9) -> List<Vec> // O(n)
fn min_enclosing_circle(pts: List<Vec>, eps: F64 = 1e-9) -> (center: Vec, radius: F64) // expected O(n)
fn circle_polygon_area(c: Vec, r: F64, poly: List<Vec>, eps: F64 = 1e-9) -> F64 // O(n)
fn arg(p: Pt) -> ByAngle // O(1), float-free exact angular sort key
fn any_segments_meet(segs: List<(a: Pt, b: Pt)>, strict: Bool = false) -> ?(Int, Int) // O(n log n), I128
struct Pt3(x: Int, y: Int, z: Int) // Add, Sub, Neg, Eq, Show
    @get fn norm2(self) -> Int // O(1)
    @get fn norm2_128(self) -> I128 // O(1)
    @get fn len(self) -> F64 // O(1)
    fn scale(self, k: Int) -> Pt3 // O(1)
fn dot3(a: Pt3, b: Pt3) -> Int // O(1)
fn dot3_128(a: Pt3, b: Pt3) -> I128 // O(1)
fn cross3(a: Pt3, b: Pt3) -> Pt3 // O(1)
fn dist2_3(a: Pt3, b: Pt3) -> Int // O(1)
fn convex_hull_3d(pts: List<Pt3>) -> List<(Int, Int, Int)> // O(n^2), I128
```

### 4.15 `std.grid` and `std.spec`

`std.grid` gives the four and eight directions and in-bounds stepping
over a `(r, c)` domain; `std.spec` the tools for checking observations
on small cases (monotonicity, convexity, Monge, recurrence guessing).

```cp
struct Dir4(k: Int)
    fn up() -> Dir4
    fn right() -> Dir4
    fn down() -> Dir4
    fn left() -> Dir4
    fn all() -> List<Dir4>
    fn from(letters: Str, ch: Char) -> ?Dir4
    @get fn cw(self) -> Dir4
    @get fn ccw(self) -> Dir4
    @get fn back(self) -> Dir4
    @get fn delta(self) -> (Int, Int)
struct Dir8(k: Int)
    fn all() -> List<Dir8>
    @get fn cw(self) -> Dir8
    @get fn ccw(self) -> Dir8
    @get fn back(self) -> Dir8
    @get fn delta(self) -> (Int, Int)
fn step<R, C, D>(self: (R, C), d: D) -> ?(R, C) // O(1)
fn ray<R, C, D>(self: (R, C), d: D) -> List<(R, C)> // O(length of the ray)
@get fn adj4<R, C>(self: (R, C)) -> List<(R, C)> // O(1)
@get fn adj8<R, C>(self: (R, C)) -> List<(R, C)> // O(1)
@get fn transposed<R, C, T>(self: [R, C]T) -> [C, R]T // O(|R| * |C|)
fn decrease<T>(xs: List<T>) -> ?Int // O(n)
fn increase<T>(xs: List<T>) -> ?Int // O(n)
fn convexity_break(xs: List<Int>) -> ?Int // O(n)
fn concavity_break(xs: List<Int>) -> ?Int
fn unimodality_break(xs: List<Int>) -> ?Int // O(n)
// O(rows * cols)
fn monge_break(cost: (Int, Int) -> Int, rows: Int, cols: Int) -> ?(Int, Int)
// O(rows * cols)
fn argmins(cost: (Int, Int) -> Int, rows: Int, cols: Int) -> List<Int>
fn dnc_break( // O(n log n), O(n^2)
fn knuth_break(opt: (Int, Int) -> Int, n: Int) -> ?(Int, Int) // O(n^2)
fn differences(xs: List<Int>) -> List<Int> // O(n)
fn newton(xs: List<Int>) -> List<Int> // O(n^2)
fn poly_degree(xs: List<Int>) -> ?Int // O(n^2)
fn poly_at(xs: List<Int>, n: Int) -> Int // O(len^2)
fn berlekamp_massey<P>(xs: List<Mod<P>>) -> List<Mod<P>>
// O(k^2 log n)
fn recurrence_at<P>(c: List<Mod<P>>, init: List<Mod<P>>, n: Int) -> Mod<P>
fn guess_at<P>(xs: List<Mod<P>>, n: Int) -> ?Mod<P>
```

### 4.16 `std.dp` [new]: the DP toolkit

The structures and optimizations a DP calls by name; the recurrence stays
in user code, and memoized recursion is `@memo` (core). Each optimization
takes the cost as an operation and states its law, which `std.spec` checks
(`monge_break`, `dnc_break`, `knuth_break`, `convexity_break`). Design,
laws and worked problems: `design/stdlib/dp.md`.

```cp
// sliding-window optimum: std.window.MonoDeque (4.8)
struct Line(k: Int, m: Int)
    fn at(self, x: Int) -> Int                // k * x + m
struct LiChao<L, K>                           // convex hull trick, any L.at -> K
    fn new(pick: (K, K) -> K, xs: Range<Int>) -> LiChao<L, K>
    fn add(mut self, l: L, on: ?Range<Int> = none) // O(log n), O(log^2 n) on a segment
    fn at(self, x: Int) -> ?K                 // O(log n)
    fn line_at(self, x: Int) -> ?L            // O(log n)
    fn clear(mut self)
fn monotone_minima(rows: Int, cols: Int, f: (Int, Int) -> ?Int)
    -> List<?(val: Int, at: Int)>             // O((rows + cols) log rows) calls
fn dnc_layer(prev: List<?Int>, cost: (Int, Int) -> Int) -> List<?Int>
    // O(n log n) calls; next[i] = min over j < i of prev[j] + cost(j, i)
fn knuth(n: Int, cost: (Int, Int) -> Int) -> List<List<Int>> // O(n^2)
fn aliens(k: Int, lambdas: Range<Int>, solve: Int -> (val: Int, count: Int))
    -> Int                                    // O(log) calls of solve
fn subset_sums(xs: List<Int>) -> List<Int>    // O(2^n), sorted
// sums over subsets / supersets: std.ntt.sos(mut f, over: Sub | Sup = Sub, inverse: Bool = false) (4.5)
```

### 4.17 `std.derive`: field-wise traits

The `Eq`, `Ord` and `Show` a struct gets field by field, written on
`fields` (decision 28.7), for an `impl` to call:
`impl Ord for Job: fn lt(a, b) = lt_fields(b, a)` orders the other way.

```cp
fn eq_fields<T>(a: T, b: T) -> Bool // O(fields)
fn lt_fields<T>(a: T, b: T) -> Bool // O(fields)
fn show_fields<T>(a: T) -> Str // O(fields)
```

## 5. Not here yet

Power-series algebra beyond products, suffix automata, link-cut trees,
persistent structures, wavelet trees, 2-SAT and the rest of `examples/local-lib/` are staged there, not
excluded: they arrive in the order the ports need them, after the [new]
items above. Until then they are the standing test that a user library
gets the stdlib's powers.
