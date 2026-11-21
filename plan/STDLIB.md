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

**Options are named arguments, not `_by` variants.** `runs(a, by: key)`,
`sort(mut a, by: key)`, `bsearch_first(r, ok)`. Today's `runs_by`,
`first_ge_by` and `last_le_by` are [rename]s to `runs(a, by:)`,
`first_ge(xs, x, by:)` and `last_le(xs, x, by:)`.

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
`sorted`, `swap`, `min`, `max`, `abs`, `sum`. The 0.3 spec adds
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

Free functions over `List<T>` and `[D]T`, the sorted-lookup family, the
compressed coordinate space, and the binary searches. The searches
return the boundary, not a found flag: `bsearch_first(lo..hi, ok)` is
the first `i` with `ok(i)` for a monotone `ok`, or `none`.

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

[new] Search on answers over reals and over unimodal functions, the two
shapes ports hand-roll most after the integer search:

```cp
fn bsearch_real(lo: F64, hi: F64, ok: F64 -> Bool, steps: Int = 100) -> F64
    // O(steps) calls of ok; the smallest x in [lo, hi] with ok(x) to
    // within (hi - lo) / 2^steps, hi when none
fn ternary_max(r: Range<Int>, f: Int -> Int) -> Int
    // O(log n) calls of f; a position of the maximum of f on r for f
    // that strictly rises then falls (plateaus only at the top)
```

[rename] `runs_by(a, key)` becomes `runs(a, by: key)`;
`first_ge_by` and `last_le_by` become `first_ge(.., by:)` and
`last_le(.., by:)`.

### 4.3 `std.math`: number theory

Integer helpers and number theory with every cost in the name or the
type. Three ways to factor, from cheapest setup to largest input:
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
fn bit_len(x: Int) -> Int
fn digits(self: Int, base: Int
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
fn Sieve.phi(self, x: Int) -> Int // O(log x)
fn Sieve.mobius(self, x: Int) -> Int // O(log x)
```

Promoted from `examples/local-lib/math/primes.cp` and `floor_sum.cp`,
which now keep only what builds on them (`divisors_of`, `mod_sum`,
`count_residues`). Ports: `proto/ports/scout_batch_16/cco05p4.cp`
(`is_prime`), `proto/ports/scout_batch_43/etsr23r4p2.cp` (`phi`).
Planned: `Sieve.primes`, a linear sieve of any multiplicative function.

Relies on: `I128` (inside `mul_mod`), the builtin `isqrt`.

### 4.4 `std.mod`: modular arithmetic

`Mod<P>` is an integer reduced mod `P`, where `P` is a constant or a
value read at startup; `Comb<P>` grows factorial tables on demand. The
0.3 spec's `Mint(mod: p)` and `Z.factorials` are the same ideas; we keep
the shipped names, which 135 and 37 port files use (decision 26.2).

```cp
struct Mod<P>(v: Int) // O(1), O(log P)
    fn new(x: Int) -> Mod<P>
    fn pow(self, e: Int) -> Mod<P> // O(log e)
    fn try_inv(self) -> ?Mod<P> // O(log P)
    @get fn inv(self) -> Mod<P>
struct Comb<P> // O(1)
    fn new(limit: Int
    fn fact(self, n: Int) -> Mod<P>
    fn inv_fact(self, n: Int) -> Mod<P>
    fn inv(self, i: Int) -> Mod<P> // O(1)
    fn choose(self, n: Int, k: Int) -> Mod<P>
    fn arrange(self, n: Int, k: Int) -> Mod<P>
    fn multichoose(self, n: Int, k: Int) -> Mod<P>
    fn catalan(self, n: Int) -> Mod<P>
```

[new] `x.try_inv() -> ?Mod<P>`, O(log P), for moduli that are not prime.

### 4.5 `std.ntt` [new]: convolution

The number-theoretic transform and the products built on it, promoted
from `examples/local-lib/math/poly.cp` (imported as `lib.math.poly` by
nine port files) and made generic over the modulus. The power-series
operations stay in the local library until ports other than the
polynomial batch reach for them.

```cp
fn ntt<P>(mut a: List<Mod<P>>, inverse: Bool = false)
    // O(n log n) in place; a.len a power of two dividing P - 1
fn convolve<P>(a: List<Mod<P>>, b: List<Mod<P>>) -> List<Mod<P>>
    // O((n + m) log(n + m)); P - 1 divisible by a power of two at least
    // n + m - 1 (998244353, 167772161, 469762049, 754974721 qualify);
    // schoolbook below 64 terms
fn convolve_mod(a: List<Int>, b: List<Int>, m: Int) -> List<Int>
    // three NTT primes and CRT; any m < 2^31, coefficients in 0..m
fn convolve_int(a: List<Int>, b: List<Int>) -> List<Int>
    // exact when every output fits in about 2^62; traps otherwise
```

Relies on: `Mod<P>` with a type-parameter modulus, `primitive_root`,
`bit_ceil`, `ctz`.

### 4.6 `std.matrix` [new]: matrices

`Mat<T>` is a dense row-major matrix over any `T` with `T.zero`,
`T.one`, `+`, `-`, `*` (`Int`, `Mod<P>`, `F64`); elimination needs `/`
and is offered where the element type divides exactly (`Mod<P>` with
prime `P`, `F64` with partial pivoting).

```cp
struct Mat<T>(n: Int, m: Int, a: List<T>)
fn Mat.zero(n: Int, m: Int = n) -> Mat<T>          // O(n m)
fn Mat.identity(n: Int) -> Mat<T>                  // O(n^2)
fn Mat.of(rows: List<List<T>>) -> Mat<T>           // O(n m)
fn at(self, i: Int, j: Int) lends mut T            // O(1)
fn mul(self, o: Mat<T>) -> Mat<T>                  // O(n m k), the `*`
fn add(self, o: Mat<T>) -> Mat<T>                  // O(n m), the `+`
fn pow(self, e: Int) -> Mat<T>                     // O(n^3 log e)
fn apply(self, v: List<T>) -> List<T>              // O(n m)
fn transpose(self) -> Mat<T>                       // O(n m)
fn rows(self) -> List<List<T>>                     // O(n m)
fn det(self) -> T                                  // O(n^3), T: Div
fn rank(self) -> Int                               // O(n m min(n, m))
fn solve(self, b: List<T>) -> ?List<T>             // O(n m min(n, m))
fn inverse(self) -> ?Mat<T>                        // O(n^3)
fn Mat.product(n: Int) -> Monoid<Mat<T>>           // for segment trees
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

[new]:

```cp
fn first(self) -> ?D            // O(n/64)
fn last(self) -> ?D             // O(n/64)
fn prev(self, i: D) -> ?D       // O(n/64); the last member < i
fn clear(mut self)              // O(n/64)
fn shift_up(mut self, k: Int)   // O(n/64) in place; bits past the end drop
fn count_below(self, i: D) -> Int   // O(n/64)
```

### 4.9 Containers: `std.bag`, `std.deque`, `std.heap`, `std.dsu`, `std.marks`

`SortedBag<T>` is an ordered multiset with order statistics,
`Deque<T>` a ring buffer, `MinHeap`/`MaxHeap` binary heaps (the heap is
chosen by name, not by a comparator flag), `Dsu<V>` union-find by size
with path compression, and `Marks<D>` an O(1)-clear visited set by
epoch.

```cp
struct SortedBag<K> // O(log n), O(1), O(n)
    fn new() -> SortedBag<K>
    fn of(xs: List<K>) -> SortedBag<K> // O(n log n)
    @get fn len(self) -> Int // O(1)
    @get fn is_empty(self) -> Bool
    fn clear(mut self)
    fn take(mut self) -> SortedBag<K> // O(1)
    fn add(mut self, x: K) -> Bool // O(log n)
    fn remove(mut self, x: K) -> Bool // O(log n)
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
    fn nth(self, i: Int) -> ?K // O(log n)
    fn to_list(self) -> List<K> // O(n)
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
struct MinHeap<T> // O(log n), O(1)
    fn new() -> MinHeap<T>
    @get fn len(self) -> Int
    @get fn is_empty(self) -> Bool
    @get fn peek(self) -> ?T
    fn clear(mut self)
    fn take(mut self) -> MinHeap<T> // O(1)
    fn push(mut self, x: T)
    fn pop(mut self) -> ?T
struct MaxHeap<T>
    fn new() -> MaxHeap<T>
    @get fn len(self) -> Int
    @get fn is_empty(self) -> Bool
    @get fn peek(self) -> ?T
    fn clear(mut self)
    fn take(mut self) -> MaxHeap<T> // O(1)
    fn push(mut self, x: T)
    fn pop(mut self) -> ?T
struct Dsu<V> // O(alpha(n)), O(log n)
    fn new() -> Dsu<V> // O(n)
    fn find(mut self, v: V) -> V
    fn mark(mut self) -> Int // O(1)
    fn undo_to(mut self, snap: Int) // O(1)
    fn unite(mut self, a: V, b: V) -> Bool
    fn merge(mut self, a: V, b: V) -> ?(kept: V, merged: V)
    fn same(mut self, a: V, b: V) -> Bool
    fn size(mut self, v: V) -> Int
    @get fn components(self) -> Int
    fn groups(mut self) -> List<List<V>> // O(n alpha(n))
struct Marks<D> // O(1)
    fn new() -> Marks<D> // O(D.K), O(1)
    fn add(mut self, v: D) -> Bool // O(1)
    fn remove(mut self, v: D) -> Bool // O(1)
    fn contains(self, v: D) -> Bool // O(1)
    fn clear(mut self) // O(1)
    @get fn len(self) -> Int // O(1)
    @get fn is_empty(self) -> Bool
```

### 4.10 Range structures

Modules `std.algebra`, `std.fenwick`, `std.prefix`, `std.segtree`,
`std.lazy` and `std.sparse`.

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
struct Fenwick<D, T> // O(n), O(log n)
    fn new(a: ?[D]T
    fn add(mut self, i: D, x: T)
    fn prefix(self, b: Bounds<D>) -> T
    fn sum(self, r: Range<D>) -> T
    fn lower_bound(self, target: Int) -> ?Bounds<D> // O(log n)
    fn kth(self, k: Int) -> ?D // O(log n)
struct XorFenwick<D, T> // O(n), O(log n)
    fn new(a: ?[D]T
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
struct SegTree<D, T>(m: Monoid<T>) // O(log n)
    fn new<M>(op: M, a: ?[D]T // O(n)
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
    fn new(act: Action<T, F>, a: ?[D]T // O(n)
    fn set(mut self, i: D, x: T)
    fn update(mut self, i: D, f: T -> T) // O(log n)
    fn get(mut self, i: D) -> T
    @get fn all(self) -> T // O(1)
    @get fn monoid(self) -> Monoid<T>
    fn fold(mut self, r: Range<D>) -> T
    fn apply(mut self, r: Range<D>, f: F)
    fn max_right(mut self, from: Bounds<D>, ok: T -> Bool) -> Bounds<D>
    fn min_left(mut self, to: Bounds<D>, ok: T -> Bool) -> Bounds<D>
const AddSum = Action<Int, Int>(0, +, (f, x, len) => x + f * len, over: +)
const AddMin = Action<Int, Int>(0, +, (f, x, len) => x + f, over: min)
const AddMax = Action<Int, Int>(0, +, (f, x, len) => x + f, over: max)
const AssignSum: Action<Int, ?Int>   // assign over sum
const AssignMin: Action<Int, ?Int>   // assign over min
const AssignMax: Action<Int, ?Int>   // assign over max
struct SparseTable<D, T>(op: (T, T) -> T) // O(n log n), O(1)
    fn new(op: (T, T) -> T, a: [D]T) -> SparseTable<D, T>
    fn fold(self, r: Range<D>) -> T // O(1)
fn idempotent<T>(op: (T, T) -> T, xs: List<T>) -> Bool
```

### 4.11 `std.graph`, `std.tree`, `std.tour`: graphs and trees

`Graph<V>` is a compressed adjacency list over a vertex domain, built by
`undirected`/`directed`; `Rooted<V>` a rooted tree with parents, depths,
Euler tour, LCA by binary lifting and subtree ranges.

```cp
type Tree<V> = [0..V.len - 1](u: V, v: V)
trait Neighbours
trait Target
struct Graph<V, E> // O(n + m)
    fn directed(edges: [E](u: V, v: V)) -> Graph<V, E>
    fn undirected(edges: [E](u: V, v: V)) -> Graph<V, E>
    fn adj(self, u: V) -> yields (to: V, e: E)
    fn out(self, u: V) -> yields (to: V, e: E)
    fn get(self, u: V) -> List<(to: V, e: E)>
    fn deg(self, u: V) -> Int // O(1)
    fn transpose(self) -> Graph<V, E> // O(n + m)
fn bfs<V, N>(next: N, from: List<V>) -> [V]?Int // O(n + m)
// O((n + m) log m)
fn dijkstra<V, N, A, W>(next: N, from: List<(V, W)>, w: A -> W) -> [V]?W
fn peel<V, N>(next: N) -> List<V> // O(n + m)
fn topo<V, N>(next: N) -> ?List<V> // O(n + m)
fn scc<V, N>(next: N) -> (domain C, comp: [V]C) // O(n + m)
fn directed_of_pairs<V>(edges: List<(V, V)>) -> [V]List<V>
fn undirected_of_pairs<V>(edges: List<(V, V)>) -> [V]List<V>
fn directed_of_triples<V, W>(edges: List<(V, V, W)>) -> [V]List<(V, W)>
fn undirected_of_triples<V, W>(edges: List<(V, V, W)>) -> [V]List<(V, W)>
fn components<V, N>(next: N) -> (domain C, comp: [V]C) // O(n + m)
fn condense<V, N, C>(next: N, comp: [V]C) -> [C]List<(to: C)>
fn two_edge_components<V, N>(next: N) -> (domain C, comp: [V]C) // O(n + m)
fn articulation_points<V, N>(next: N) -> [V]Bool // O(n + m)
fn biconnected_components<V, N>(next: N) -> List<List<V>>
fn two_color<V, N>(next: N) -> ?[V]Bool // O(n + m)
struct Rooted<V, Pre> // O(n)
    fn new<N>(adj: N, root: V)
    fn from_parents(par: [V]?V) // O(n)
    fn weighted<N, A>(adj: N, root: V, len: A -> Int)
    fn pos(self, v: V) -> Pre // O(1)
    fn parent(self, v: V) -> ?V
    fn reorder<T>(self, a: [V]T) -> [Pre]T // O(n)
    fn children(self, v: V) -> List<V>
    // O(k)
    fn excl_fold<T, M>(self, v: V, m: M, f: V -> T) -> yields (c: V, rest: T)
    fn depth(self, v: V) -> Int
    fn size(self, v: V) -> Int
    @get fn subtree_sizes(self) -> [V]Int
    @get fn centroid(self) -> V // O(depth)
    @get fn centroids(self) -> List<V> // O(depth)
    @get fn preorder(self) -> List<V>
    @get fn postorder(self) -> List<V>
    @get fn tour(self) -> yields (v: V, entering: Bool) // O(n)
    fn sub(self, v: V) -> Range<Pre>
    fn is_anc(self, a: V, b: V) -> Bool // O(1)
    fn lower(self, u: V, v: V) -> V // O(1)
    fn lca(self, a: V, b: V) -> V // O(log n)
    fn dist(self, a: V, b: V) -> Int // O(log n)
    fn path(self, a: V, b: V) -> List<V> // O(|path|)
    fn diameter(self) -> (len: Int, u: V, v: V) // O(n)
    fn ancestor(self, v: V, k: Int) -> ?V // O(log n)
    fn step(self, a: V, b: V, k: Int) -> ?V // O(log n)
    fn lift_while(self, v: V, pred: V -> Bool) -> ?V // O(log^2 n)
    fn chains( // O(log n)
    fn fold_path<S, T>(self, a: V, b: V, ds: S, edges: Bool // O(log^2 n)
    // O(log^2 n)
    fn apply_path<S, F>(self, a: V, b: V, mut ds: S, tag: F, edges: Bool
    // O(k log k + k log n)
    fn virtual_tree(self, keys: List<V>)
        -> (nodes: List<V>, edges: List<(u: V, v: V)>)
```

[new], promoted from `examples/local-lib/graph/`:

```cp
fn mst<V>(edges: List<(u: V, v: V, w: Int)>) -> (total: Int, used: List<Int>)
    // O(m log m) Kruskal; indices of the chosen edges
```

### 4.12 `std.flow`: flows and matchings

Dinic's max flow on `FlowGraph<V>` with arcs as handles, the min cut,
and the assignment problem.

```cp
struct Arc(id: Int)
    @get fn rev(self) -> Arc
struct FlowGraph<V> // O(1), O(V^2 E), O(E sqrt(V))
    fn new() -> FlowGraph<V>
    fn add(mut self, u: V, v: V, cap: ?Int) -> Arc
    fn add_both(mut self, u: V, v: V, cap: Int) -> Arc
    fn flow(self, a: Arc) -> Int
    fn cap(self, a: Arc) -> Int
    fn target(self, a: Arc) -> V
    fn max_flow(mut self, s: V, t: V) -> Int
    fn min_cut(self, s: V) -> [V]Bool
    fn reset(mut self) // O(E)
// O(n^3)
fn max_weight_matching<L, R>(weight: (L, R) -> Int) -> (total: Int, mate: [L]R)
```

[new], promoted from `examples/local-lib/graph/matching.cp` and
`min_cost_flow.cp`:

```cp
fn max_matching<L, R>(adj: [L]List<R>) -> (size: Int, mate: [L]?R)
    // O(m sqrt n) Hopcroft-Karp
fn min_vertex_cover<L, R>(adj: [L]List<R>) -> (left: [L]Bool, right: [R]Bool)
    // O(m sqrt n); Konig's theorem from the matching
struct CostFlow<V>
fn CostFlow.new() -> CostFlow<V>
fn add(mut self, u: V, v: V, cap: Int, cost: Int) -> Arc     // O(1)
fn flow(mut self, s: V, t: V, limit: ?Int = none) -> (flow: Int, cost: Int)
    // O(F m log n) successive shortest paths with potentials;
    // costs may be negative on the first pass (Bellman-Ford start)
```

### 4.13 `std.strings`: strings

ASCII byte strings, the prefix and Z functions, suffix array and LCP,
and the polynomial rolling hash with two random bases.

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

[new], promoted from `examples/local-lib/strings/`:

```cp
fn manacher(s: Str) -> (odd: List<Int>, even: List<Int>)   // O(n)
fn is_palindrome(s: Str) -> Bool                           // O(n)
struct AhoCorasick
fn AhoCorasick.new(patterns: List<Str>) -> AhoCorasick     // O(sum |p| * 26)
fn matches(self, text: Str) -> yields (at: Int, pattern: Int)
    // O(|text| + output)
fn counts(self, text: Str) -> List<Int>                    // O(|text| + nodes)
```

### 4.14 `std.geometry`: integer geometry

Exact integer points; anything with an angle or a length that is not an
integer is F64 and says so in its type.

```cp
struct Pt(x: Int, y: Int)
    @get fn norm2(self) -> Int
    @get fn len(self) -> F64
    @get fn perp(self) -> Pt
fn dot(a: Pt, b: Pt) -> Int
fn cross(a: Pt, b: Pt) -> Int
fn turn(a: Pt, b: Pt, c: Pt) -> Int
struct ByAngle(p: Pt)
fn on_segment(p: Pt, a: Pt, b: Pt) -> Bool
fn segments_meet(a: Pt, b: Pt, c: Pt, d: Pt) -> Bool
fn area2(poly: List<Pt>) -> Int // O(n)
fn locate(poly: List<Pt>, p: Pt) -> Int // O(n)
```

[new], promoted from `examples/local-lib/geometry/convex_hull.cp`:

```cp
fn convex_hull(pts: List<Pt>) -> List<Pt>    // O(n log n) monotone chain,
                                             // counter-clockwise, no collinear
fn diameter2(hull: List<Pt>) -> Int          // O(n) rotating calipers
fn closest_pair2(pts: List<Pt>) -> Int       // O(n log n)
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
fn step<R, C, D>(self: (R, C), d: D) -> ?(R, C)
fn ray<R, C, D>(self: (R, C), d: D) -> List<(R, C)>
@get fn adj4<R, C>(self: (R, C)) -> List<(R, C)>
@get fn adj8<R, C>(self: (R, C)) -> List<(R, C)>
@get fn transposed<R, C, T>(self: [R, C]T) -> [C, R]T
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

## 5. Not here yet

Power-series algebra beyond products, suffix automata, link-cut trees,
persistent structures, wavelet trees, heavy-light decomposition, Li Chao
trees, 2-SAT and the rest of `examples/local-lib/` are staged there, not
excluded: they arrive in the order the ports need them, after the [new]
items above. Until then they are the standing test that a user library
gets the stdlib's powers.
