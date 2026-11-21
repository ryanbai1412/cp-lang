# Range structures (`std.algebra`, `std.prefix`, `std.fenwick`, `std.segtree`, `std.lazy`, `std.sparse`, `std.range`)

Status: design for section 4.10 of `plan/STDLIB.md`. Covers every
range-query and range-update pattern from competitive programming, from
O(1) prefix sums to lazy/dual segment trees, persistent trees, Li Chao
trees, segment-tree beats, and offline sweeps.

## 1. Contest pattern survey

Every range structure answers a point on the trade-off surface between
update generality, query generality, algebraic requirements, and constant
factor. Surveying four primary sources fixes the exact patterns the stdlib
must cover cleanly.

**AtCoder Library (ACL)** distills 80% of contest range work into three
structures: `fenwick_tree` (point add, range sum in O(log n)), `segtree`
(point set, range monoid fold, `max_right`, `min_left` in O(log n)), and
`lazy_segtree` (range action apply, range monoid fold, `max_right`,
`min_left` in O(log n)). ACL's biggest win is the `max_right` / `min_left`
binary search directly on the tree, eliminating O(log^2 n) binary searches
around `fold`. Its biggest friction in C++ is boilerplate: every tree
requires a separate `S op(S, S)`, `S e()`, `S mapping(F, S)`,
`F composition(F, F)`, `F id()`, plus a fake monoid whenever you only want
range-apply and point-get.

**KACTL** covers the next 15%: `FenwickTree` (with `lower_bound`),
`FenwickTree2d`, `RMQ` (sparse table, O(1) idempotent range query),
`SegmentTree` (iterative 2n), `LazySegmentTree` (recursive with range
set/add), `LineContainer` (dynamic convex hull trick), and `Mo` (Hilbert
or alternating-block query ordering). KACTL keeps each template tiny by
hard-coding `+` or `max` and asking contestants to edit the template in
place for custom nodes.

**ecnerwala's library (`proto/ports/ecnerwala/*_orig.cpp`)** separates
tree topology from node payloads via `seg_tree::in_order_layout` and
`circular_layout`, exposing `for_each`, `for_each_l_to_r`,
`for_parents_down`, and `for_parents_up`. In the corpus this pattern
appears in `cf2147f` (range add + minimum-cover gap sum), `cf2255e2`
(dual segment tree over history-recording tags), `cf2207f` (DP range add +
range max), `cf2165f` (point set + range max over values), `cf2262b`
(prefix-max maintenance), and `cf2135f` (persistent hashed multiset
segment tree). The layout abstraction gives C++ zero-overhead inlining and
lets a dual segment tree skip `pull` entirely, at the cost of ~250 lines
of pasted layout code and manual node-index arithmetic.

**CP Lang corpus frictions (`research/frictions/`)** pinpoint six exact
gaps between today's `stdlib/` and `design/EXEMPLARS-PORTS.md`:

- **Dual segment trees need a fake monoid (`cf2255e2.md`).** Range-apply
  plus point-get has no fold at all, yet `LazySegTree` forces a dummy
  `over: Monoid(id: blank, op: (c, d) => c)` and wastes `2 log n` pulls
  per update.
- **Max-subarray summary is re-derived (`cf2255e2.md`).** The 4-field
  Kadane monoid (`best`, `total`, `pre`, `suf`) with `?Seg` identity and
  non-negative scaling `* k` is the most repeated custom monoid.
- **Constant-initialized trees allocate throwaway arrays (`cf2207f.md`).**
  `LazySegTree<Pos, Int>(AddMax, [Pos] _ => 0)` allocates a temporary
  array just to fill leaves with `0` instead of `Int.MIN`; a named
  `fill:` option removes both the allocation and the noise.
- **Difference Fenwicks and prefix/suffix monoid folds are hand-rolled
  (`cf2062e2.md`).** Range-add / point-get on a Fenwick repeats the
  right-endpoint `if let after = ...` guard, and prefix/suffix monoid
  tables over `t.Pre` manually index `[0..=n]` with `.ord` and `.ord + 1`
  instead of returning `[D.bounds]T`.
- **Persistent hashed multisets and dynamic trees need raw arenas
  (`cf2135f.md`).** Hand-rolling `pool: List<Node>` with sentinel `0` and
  forged `Int` handles breaks index safety and adds 30 lines of ceremony.
- **Closure indirection and missing niche packing hurt hot loops
  (`cf2147f.md`, `cf2165f.md`, `cf2255e2.md`, `cf2084f.md`).** Storing
  `Monoid.op` and `Action.{apply,compose}` as `std::function` causes
  1.6x-4.4x slowdowns; this is a core-lane codegen optimization, not an
  API compromise.

**Remaining ICPC / AtCoder / CF Div. 1 patterns** complete the picture:
static non-idempotent O(1) queries (disjoint sparse table), O(1) amortized
sliding-window folds (`MonoDeque` for min/max, `TwoStackFold` for
inverse-free monoids like `gcd` or matrix product), coordinate-free
dynamic segment trees (`SparseSegTree` over `lo..hi` up to `10^18`),
persistent segment trees (`PersistentSegTree`, `RangeKth`), merge-sort
trees (`MergeSortTree` for static range order counts), Li Chao trees
(`LiChao`, `LiChaoTag` with line and segment insertion), segment-tree
beats (`ChminSegTree` / `BeatsSegTree` for range `chmin`/`chmax`/`add`),
and offline sweeps (`mo_order` / `mo_walk` for Mo's algorithm, and
offline time-interval divide-and-conquer for rollback structures as in
`cf2069f`).

## 2. Design principles and vocabulary

One naming scheme governs every structure in this area so a contestant can
guess any method from its neighbours:

- **Point access:** `t.get(i)` reads position `i`; `t.set(i, x)`
  overwrites it; `t.update(i, f)` replaces it with `f(cur)` in one pass;
  `t.add(i, x)` increments it (additive structures); `t.flip(i, x)` xors
  it (`XorFenwick`).
- **Range query:** `t.fold(r)` folds range `r` under the structure's
  monoid or semigroup; `t.sum(r)` is used where the operation is addition
  and nothing else makes sense (`Prefix`,
  `Fenwick`, `Fenwick2D`, `RangeFenwick`, `ChminSegTree`); `t.xor(r)` on
  `XorFenwick`; `t.all` is an O(1) `@get` property for the whole domain.
- **Range update:** `t.apply(r, f)` applies lazy tag `f` to every
  position in `r` (`LazySegTree`, `DualSegTree`); `t.add(r, x)` adds `x`
  to `r` (`DualFenwick`, `RangeFenwick`, `Diff`).
- **Tree binary search:** `t.max_right(from, ok)` returns the furthest
  `Bounds<D>` right of `from` with `ok(fold(from..b))`; `t.min_left(to,
  ok)` returns the furthest `Bounds<D>` left of `to` with
  `ok(fold(b..to))`. `Fenwick` keeps `lower_bound(target) -> ?Bounds<D>`
  and `kth(k) -> ?D` (1-based) for prefix-sum descent.
- **Options are named arguments, never `_by` or `_filled` variants.**
  Constructors accept `a: ?[D]T = none` and `fill: ?T = none` so
  `LazySegTree<Pos, Int>(AddMax, fill: 0)` and
  `SegTree<V, Int>(max, a)` share one constructor.
- **Every function documents its time and memory cost.** O(1) vs O(log n)
  vs amortized O(log^2 n) vs O(n log n) is part of the contract.
- **Nothing hidden under the hood.** `Prefix` is static O(1), `Fenwick`
  is O(log n) group sum, `SparseTable` requires idempotence,
  `DisjointSparseTable` does not, `DualSegTree` skips pulls, `LiChao` is
  dynamic pointer-arena over `lo..=hi`. The type name chooses the
  algorithm and memory layout.

## 3. Coherent API by module

### 3.1 `std.algebra` — operation values and stock monoids

Algebra stays value-based (`Monoid<T>`, `Action<T, F>`) per decision 26.4
until user traits gain associated constants. Bare operators (`+`, `*`,
`^`, `|`, `&`, `min`, `max`, `gcd`) convert automatically via
`Monoid<T>.of(op)` with exact identities (`none` for `min`/`max` over
`?T`, never a magic `INF`).

```cp
// Existing core algebra
struct Monoid<T>(id: T, op: (T, T) -> T)
    law assoc(a: T, b: T, c: T) = op(op(a, b), c) == op(a, op(b, c))
    law ident(a: T) = op(id, a) == a and op(a, id) == a
    fn of<M>(m: M) -> Monoid<T>                     // O(1)

struct Action<T, F>(id: F, compose: (F, F) -> F,
                    apply: (F, T, Int) -> T, over: Monoid<T>)
    law ident(a: T) = apply(id, a, 1) == a
    law composes(f: F, g: F, a: T) =
        apply(compose(f, g), a, 1) == apply(f, apply(g, a, 1), 1)
    fn new<M>(id: F, compose: (F, F) -> F,
              apply: (F, T, Int) -> T, over: M) -> Action<T, F>  // O(1)

struct Desc<T>(x: T)
fn desc<T>(x: T) -> Desc<T>                         // O(1)
trait Reversible: fn rev(self) -> Self = self
fn fold<T, M>(m: M, xs: List<T>) -> T               // O(n)
```

Addition to `std.algebra`: **`MaxSub`**, the Kadane maximum-subarray
monoid (closing `cf2255e2.md` and `EXEMPLARS-PORTS.md`). Storing
`empty: Bool` avoids wrapping every node in `?MaxSub` while keeping
`MaxSub.empty` as a true identity. All operations are O(1):

```cp
/// Maximum non-empty subarray summary (Kadane monoid).
/// Concatenation `+` is associative with identity `MaxSub.empty`;
/// `s * k` scales every value by non-negative `k`.
/// `best` traps if empty; `try_best` returns `?Int`.
struct MaxSub(has: Bool, best: Int, total: Int, pre: Int, suf: Int)
    const empty: MaxSub                             // O(1), identity
    fn of(v: Int) -> MaxSub                         // O(1), single element
    fn join(a: MaxSub, b: MaxSub) -> MaxSub         // O(1), also `a + b`
    fn scale(self, k: Int) -> MaxSub                // O(1), k >= 0; also `self * k`
    @get fn try_best(self) -> ?Int                  // O(1)
    fn monoid() -> Monoid<MaxSub>                   // O(1)
    // impl Reversible: swaps `pre` and `suf` for tree-path folds!
```

Implementing `Reversible` on `MaxSub` makes HLD / `Rooted.fold_path`
maximum-subarray queries on trees work out of the box with zero extra
code.

### 3.2 `std.prefix` — static prefix, suffix, and difference tables

Static O(1) queries after O(n) or O(r * c) preprocessing, indexed by
`Bounds<D>` so off-by-one cut shifts cannot compile.

```cp
// Unified shape-dispatched Prefix family (constructed via `prefix(a)`)
struct Prefix<D, T>                                 // 1-D over [D]T
    fn new(a: [D]T) -> Prefix<D, T>                 // O(|D|)
    fn prefix(self, b: Bounds<D>) -> T              // O(1)
    fn sum(self, r: Range<D>) -> T                  // O(1)

struct Prefix<R, C, T>                              // 2-D over [R, C]T
    fn new(a: [R, C]T) -> Prefix<R, C, T>           // O(|R| * |C|)
    fn prefix(self, r: Bounds<R>, c: Bounds<C>) -> T // O(1)
    fn sum(self, rows: Range<R>, cols: Range<C>) -> T // O(1)

struct Prefix<T>                                    // List<T> / [0..n]T
    fn new(a: List<T>) -> Prefix<T>                 // O(n)
    fn prefix(self, k: Int) -> T                    // O(1)
    fn sum(self, r: Range<Int>) -> T                // O(1)

fn prefix(a)                                        // dispatches on [D]T, [R, C]T, or List<T>
```

Additions to `std.prefix`:

- **`prefix_fold` and `suffix_fold`** (closing `cf2062e2.md`): return
  `[D.bounds]T` where `pref[b]` is the fold of `..b` and `suff[b]` is the
  fold of `b..`, with `pref[D.start] == m.id` and `suff[D.end] == m.id`.
  Given any range `r: Range<D>`, the fold of everything *outside* `r` is
  simply `m.op(pref[r.start], suff[r.end])` — zero `.ord + 1` arithmetic.
- **`Diff<D, T>` and `Diff<R, C, T>`**: offline range-add difference
  arrays that accumulate O(1) `add(...)` calls and materialize `[D]T` or
  `[R, C]T` in one O(n) / O(r * c) `finish()` pass (eliminating the
  manual right-end boundary checks in problems like `cf2084e`).

```cp
fn prefix_fold<D, T, M>(op: M, a: [D]T) -> [D.bounds]T  // O(n)
fn suffix_fold<D, T, M>(op: M, a: [D]T) -> [D.bounds]T  // O(n)

struct Diff<D, T>
    fn new() -> Diff<D, T>                          // O(|D|)
    fn add(mut self, r: Range<D>, x: T)             // O(1)
    fn finish(self) -> [D]T                         // O(|D|)

struct Diff<R, C, T>
    fn new() -> Diff<R, C, T>                       // O(|R| * |C|)
    fn add(mut self, rows: Range<R>, cols: Range<C>, x: T)  // O(1)
    fn finish(self) -> [R, C]T                      // O(|R| * |C|)
```

### 3.3 `std.fenwick` — Fenwick trees (point, dual, range, xor, 2D)

Fenwick trees require an abelian group (`+` and `-`, or `^`). They are
2-4x faster and use half the memory of `SegTree` when addition or xor
suffices.

```cp
// Existing
struct Fenwick<D, T>                                // point add, range sum
    fn new(a: ?[D]T = none) -> Fenwick<D, T>        // O(n)
    fn add(mut self, i: D, x: T)                    // O(log n)
    fn prefix(self, b: Bounds<D>) -> T              // O(log n)
    fn sum(self, r: Range<D>) -> T                  // O(log n)
    fn first_ge(self, target: Int) -> ?Bounds<D> // O(log n), non-negative
    fn kth(self, k: Int) -> ?D                      // O(log n), 1-based

struct XorFenwick<D, T>                             // point xor, range xor
    fn new(a: ?[D]T = none) -> XorFenwick<D, T>     // O(n)
    fn flip(mut self, i: D, x: T)                   // O(log n)
    fn prefix(self, b: Bounds<D>) -> T              // O(log n)
    fn xor(self, r: Range<D>) -> T                  // O(log n)

struct Fenwick2D<R, C, T>                           // point add, rect sum
    fn new() -> Fenwick2D<R, C, T>                  // O(R * C)
    fn add(mut self, r: R, c: C, x: T)              // O(log R * log C)
    fn sum(self, rows: Range<R>, cols: Range<C>) -> T  // O(log R * log C)
```

Additions to `std.fenwick`:

- **`DualFenwick<D, T>`** (closing `cf2062e2.md`): range-add and
  point-get via a difference Fenwick. Making this a separate type rather
  than adding `range_add` onto `Fenwick` enforces the invariant: you can
  never accidentally call `sum(r)` on a difference tree.
- **`RangeFenwick<D, T>`**: range-add and range-sum over `Int` using two
  Fenwick arrays (`d[i]` and `i * d[i]`), O(log n) with ~2n memory and
  far smaller constant factor than `LazySegTree<D, Int>(AddSum)`.

```cp
struct DualFenwick<D, T>                            // range add, point get
    fn new(a: ?[D]T = none) -> DualFenwick<D, T>    // O(n)
    fn add(mut self, r: Range<D>, x: T)             // O(log n)
    fn get(self, i: D) -> T                         // O(log n)

struct RangeFenwick<D>                              // range add, range sum (Int)
    fn new(a: ?[D]Int = none) -> RangeFenwick<D>    // O(n)
    fn add(mut self, r: Range<D>, x: Int)           // O(log n)
    fn prefix(self, b: Bounds<D>) -> Int            // O(log n)
    fn sum(self, r: Range<D>) -> Int                // O(log n)
```

### 3.4 `std.segtree` — iterative point-update segment tree

Iterative bottom-up segment tree of size `2 * bit_ceil(n)`, keeping
left-to-right fold order for non-commutative monoids.

```cp
struct SegTree<D, T>(m: Monoid<T>)
    fn new<M>(op: M, a: ?[D]T = none, fill: ?T = none) -> SegTree<D, T>  // O(n)
    fn of<M>(op: M, f: D -> T) -> SegTree<D, T>     // O(n)
    fn refresh(mut self, i: D)                      // O(log n)
    fn get(self, i: D) -> T                         // O(1)
    fn set(mut self, i: D, x: T)                    // O(log n)
    fn update(mut self, i: D, f: T -> T)            // O(log n)
    fn fold(self, r: Range<D>) -> T                 // O(log n)
    @get fn all(self) -> T                          // O(1)
    @get fn monoid(self) -> Monoid<T>               // O(1)
    fn max_right(self, from: Bounds<D>, ok: T -> Bool) -> Bounds<D>  // O(log n)
    fn min_left(self, to: Bounds<D>, ok: T -> Bool) -> Bounds<D>     // O(log n)
```

Law for `max_right(from, ok)`: `ok(m.id)` must hold, and
`ok(fold(from..b))` must be monotone (true then false) as `b` increases.
Symmetric law for `min_left(to, ok)`. Adding `fill: ?T = none` lets a
contestant write `SegTree<V, Int>(max, fill: 0)` without allocating a
throwaway `[V]Int` array (if both `a` and `fill` are given, `new` traps).

### 3.5 `std.lazy` — lazy segment tree and dual segment tree

`LazySegTree` handles range-apply + range-fold; **`DualSegTree`** handles
range-apply + point-get with no monoid, no `pull`, and no `span` array
(closing `cf2255e2.md` and matching `EXEMPLARS-PORTS.md`).

```cp
struct LazySegTree<D, T, F>(act: Action<T, F>)
    fn new(act: Action<T, F>, a: ?[D]T = none,
           fill: ?T = none) -> LazySegTree<D, T, F> // O(n)
    fn get(mut self, i: D) -> T                     // O(log n)
    fn set(mut self, i: D, x: T)                    // O(log n)
    fn update(mut self, i: D, f: T -> T)            // O(log n)
    fn fold(mut self, r: Range<D>) -> T             // O(log n)
    fn apply(mut self, r: Range<D>, f: F)           // O(log n)
    @get fn all(self) -> T                          // O(1)
    @get fn monoid(self) -> Monoid<T>               // O(1)
    fn max_right(mut self, from: Bounds<D>, ok: T -> Bool) -> Bounds<D>  // O(log n)
    fn min_left(mut self, to: Bounds<D>, ok: T -> Bool) -> Bounds<D>     // O(log n)

/// Range apply, point get — no fold monoid, no pulls, no span array.
/// `compose(new_f, old_f)` applies `old_f` first, then `new_f`.
/// Memory: `bit_ceil(n)` leaves + `bit_ceil(n)` internal tags.
struct DualSegTree<D, T, F>
    fn new(id: F, compose: (F, F) -> F, apply: (F, T) -> T,
           a: ?[D]T = none, fill: ?T = none) -> DualSegTree<D, T, F>  // O(n)
    fn apply(mut self, r: Range<D>, f: F)           // O(log n)
    fn get(mut self, i: D) -> T                     // O(log n)
    fn set(mut self, i: D, x: T)                    // O(log n)
    fn update(mut self, i: D, f: T -> T)            // O(log n)
```

Stock actions in `std.lazy`:

```cp
const AddSum: Action<Int, Int>                      // + over +
const AddMin: Action<Int, Int>                      // + over min
const AddMax: Action<Int, Int>                      // + over max
const AssignSum: Action<Int, ?Int>                  // assign over +
const AssignMin: Action<Int, ?Int>                  // assign over min
const AssignMax: Action<Int, ?Int>                  // assign over max
const AffineSum: Action<Int, (Int, Int)>            // (mul, add) over +
fn affine_sum<T>() -> Action<T, (T, T)>             // generic (Int, Mod<P>)
```

Note on **non-pushing / commutative dual trees**: when `compose` is
commutative (e.g., range `max=` or range `+=`), a dual tree does not even
need top-down `push` — `apply` composes onto O(log n) canonical nodes and
`get` composes up the root path. `DualSegTree` pushes top-down by default
so non-commutative actions (like `assign` and `cf2255e2`'s history tags)
are always sound without a trap; commutative range-add belongs to
`DualFenwick`.

### 3.6 `std.sparse` — O(1) static range tables

```cp
// Existing: O(n log n) build, O(1) query; op must be associative and idempotent
struct SparseTable<D, T>(op: (T, T) -> T)
    fn new(op: (T, T) -> T, a: [D]T) -> SparseTable<D, T>  // O(n log n)
    fn fold(self, r: Range<D>) -> T                 // O(1); asserts non-empty
fn idempotent<T>(op: (T, T) -> T, xs: List<T>) -> Bool     // O(n)

// New: O(n log n) build, O(1) query; any associative semigroup (no idempotence!)
struct DisjointSparseTable<D, T>(op: (T, T) -> T)
    fn new(op: (T, T) -> T, a: [D]T) -> DisjointSparseTable<D, T>  // O(n log n)
    fn fold(self, r: Range<D>) -> T                 // O(1); asserts non-empty
```

`DisjointSparseTable` divides the array recursively at midpoints, storing
suffix folds left of `mid` and prefix folds right of `mid` at each level.
Any query `[lo, hi)` of length > 1 splits cleanly at the unique level
`bit_width(lo ^ (hi - 1)) - 1`, answering non-idempotent static queries
(`Mod<P>` products, `Mat<T>` products, `MaxSub`) in O(1) without needing
inverses.

### 3.7 `std.window` — O(1) amortized sliding-window folds

Promoted from `examples/local-lib/ds/mono_deque.cp` and
`two_stack_fold.cp`:

```cp
/// Sliding-window min or max over positions pushed in increasing order.
struct MonoDeque<T>
    fn new(pick: (T, T) -> T) -> MonoDeque<T>       // O(1); min or max
    fn by(better: (T, T) -> Bool) -> MonoDeque<T>   // O(1); custom strict order
    @get fn len(self) -> Int                        // O(1)
    @get fn is_empty(self) -> Bool                  // O(1)
    @get fn best(self) -> ?T                        // O(1)
    fn clear(mut self)                              // O(1)
    fn push(mut self, at: Int, val: T)              // amortized O(1)
    fn drop_before(mut self, min_at: Int)           // amortized O(1)

/// FIFO queue supporting O(1) monoid fold of all elements without inverses.
struct WindowFold<T>(m: Monoid<T>)
    fn new<M>(op: M) -> WindowFold<T>               // O(1)
    @get fn len(self) -> Int                        // O(1)
    @get fn is_empty(self) -> Bool                  // O(1)
    @get fn fold(self) -> T                         // O(1)
    fn push(mut self, x: T)                         // O(1)
    fn pop(mut self) -> ?T                          // amortized O(1)
```

### 3.8 Advanced range structures (`std.ranges` / dedicated modules)

These cover the remaining Div. 1 / ICPC range structures with the same
domain-safe, cost-documented conventions.

**Merge-sort tree (`std.mergesort` or `std.range`):** static `[D]T`,
O(n log n) build and memory, O(log^2 n) queries for counting or locating
values inside a range:

```cp
struct MergeSortTree<D, T>
    fn new(a: [D]T) -> MergeSortTree<D, T>          // O(n log n)
    fn count_lt(self, r: Range<D>, x: T) -> Int     // O(log^2 n)
    fn count_le(self, r: Range<D>, x: T) -> Int     // O(log^2 n)
    fn count_in(self, r: Range<D>, lo: T, hi: T) -> Int  // O(log^2 n), [lo, hi)
    fn first_ge(self, r: Range<D>, x: T) -> ?T      // O(log^2 n)
```

**Dynamic / sparse segment tree (`SparseSegTree<T>`):** coordinate-free
point-update / range-fold over an integer interval `lo..hi` (up to
`10^18`), creating O(log W) nodes per update (`W = hi - lo`):

```cp
struct SparseSegTree<T>(m: Monoid<T>, lo: Int, hi: Int)
    fn new<M>(op: M, lo: Int, hi: Int) -> SparseSegTree<T>  // O(1)
    fn set(mut self, x: Int, v: T)                  // O(log W)
    fn update(mut self, x: Int, f: T -> T)          // O(log W)
    fn fold(self, r: Range<Int>) -> T               // O(log W)
    @get fn nodes(self) -> Int                      // O(1)
```

**Persistent segment tree & range k-th (`std.persistent`):** uses a
nominal `Version` handle so version IDs can never be confused with array
indices or counts (closing `cf2135f.md` and promoting
`examples/local-lib/ds/range_kth.cp`):

```cp
struct Version(id: Int)

struct PersistentSegTree<D, T>(m: Monoid<T>)
    fn new<M>(op: M, a: ?[D]T = none) -> (PersistentSegTree<D, T>, Version)  // O(n)
    fn set(mut self, v: Version, i: D, x: T) -> Version     // O(log n) time & nodes
    fn update(mut self, v: Version, i: D, f: T -> T) -> Version  // O(log n)
    fn get(self, v: Version, i: D) -> T             // O(log n)
    fn fold(self, v: Version, r: Range<D>) -> T     // O(log n)

struct RangeKth<D>                                  // static range order stats
    fn new(a: [D]Int) -> RangeKth<D>                // O(n log n)
    fn kth(self, r: Range<D>, k: Int) -> ?Int       // O(log n), 0-based
    fn count_lt(self, r: Range<D>, x: Int) -> Int   // O(log n)
```

**Li Chao tree (`std.lichao`):** promotes `examples/local-lib/ds/li_chao.cp`
and adds segment insertion (`add_segment`, O(log^2 W)) and optional
`max` mode:

```cp
struct Line(k: Int, m: Int)
    fn at(self, x: Int) -> Int                      // O(1)

struct LiChao(lo: Int, hi: Int)
    fn new(lo: Int, hi: Int) -> LiChao              // O(1)
    fn add(mut self, line: Line)                    // O(log W)
    fn add_segment(mut self, r: Range<Int>, line: Line)  // O(log^2 W)
    fn min_at(self, x: Int) -> ?Int                 // O(log W)

struct TaggedLine<V>(k: Int, m: Int, tag: V)
struct LiChaoTag<V>(lo: Int, hi: Int)
    fn new(lo: Int, hi: Int) -> LiChaoTag<V>        // O(1)
    fn add(mut self, line: TaggedLine<V>)           // O(log W)
    fn add_segment(mut self, r: Range<Int>, line: TaggedLine<V>)  // O(log^2 W)
    fn min_at(self, x: Int) -> ?(Int, V)            // O(log W)
```

**Segment-tree beats (`std.beats`):** promotes and generalizes
`examples/local-lib/ds/chmin_segtree.cp` to full Ji Driver beats
(`chmin`, `chmax`, `add` with `sum`, `min`, `max`), amortized O(log^2 n)
per operation, O(n) memory:

```cp
struct BeatsSegTree<D>
    fn new(a: [D]Int) -> BeatsSegTree<D>            // O(n)
    fn chmin(mut self, r: Range<D>, x: Int)         // amortized O(log^2 n)
    fn chmax(mut self, r: Range<D>, x: Int)         // amortized O(log^2 n)
    fn add(mut self, r: Range<D>, x: Int)           // amortized O(log^2 n)
    fn sum(mut self, r: Range<D>) -> Int            // O(log n)
    fn min_in(mut self, r: Range<D>) -> ?Int        // O(log n)
    fn max_in(mut self, r: Range<D>) -> ?Int        // O(log n)
```

**Offline sweeps & Mo's algorithm (`std.mo`):** promotes
`examples/local-lib/ds/mo.cp` into domain-typed `Range<D>` queries and
adds an offline time-interval segment tree (`TimeTree<Q, E>`) for dynamic
connectivity / rollback DSU problems (`cf2069f` pattern):

```cp
/// Ordering of offline range queries in O((n + q) sqrt n) window moves.
/// Returns query indices in traversal order.
fn mo_order<D>(queries: List<Range<D>>) -> List<Int>         // O(q log q)

/// Generator yielding `(pos: D, entering: Bool)` moves from `from` to `to`.
/// Expands before shrinking so the window is never inverted.
fn mo_steps<D>(from: Range<D>, to: Range<D>) -> yields (pos: D, entering: Bool)
                                                             // O(|dl| + |dr|)

/// Offline divide-and-conquer over time `0..q`: attach events active on
/// `r: Range<T>` and walk leaves in O((q + e) log q) push/undo calls.
struct TimeTree<T, E>
    fn new() -> TimeTree<T, E>                               // O(q)
    fn add(mut self, r: Range<T>, event: E)                  // O(log q)
    fn visit(self, enter: E -> (), leave: E -> (), at: T -> ())  // O((q + e) log q)
```

## 4. Laws, domains, and core-lane coordination

### 4.1 Algebraic laws

Every structure documents the exact law its operation must satisfy, and
`cpl test` checks them with `laws_hold` / `idempotent`:

- `SegTree`, `DisjointSparseTable`, `WindowFold`, `SparseSegTree`,
  `PersistentSegTree`: associative `op` (and identity `id` where empty
  ranges are allowed).
- `SparseTable`: associative and idempotent `op` (`op(x, x) == x`).
- `Fenwick`, `DualFenwick`, `RangeFenwick`, `Prefix`: abelian
  group (`+`, `-`, `zero`); `XorFenwick`: self-inverse `^`.
- `LazySegTree`: `Action` identity, composition order
  `apply(compose(f, g), x) == apply(f, apply(g, x))`, and distributivity
  `apply(f, op(a, b)) == op(apply(f, a), apply(f, b))`.
- `DualSegTree`: `Action` identity and composition order (no
  distributivity needed because there is no fold).
- `max_right` / `min_left`: `ok(m.id)` is true and `ok` switches from
  true to false at most once.

### 4.2 Domains and index spaces

Every fixed-size range structure is parameterized by a domain `D` (or
`R, C` in 2D) and accepts `Range<D>` (`l..=r`, `l..r`, `..i`, `i..`, `..`)
and `Bounds<D>` (`i.l`, `i.r`, `D.start`, `D.end`). Passing an index from
a different domain or an off-by-one raw `Int` is a compile-time error.
Coordinate-free structures (`SparseSegTree`, `LiChao`) explicitly take
`lo..hi` / `lo..=hi` over `Int` because their coordinates are dynamic and
up to `10^18`. Persistent trees return nominal `Version` handles rather
than raw `Int` node indices.

### 4.3 Core-lane dependencies (no compiler privilege)

All structures above are ordinary CP Lang library code. Four core-lane
improvements (already tracked in `research/frictions/` and `design/DELTA.md`)
make them zero-overhead and even cleaner without any special-casing for
range structures:

1. **Monomorphizing compile-time closures in generic structs**
   (`cf2147f.md`, `cf2165f.md`, `cf2255e2.md`, `cf2084f.md`): eliminates
   `std::function` indirect calls inside `SegTree`, `LazySegTree`, and
   `DualSegTree`, recovering the 1.6x-4.4x speed gap against hand-rolled
   C++.
2. **Niche packing `?Struct` and `const&` record parameters**
   (`cf2147f.md`, `cf2255e2.md`): eliminates `std::optional` padding and
   by-value tag copies in lazy/dual trees.
3. **`Bounds<D>` as a first-class generic `Index` domain and
   `Bounds<D>..Bounds<D>` intervals** (`cf2147f.md`, `cf2207f.md`,
   `cf2165f.md`): lets `LazySegTree<Bounds<Val>, ...>` work directly
   without a parallel `domain Cut = 0..=n`, and lets `fold(a[j].r..a[i].l)`
   express exclusive-start intervals without `(a[j] + 1)!`.
4. **Writable places / `Index` trait** (`design/spec/27`): lets `st[i]`
   read a leaf and `st[i] = x` write it alongside `.get`/`.set`.

## 5. What stays user code

A stdlib that tries to absorb every contest trick becomes unguessable.
The following stay user code, built cleanly on top of the stdlib:

- **Problem-specific monoids and actions** (10-15 lines): `Low` in
  `cf2147f`, `Tag`/`Path` in `cf2255e2`, affine + parity counts, bracket
  matching. The stdlib provides the tree and stock actions; the user
  writes only the problem's algebra.
- **One-off beats invariants beyond `chmin`/`chmax`/`add`**: range
  `%= x`, range `sqrt`, range bitwise AND/OR beats, or historic max/min.
  Each has problem-specific pruning conditions and is cleaner as a
  35-line recursive function than a 10-parameter generic framework.
- **Ordinary sweep-line event loops**: sorting events and scanning with a
  `Fenwick` or `LazySegTree` is 5 lines of ordinary `for` loop. Only
  non-trivial offline scheduling (`mo_order` / `mo_steps`, `TimeTree`)
  belongs in the stdlib.
- **CDQ divide-and-conquer and 2D compressed Fenwick trees**: CDQ is a
  12-line recursive function coupled to the problem's 3D partial order;
  offline 2D compressed Fenwick is problem-specific point pre-collection.

## 6. Real problems written against this API

Five real contest problems written at the level of
`design/EXEMPLARS-PORTS.md`, exercising every layer of the design.

### 6.1 CF 2147F — Exchange Queries (`LazySegTree` + custom monoid)

```cp
// Cut c (values <= c vs > c) is free when no item straddles it; free cuts
// split items into SCCs reaching all lower ones, giving n(n+1)/2 +
// sum d(d-1)/2 over gaps d between consecutive free cuts. Item i covers
// cuts min(p,s)..max(p,s): maintain the minimum-cover cuts and gap sum.
@!multitest
let (n, q): (2..=100_000, 1..=100_000) = read()
domain Item = 1..=n
domain Cut = 0..=n
var p: [Item]Cut = read()
var s: [Item]Cut = read()

struct Low(cover: Int, first: Cut, last: Cut, gaps: Int)

fn merge(a: ?Low, b: ?Low) -> ?Low:
    let a = a ?? return b
    let b = b ?? return a
    if a.cover < b.cover: return a
    if b.cover < a.cover: return b
    let d = b.first.to_idx - a.last.to_idx
    Low(a.cover, a.first, b.last, a.gaps + b.gaps + d * (d - 1) / 2)

let act = Action<?Low, Int>(
    id: 0,
    compose: +,
    apply: (d, x, _) => if let v = x: Low(v.cover + d, v.first, v.last, v.gaps) else: none,
    over: Monoid(id: none, op: merge),
)
var low = LazySegTree<Cut, ?Low>(act, [Cut] c => Low(0, c, c, 0))

fn cover(i: Item, d: Int):
    let (lo, hi) = (min(p[i], s[i]), max(p[i], s[i]))
    if lo < hi: low.apply(lo..hi, d)

for i in Item: cover(i, 1)
let base = n * (n + 1) / 2
for _ in 0..q:
    let (t, i, j): (1..=2, Item, Item) = read()
    cover(i, -1)
    cover(j, -1)
    if t == 1: p.swap_at(i, j) else: s.swap_at(i, j)
    cover(i, 1)
    cover(j, 1)
    print(base + low.all!.gaps)
```

### 6.2 CF 2255E2 — What Will Remain at the End? (`DualSegTree` + `MaxSub`)

No fake monoid, no `over`, no hand-rolled Kadane tuple:

```cp
// Every version appends each position's value to its history; a query is
// the best subarray of one history. Negate and clamp depend only on sign
// and scale with |val|, so tags compose in O(1) inside one DualSegTree.
use std.algebra.{MaxSub}
use std.lazy.{DualSegTree}

type Sign = -1..=1
fn sgn(v: Int) -> Sign = if v < 0: -1 elif v == 0: 0 else: 1

struct Cell(val: Int, hist: MaxSub)
struct Path(to: Sign, seen: MaxSub)
struct Tag(neg: Path, zero: Path, pos: Path, set: ?Cell)

fn path(f: Tag, s: Sign) -> Path = if s < 0: f.neg elif s == 0: f.zero else: f.pos
fn tag(to: Sign -> Sign, seen: Sign -> MaxSub = _ => MaxSub.empty, set: ?Cell = none) -> Tag =
    Tag(Path(to(-1), seen(-1)), Path(to(0), seen(0)), Path(to(1), seen(1)), set)

fn run(f: Tag, c: Cell) -> Cell:
    let p = path(f, sgn(c.val))
    let hist = c.hist + p.seen * abs(c.val)
    if let s = f.set: Cell(s.val, hist + s.hist) else: Cell(abs(c.val) * p.to, hist)

fn then(g: Tag, f: Tag) -> Tag:
    if let s = f.set: return Tag(f.neg, f.zero, f.pos, run(g, s))
    fn via(p: Path) -> Path = Path(path(g, p.to).to, p.seen + path(g, p.to).seen)
    Tag(via(f.neg), via(f.zero), via(f.pos), g.set)

let id_tag = tag(s => s)
let record = tag(s => s, s => MaxSub.of(s))
let negate = tag(s => -s)
let clamp = tag(s => max(s, 0))
fn assign(x: Int) -> Tag = tag(s => s, set: Cell(x, MaxSub.empty))

@!multitest
let (n, q): (1..=500_000, 1..=500_000) = read()
domain I = 1..=n
let a: [I]-1_000_000_000..=1_000_000_000 = read()
var st = DualSegTree<I, Cell, Tag>(
    id: id_tag,
    compose: then,
    apply: run,
    a: [I] i => Cell(a[i], MaxSub.of(a[i])),
)
var last: U64 = 0
fn at(y: U64) -> I = I(Int((y ^ last) % U64(n)) + 1)
for _ in 0..q:
    match read::<1..=4>():
        4 =>
            let ans = st.get(at(read())).hist.best
            print(ans)
            last = U64(ans)
        kind =>
            let (x, y) = (at(read()), at(read()))
            let f = if kind == 1: assign(read()) elif kind == 2: negate else: clamp
            st.apply(min(x, y)..=max(x, y), f)
    st.apply(.., record)
```

### 6.3 CF 2062E2 — The Game, Hard (`prefix_fold` + `suffix_fold` + `DualFenwick`)

`prefix_fold` / `suffix_fold` index by `Bounds<t.Pre>` directly, and
`DualFenwick.add(t.sub(v), 1)` eliminates the manual right-end guard:

```cp
// Root at 1 and flatten by preorder. v wins iff max weight outside sub(v)
// exceeds w[v], and every u with w[u] > w[v] lies in sub(v), attains that
// outside max, or is a path-prefix-max ancestor covering all outside maxes.
use std.fenwick.{DualFenwick, Fenwick}
use std.prefix.{prefix_fold, suffix_fold}

// ... after rooting t = Rooted(g, V(1)) and computing path_max / prev_max:
let m = Monoid<MaxInfo>(empty_max, combine)
let pref = prefix_fold(m, [t.Pre] p => MaxInfo(w[t.at(p)], 1, p, p))
let suff = suffix_fold(m, [t.Pre] p => MaxInfo(w[t.at(p)], 1, p, p))

var sub_bit = Fenwick<t.Pre, Int>()
var anc_bit = DualFenwick<t.Pre, Int>()
var winners: List<V> = []
var seen = 0

for bucket in by_weight.rev:
    for v in bucket:
        let r = t.sub(v)
        let out = combine(pref[r.lo], suff[r.hi])
        if out.w > w[v]:
            let covered = sub_bit.sum(r) + out.cnt + anc_bit.get(out.lo)
            if covered == seen: winners.push(v)
    for v in bucket:
        sub_bit.add(t.pos(v), 1)
        if is_path_max[v]: anc_bit.add(t.sub(v), 1)
        seen += 1
```

### 6.4 CF 2207F — Hanabi (`LazySegTree` with `fill: 0`)

No temporary array allocation to initialize a DP segment tree to zero:

```cp
// Sweep ranks r left to right; dp[pos] is the best score ending at cut pos.
domain Pos = 0..=n
var seg = LazySegTree<Pos, Int>(AddMax, fill: 0)
for r in Rank:
    let best = seg.all + Int(not must_rank[r])
    seg.set(Pos(r.to_idx + 1), best)
    for lo in by_right[r]: seg.apply(..=Pos(lo.to_idx), 1)
print(seg.all)
```

### 6.5 CF 86D — Powerful Array (`mo_order` + `mo_steps`)

Offline frequency-squared queries over `Range<I>` with zero raw-index
window bookkeeping:

```cp
// Maintain sum(cnt[x]^2 * x) as one window walks across queries in Mo order.
use std.mo.{mo_order, mo_steps}

let (n, q): (1..=200_000, 1..=200_000) = read()
domain I = 1..=n
domain Q = 0..q
let a: [I]1..=1_000_000 = read()
let queries = [Q] _ => (let (l, r): (I, I) = read(); l..=r)

var cnt: [0..=1_000_000]Int = 0
var cur = 0
var ans: [Q]Int = 0
var win = I.start..I.start

for qi in mo_order([queries[i] for i in Q]):
    let target = queries[Q(qi)]
    for (pos, entering) in mo_steps(win, target):
        let x = a[pos]
        cur -= cnt[x] * cnt[x] * x
        cnt[x] += if entering: 1 else: -1
        cur += cnt[x] * cnt[x] * x
    win = target
    ans[Q(qi)] = cur

for i in Q: print(ans[i])
```

## 7. Rollout and maximally ambitious next step

**Phase 1 (ship now in `stdlib/`):**
- `std.algebra`: `MaxSub` (with `Reversible`, `+`, `*`, `of`, `empty`,
  `monoid`).
- `std.prefix`: `prefix_fold`, `suffix_fold`, `Diff`.
- `std.fenwick`: `DualFenwick`, `RangeFenwick`.
- `std.segtree`: `fill: ?T = none` on `SegTree.new`.
- `std.lazy`: `DualSegTree`, `AffineSum`, `fill: ?T = none` on
  `LazySegTree.new`.
- `std.sparse`: `DisjointSparseTable`.
- `std.window`: `MonoDeque`, `WindowFold` (promoted from
  `examples/local-lib/ds/`).
- `std.mo`: `mo_order`, `mo_steps` (domain-typed promotion of
  `examples/local-lib/ds/mo.cp`).

**Phase 2 (advanced range structures in `stdlib/`):**
- `MergeSortTree`, `SparseSegTree`, `PersistentSegTree` + `RangeKth`,
  `LiChao` + `LiChaoTag` (with `add_segment`), `BeatsSegTree` (full
  `chmin` + `chmax` + `add`), and `TimeTree` (offline rollback D&C).

**Maximally ambitious version:** once the core lane delivers compile-time
closure monomorphization, `?Struct` niche packing, and `Bounds<D>` as an
`Index` domain, every range structure in `stdlib/` matches hand-rolled C++
within 1.1x runtime and 1.0x memory while guaranteeing domain safety,
checked algebraic laws via `cpl test`, and zero-boilerplate custom nodes.
