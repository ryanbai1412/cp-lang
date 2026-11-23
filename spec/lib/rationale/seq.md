# `std.seq`: Sequences, Search, Scans, Permutations, and Compression

Design for section 4.2 of `plan/STDLIB.md`. Everything here is ordinary CP Lang
library code with no compiler privilege; where a spelling waits on a general
core feature (mutable slice lending, three-way `cmp:` on `sort`, variadic
`min`/`max`), this document says so explicitly instead of special-casing.

---

## 1. Survey of contest patterns

Every facility in `std.seq` comes from recurring patterns across Codeforces
Div. 1, AtCoder AGC/ARC/ABC, and ICPC World Finals solutions, cross-checked
against KACTL, AtCoder Library, and an empirical audit of two local corpora:

1. **279 accepted C++ solutions in `proto/ports/ecnerwala/*_orig.cpp`**
   (counting `main()` only, excluding pasted library headers): `std::swap` (60),
   `std::reverse` (39), `std::sort` (32), `std::accumulate` (19), `std::iota`
   (13), `std::rotate` (12), `std::max_element` (9), `std::is_sorted` (8),
   `std::find` (6), `std::next_permutation` (6), `std::lower_bound` (5),
   `std::nth_element` (5), `std::min_element` (5), `std::unique` (3),
   `std::merge` (1).
2. **51 sequence-related friction reports in `research/frictions/`**, which
   exposed six recurring gaps in the initial `stdlib/seq.cp`:
   - `runs` only accepted `List<T>` (`cf2084h`, `cf2069e`) and had no
     adjacency-predicate split (`chunk_by`) for alternating runs.
   - `runs_by` allocated a `List<T>` per run and ran 50x slower than a manual
     loop on `cf2135f` ($n = 4 \cdot 10^5$, 18.6 s vs 0.35 s).
   - No `prefix_fold` / `suffix_fold` returning `[D.cuts]T` (`cf2062e2`),
     forcing parallel `[0..=n]` arrays indexed by `.ord` and `.ord + 1`.
   - No `prod` reduction matching `sum` (`cf2201e`), and no `buckets` /
     `group_by` helper mapping a domain `D` into `[K]List<D>` (`cf2062e1`,
     `cf2062e2`).
   - Hand-written permutation inversion (`cf2187f1`, `cf2084g2`), cycle
     decomposition, and nearest-smaller/larger monotone-stack loops with `-1`/`n`
     sentinels.
   - Missing `bsearch_real`, `ternary_max`/`ternary_min`, `first_gt`/`last_lt`
     on sequences and `Compressed`, `rotate_right`, and O(1)-space in-place
     `rotate_left` (`cf2207h3`; the old `rotate_left` copied the whole list).

Across KACTL, AtCoder Library, and Codeforces editorials, eight cohesive
sequence families cover virtually all contest work:

1. Reductions and predicates (folds, extrema, arg-extrema, sortedness).
2. Sorting, selection (`nth_element`), merging, and in-place reordering.
3. Runs, chunking, domain bucketing, windows, pairs, and zipping.
4. Prefix, suffix, and exclusive scans over elements and boundary cuts.
5. Coordinate compression with its own fresh domain.
6. Permutations (lexicographic stepping, inversion, composition, cycles,
   powers, inversion count).
7. Binary search on answers (integers and reals), unimodal (ternary) search,
   and the four-name sorted-lookup family.
8. Fixed-window sliding extrema and nearest-smaller/larger monotone-stack
   queries.

---

## 2. Design principles and domain integration

**One polymorphism: `self: [D]T`.** In CP Lang, a function parameterized as
`fn f<D, T>(self: [D]T, ...)` accepts domain arrays `[V]T`, ranged arrays
`[1..=n]T`, `List<T>` (with `D = Int`), and `Str` (with `D = Int, T = Char`)
with one definition and zero runtime dispatch. Sequence algorithms take
`self: [D]T` whenever they read by index, and return `?D`, `Range<D>`, `[D]?D`,
or `[D.cuts]T` so index spaces are never downgraded to raw `Int`.

**Options are named arguments, never `_by` proliferation.** Following
`plan/STDLIB.md` section 2, key extractors use `by: T -> K = x => x` with
default type parameter `K = T`:

```cp
xs.argmax()                     // ?D by element order
xs.argmax(by: p => p.score)     // ?D by key
xs.first_ge(x)                  // ?D in sorted order
xs.first_ge(k, by: p => p.key)  // ?D by key
```

Existing `_by` names (`min_by`, `max_by`, `runs_by`, `first_ge_by`,
`last_le_by`) stay working as deprecated aliases until a codemod migrates all
ports.

**Zero-allocation generators by default.** Anything that walks subranges or
neighbours (`chunk_by`, `run_ranges`, `pairs`, `with_prev`, `cyclic_pairs`,
`excl_fold`) is a `yields` generator. In particular, `chunk_by` and
`run_ranges` yield `Range<D>` rather than copied `List<T>` slices, so a caller
gets `r.len` in O(1), `a[r.first!]` in O(1), and `for i in r:` with zero
allocations, and only copies if it explicitly writes `a[r]`.

**Costs are visible.** In-place mutators take `mut a` (`sort`, `reverse`,
`dedup`, `rotate_left`, `rotate_right`, `nth_element`, `next_permutation`,
`prev_permutation`) and use O(1) extra space. Value-returning twins are past
participles (`sorted`, `reversed`, `deduped`, `resized`) and clearly document
their O(n) allocation.

**Sentinels are banned.** Extrema on possibly-empty inputs return `?T` or `?D`;
sorted lookups return `?D`; nearest-smaller/larger tables return `[D]?D`;
binary searches return `?I`.

---

## 3. Complete API surface

### 3.1 Reductions and predicates

Builtins already provide `sum`, `min`, `max`, `all`, `any`, `count`, `first`,
and `xs.find(x)` / `xs.find(pred)` (with zero-allocation comprehension forms
`all(p(x) for x in xs)`, `any(p(x) for x in xs)`, `count(p(x) for x in xs)`,
`first(x for x in xs if p(x))`). `std.seq` completes the family:

```cp
// Multiplicative reduction; returns T.one on empty. O(n) time, O(1) space.
pub fn prod<S, T>(it: S) -> T

// True iff by(self[i]) <= by(self[j]) for all adjacent i < j.
// O(n) time, O(1) space.
pub fn is_sorted<D, T, K = T>(self: [D]T, by: T -> K = x => x) -> Bool

// Index of the first maximum / minimum under `by`, or `None` when empty.
// O(n) time, O(1) space. (Also @get argmax / argmin when called without args.)
@get pub fn argmax<D, T>(self: [D]T) -> ?D
@get pub fn argmin<D, T>(self: [D]T) -> ?D
pub fn argmax_by<D, T, K>(self: [D]T, by: T -> K) -> ?D  // [target: argmax(by:)]
pub fn argmin_by<D, T, K>(self: [D]T, by: T -> K) -> ?D  // [target: argmin(by:)]

// Element with smallest / largest key (first on ties), or `None`.
// O(n) time, O(1) space.
pub fn min_by<S, T, K>(it: S, key: T -> K) -> ?T  // [target: min(it, by:)]
pub fn max_by<S, T, K>(it: S, key: T -> K) -> ?T  // [target: max(it, by:)]
```

Laws:
- `prod([]) == T.one` and `prod(a + b) == prod(a) * prod(b)`.
- `is_sorted(a)` holds iff `a == sorted(a)` (for `List<T>`).
- When `i = a.argmax!`, every `j < i` has `a[j] < a[i]` and every `j` has
  `a[j] <= a[i]` (first extremum wins ties).

### 3.2 Sorting, selection, merging, and in-place reordering

Builtins provide `sort(mut xs, by:, desc:)` and `sorted(xs, by:, desc:)`
(O(n log n)); `std.algebra.desc(x)` reverses individual tuple fields in `by:`.
`std.seq` adds:

```cp
// Partition `a` in place so a[0..k] <= a[k] <= a[k+1..].
// Expected O(n) time (deterministic median-of-three quickselect), O(1) space.
// Traps if k is outside 0..a.len.
pub fn nth_element<T>(mut a: List<T>, k: Int)

// Merge two sorted lists into one sorted list, stably.
// O(a.len + b.len) time and space.
pub fn merge_sorted<T>(a: List<T>, b: List<T>) -> List<T>

// Reverse `a` in place. O(n) time, O(1) space.
pub fn reverse<T>(mut a: List<T>)

// Drop consecutive equal values in place (a sorted list keeps unique values).
// O(n) time, O(1) space.
pub fn dedup<T>(mut a: List<T>)

// Value form of `dedup`. O(n) time and space.
pub fn deduped<T>(self: List<T>) -> List<T>

// Stable split into elements satisfying `keep` and the rest.
// O(n) time and space.
pub fn partition<T>(self: List<T>, keep: T -> Bool) -> (List<T>, List<T>)

// Rotate `a` left by `k` in place (3-reversal algorithm). Euclidean mod `k`:
// negative `k` rotates right. O(n) time, O(1) space.
pub fn rotate_left<T>(mut a: List<T>, k: Int)

// Rotate `a` right by `k` in place: `rotate_left(mut a, -k)`.
// O(n) time, O(1) space.
pub fn rotate_right<T>(mut a: List<T>, k: Int)

// Swap two positions of a domain array in place; safe when i == j. O(1).
pub fn swap_at<D, T>(mut self: [D]T, i: D, j: D)

// Truncate or pad `self` to length `n` with `fill`. O(n) time and space.
pub fn resized<T>(self: List<T>, n: Int, fill: T) -> List<T>

// Move elements out of `self` leaving it empty. O(1) time (O(n) until move-take).
pub fn take<T>(mut self: List<T>) -> List<T>

// Insert/remove at index `i`, shifting the tail. O(n) time, O(1) extra space.
pub fn insert<T>(mut self: List<T>, i: Int, x: T)
pub fn remove<T>(mut self: List<T>, i: Int) -> T
```

Laws:
- `nth_element(mut a, k)` preserves the multiset of `a`, `a[k] == sorted(a)[k]`,
  `all(a[i] <= a[k] for i in 0..k)`, and `all(a[i] >= a[k] for i in k+1..a.len)`.
- `merge_sorted(sorted(a), sorted(b)) == sorted(a + b)`.
- `rotate_left(mut a, k)` followed by `rotate_right(mut a, k)` restores `a`.
- `reverse(mut a)` twice restores `a`.

### 3.3 Runs, chunking, domain bucketing, windows, pairs, and zip

Note that `.items` is CP Lang's uniform `enumerate` across `List<T>`, `[D]T`,
`Str`, and `Map<K, V>`: `for (i, x) in a.items:` gives typed `(D, T)` on
`[D]T` and `(Int, T)` on `List<T>`.

```cp
// Maximal contiguous ranges where adjacent elements satisfy `same(prev, cur)`.
// Zero allocations; works on List<T>, [D]T, and Str. O(n) total, O(1) space.
pub fn chunk_by<D, T>(self: [D]T, same: (T, T) -> Bool) -> yields Range<D>

// Maximal contiguous ranges of equal `by(x)` keys.
// Zero allocations; replaces `runs_by` in hot loops (50x faster on cf2135f).
// O(n) total time, O(1) extra space.
pub fn run_ranges<D, T, K = T>(self: [D]T, by: T -> K = x => x) -> yields Range<D>

// Run-length encoding of equal neighbours: `s.runs()` works on Str, List<T>,
// and [D]T alike. O(n) time and O(runs) space.
pub fn runs<D, T>(self: [D]T) -> List<(val: T, len: Int)>

// [deprecated alias, kept for existing ports] Materialized keyed runs.
pub fn runs_by<T, K>(a: List<T>, by: T -> K) -> List<(key: K, items: List<T>)>

// Bucket every key `i` of `self` under its value `self[i]` in domain `K`,
// preserving key order within each bucket. O(D.count + K.count) time/space.
// Decision 158: this is the one domain-keyed grouping (`preimages` folded in).
pub fn buckets<D, K: Domain>(self: [D]K) -> [K]List<D>

// Contiguous length-`k` windows; O(k) per yielded window.
pub fn windows<T>(self: List<T>, k: Int) -> yields List<T>

// Non-overlapping length-`k` chunks (the last may be shorter). O(n) total.
pub fn chunks<T>(self: List<T>, k: Int) -> yields List<T>

// Adjacent pairs `(a[i-1], a[i])`; empty when len < 2. O(1) per step.
pub fn pairs<D, T>(self: [D]T) -> yields (T, T)

// Every element with the previous one, starting with `(None, first)`. O(1)/step.
pub fn with_prev<D, T>(self: [D]T) -> yields (?T, T)

// Adjacent pairs wrapping last to first. O(1) per step.
pub fn cyclic_pairs<D, T>(self: [D]T) -> yields (T, T)

// Pairwise `(a[i], b[i])` up to `min(a.len, b.len)`. O(min(n, m)) time/space.
pub fn zip<A, B>(a: List<A>, b: List<B>) -> List<(A, B)>
```

Laws:
- The ranges yielded by `a.chunk_by(same)` partition `D` into non-empty
  contiguous ranges; inside every range `r`, `same(a[i], a[j])` holds for
  adjacent `i, j in r`, and fails across every boundary between consecutive
  ranges.
- `a.run_ranges(by: f)` is identical to `a.chunk_by((x, y) => f(x) == f(y))`.
- `sum(r.len for r in a.runs()) == a.len` and no two adjacent entries share
  `.val`.
- For `b = key.buckets()`, every `i in D` appears in `b[key[i]]` exactly once,
  in increasing `D` order.

### 3.4 Prefix, suffix, and exclusive scans

Static invertible range sums (`+`/`-`) live in `std.prefix` (`prefix(a)` /
`Prefix`). General semigroup/monoid scans live here:

```cp
// Inclusive prefix scan (no identity needed): out[i] = f(out[prev], xs[i]).
// O(n) calls of `f`, O(n) space.
pub fn scan<D, T>(xs: [D]T, f: (T, T) -> T) -> [D]T

// Inclusive suffix scan (no identity needed): out[i] = f(xs[i], out[next]).
// O(n) calls of `f`, O(n) space.
pub fn suffix_scan<D, T>(xs: [D]T, f: (T, T) -> T) -> [D]T

// Exclusive cut-indexed prefix monoid fold: `pref[b]` is the fold of all
// elements strictly left of cut `b`, with `pref[D.first.l] == m.id`.
// O(n) calls of `m`, O(n) space.
pub fn prefix_fold<D, T, M>(xs: [D]T, m: M) -> [D.cuts]T

// Exclusive cut-indexed suffix monoid fold: `suff[b]` is the fold of all
// elements strictly right of cut `b`, with `suff[D.last.r] == m.id`.
// For any range `r: Range<D>`, `m.op(pref[r.start], suff[r.end])` folds everything
// outside `r` in O(1). O(n) calls of `m`, O(n) space.
pub fn suffix_fold<D, T, M>(xs: [D]T, m: M) -> [D.cuts]T

// For each element, yield `(x, fold of all other elements in order)` with no
// inverse required. O(n) calls of `m`, O(n) space.
pub fn excl_fold<T, M>(self: List<T>, m: M) -> yields (x: T, rest: T)
```

Laws:
- For `pref = prefix_fold(xs, m)` and `suff = suffix_fold(xs, m)`, at every cut
  `b in D.cuts`, `op(pref[b], suff[b]) == fold(m, xs)`.
- `scan(xs, op)[i] == prefix_fold(xs, m)[i.r]` for every `i in D`.
- `suffix_scan(xs, op)[i] == suffix_fold(xs, m)[i.l]` for every `i in D`.

### 3.5 Coordinate compression

`compress(all)` sorts and deduplicates `all`, returning a `Compressed<D, T>`
whose fresh domain `xs.D` keeps compressed indices disjoint from raw values and
from any other compression:

```cp
pub struct Compressed<D, T>(vals: [D]T):
    pub fn get<K>(self, i: K)  // O(1) element or O(r.len) slice
    pub fn slice(self, r: Range<D>) -> List<T>
    @get pub fn len(self) -> Int           // O(1)
    pub fn contains(self, val: T) -> Bool  // O(log n)
    pub fn try_at(self, val: T) -> ?D      // O(log n); None if absent
    pub fn at(self, val: T) -> D           // O(log n); traps if absent
    pub fn first_ge(self, val: T) -> ?D    // O(log n)
    pub fn first_gt(self, val: T) -> ?D    // O(log n)
    pub fn last_le(self, val: T) -> ?D     // O(log n)
    pub fn last_lt(self, val: T) -> ?D     // O(log n)

// O(n log n) time, O(n) space.
pub fn compress<T>(all: List<T>)
```

Laws:
- `xs.vals` is strictly increasing.
- `xs.try_at(v) == Some(i)` iff `xs[i] == v`.
- `xs.first_ge(v)` and `xs.last_lt(v)` partition `xs.D` at `v`; `xs.first_gt(v)`
  and `xs.last_le(v)` partition `xs.D` after `v`.

### 3.6 Permutations

```cp
pub type Perm<V> = [V]V

// Step `a` to the next / previous lexicographic permutation in place.
// Returns false and wraps around when already last / first.
// O(n) worst case, O(1) amortized, O(1) extra space.
pub fn next_permutation<T>(mut a: List<T>) -> Bool
pub fn prev_permutation<T>(mut a: List<T>) -> Bool

// Inverse of a bijection `p: [I]V -> [V]I`. Traps if `p` is not bijective.
// O(n) time and space.
pub fn inverse<I, V>(p: [I]V) -> [V]I

// Composition `(g o f)[i] = g[f[i]]`. O(A.count) time and space.
pub fn compose<A, B, C>(f: [A]B, g: [B]C) -> [A]C

// Disjoint cycle decomposition of a permutation `p: [V]V`.
// Each cycle starts at its smallest element in `V`. O(n) time and space.
pub fn perm_cycles<V>(p: [V]V) -> List<List<V>>

// `k`-th power of a permutation (`k >= 0`, up to 10^18) via cycle
// decomposition. O(n) time and space.
pub fn perm_pow<V>(p: [V]V, k: Int) -> [V]V

// Number of pairs (i < j) with `a[i] > a[j]` via merge sort.
// O(n log n) time, O(n) space.
pub fn inversions<T>(a: List<T>) -> Int
```

Laws:
- `compose(p, inverse(p)) == [I] i => i` and `compose(inverse(p), p) == [V] v => v`.
- `perm_pow(p, 0)` is the identity; `perm_pow(p, a + b) == compose(perm_pow(p, a), perm_pow(p, b))`.
- The parity of `inversions(p)` equals `(n - perm_cycles(p).len) % 2`.
- Stepping `next_permutation` from sorted `a` visits every distinct arrangement
  exactly once in ascending order and returns to `a`; `prev_permutation` is its
  exact reverse.

### 3.7 Binary search, unimodal search, and sorted lookup

```cp
// First / last position in `r` where monotone `ok` holds, or `None`.
// O(log r.len) calls of `ok`, O(1) space.
pub fn bsearch_first<I>(r: Range<I>, ok: I -> Bool) -> ?I
pub fn bsearch_last<I>(r: Range<I>, ok: I -> Bool) -> ?I

// Smallest `x` in `[lo, hi]` with monotone `ok(x)` to within
// `(hi - lo) / 2^steps`, or `hi` when None. O(steps) calls of `ok`.
pub fn bsearch_real(lo: F64, hi: F64, ok: F64 -> Bool, steps: Int = 100) -> F64

// Position of a maximum / minimum of unimodal `f` on non-empty `r`
// (strictly rising then falling, or falling then rising; plateaus only at
// the extremum). O(log r.len) calls of `f`, O(1) space.
pub fn ternary_max(r: Range<Int>, f: Int -> Int) -> Int
pub fn ternary_min(r: Range<Int>, f: Int -> Int) -> Int

// Unimodal extremum over `[lo, hi]` to within `(hi - lo) * (2/3)^steps`.
// O(steps) calls of `f`, O(1) space.
pub fn ternary_max_real(lo: F64, hi: F64, f: F64 -> F64, steps: Int = 100) -> F64
pub fn ternary_min_real(lo: F64, hi: F64, f: F64 -> F64, steps: Int = 100) -> F64

// Complete four-name sorted-lookup family over `[D]T` (including `List<T>`).
// Returns `?D` (`?Int` for `List<T>`). O(log n) time, O(1) space.
pub fn first_gt<D, T, K = T>(self: [D]T, target: K, by: T -> K = x => x) -> ?D
pub fn last_lt<D, T, K = T>(self: [D]T, target: K, by: T -> K = x => x) -> ?D

// Keyed twins of builtin `first_ge` / `last_le` (kept for existing ports;
// unified with `first_ge(.., by:)` / `last_le(.., by:)` once builtins move).
pub fn first_ge_by<T, K>(xs: List<T>, target: K, by: T -> K) -> ?Int
pub fn last_le_by<T, K>(xs: List<T>, target: K, by: T -> K) -> ?Int
```

Laws:
- For any monotone predicate `ok`, `bsearch_first(r, ok) == first(x for x in r if ok(x))`
  and `bsearch_last(r, ok) == max(x for x in r if ok(x))`.
- For any sorted `xs` and `target`, `first_ge` / `last_lt` and `first_gt` /
  `last_le` are exact complements around `target`.
- For any strictly unimodal `f` on `r`, `f(ternary_max(r, f)) == max(f(x) for x in r)`.

### 3.8 Monotone stack and fixed-window sliding extrema

```cp
// Nearest index strictly to the left / right with value strictly less than
// (or strictly greater than) `self[i]`, or `None`.
// With `strict: false`, matches `<=` / `>=`.
// O(n) time and O(n) space via monotone stack.
pub fn prev_less<D, T>(self: [D]T, strict: Bool = true) -> [D]?D
pub fn next_less<D, T>(self: [D]T, strict: Bool = true) -> [D]?D
pub fn prev_greater<D, T>(self: [D]T, strict: Bool = true) -> [D]?D
pub fn next_greater<D, T>(self: [D]T, strict: Bool = true) -> [D]?D

// Minimum / maximum of every contiguous length-`k` window of `self`
// (`1 <= k <= self.len`), in left-to-right order (length `self.len - k + 1`).
// O(n) time, O(n) space via monotonic deque.
pub fn sliding_min<D, T>(self: [D]T, k: Int) -> List<T>
pub fn sliding_max<D, T>(self: [D]T, k: Int) -> List<T>
```

Laws:
- For `p = a.prev_less()`: if `p[i] == Some(j)`, then `j < i`, `a[j] < a[i]`,
  and every `m` with `j < m < i` has `a[m] >= a[i]`; if `p[i] == None`, every
  `j < i` has `a[j] >= a[i]`.
- `a.sliding_min(k)[i] == min(a[i..i + k])!` and
  `a.sliding_max(k)[i] == max(a[i..i + k])!`.

### 3.9 Map entry helpers (currently in `std.seq`)

`get_or_insert`, `get_or_insert_with`, and `update` on `Map<K, V>` currently
live in `stdlib/seq.cp` and remain there until a dedicated `std.map` module
exists (so no port breaks). Range cut getters `Range.l`, `Range.r`, and
`Range.ends` stay in `std.seq` alongside range operations.

---

## 4. What stays user code

A good stdlib knows where to stop:

- **General two-pointer loops.** A two-pointer sweep (`var r = 0; for l in 0..n:
  while r < n and can_add(r): ...`) carries problem-specific incremental state
  (frequency maps, current sum, distinct count, deque invariants). Wrapping it
  in a 3-callback combinator hides control flow and makes debugging harder than
  a 5-line loop. `std.seq` covers the two universal, bug-prone special cases
  (`sliding_min`/`sliding_max` and `prev_less`/`next_less` monotone stacks) and
  leaves arbitrary two-pointer state to plain loops.
- **Custom monotone-stack DP transitions.** In `cf2084h`, the `Staircase`
  structure maintains a shifted stack plus a weighted area sum. That invariant
  belongs to the problem, not the stdlib.
- **Rolling DP buffers.** Two arrays plus `fill` and `swap` make the memory
  layout and transition order explicit (`design/spec/22-sequences.md` 22.2).
- **One-problem helpers.** Suffix MEX (`cf2066b`), permutation trees
  (`cf2187f1`), and ODT interval maps (`cf2237h`, which belongs in containers /
  range structures) stay out of `std.seq`.

---

## 5. Coordination with the core-extensions lane

Everything in section 3 compiles as ordinary CP Lang today except four
cross-cutting items owned by the core lane:

1. **Mutable slice lending (`mut a[l..r]`):** lets `reverse`, `rotate_left`,
   `rotate_right`, `sort`, and `next_permutation` act on subranges in place
   without slice-range wrappers (`cf2207h3`).
2. **Three-way `cmp:` option on `sort`/`sorted`:** avoids a wrapper struct and
   `impl Ord` over global state when sorting by a non-`Ord` comparison
   (`cf2135f`).
3. **Variadic `min`/`max` and predicate overloads on builtin `all`/`any`/`count`:**
   eliminates `max([a, b, c])` list literals (`cf2069e`) and lets
   `count(xs, pred)` / `all(xs, pred)` sit beside generator comprehensions
   (`cf2207e2`, `cf2245d1`).
4. **Overloading `first_ge`/`last_le` and `argmax`/`argmin` with optional `by:`:**
   builtin `List.first_ge` and `@get argmax` currently take fixed arity; once
   core allows optional `by:` on methods/getters or moves `first_ge`/`last_le`
   out of builtins into `std.seq`, `first_ge_by`/`last_le_by` and
   `argmax_by`/`argmin_by` collapse into one name each.

---

## 6. Real problems written against `std.seq`

### 6.1 Codeforces 2069 E — A, B, AB and BA

Partition `s` into maximal alternating runs with `s.chunk_by((x, y) => x != y)`.
Zero heap allocations for the run decomposition, no `start` index bookkeeping,
and no end-of-string boundary bug:

```cp
// Every AB/BA pair uses one 'A' and one 'B', so we need at least
// max(0, cnt_a - a, cnt_b - b) pairs. Partition s into maximal alternating
// runs: an odd run of length 2k+1 gives k flexible pairs; an even run of
// length 2k gives k pairs of its starting type or k-1 of any mix.
@!multitest
use std.seq.{chunk_by}

let s: Str = read()
let (a, b, ab, ba): (Int, Int, Int, Int) = read()

let cnt_a = s.count('A')
var need = max([0, cnt_a - a, (s.len - cnt_a) - b])
if need > ab + ba: return print("NO")

var wild = 0
var ab_runs, ba_runs: List<Int> = ([], [])
for r in s.chunk_by((x, y) => x != y):
    if r.len % 2 == 1: wild += r.len / 2
    elif s[r.first!] == 'A': ab_runs.push(r.len / 2)
    else: ba_runs.push(r.len / 2)

need -= wild
let (sum_ab, sum_ba) = (sum(ab_runs), sum(ba_runs))
if sum_ab + sum_ba < need: return print("NO")
if min(sum_ab, ab) + min(sum_ba, ba) >= need: return print("YES")

var (over, cap, rem, other) = if sum_ab > ab:
    (ab_runs, ab, sum_ab, min(sum_ba, ba))
else:
    (ba_runs, ba, sum_ba, min(sum_ab, ab))
over.sort()
for l in over.rev:
    rem -= l
    need -= l - 1
    if min(rem, cap) + other >= need: return print("YES")
print("NO")
```

### 6.2 Codeforces 2084 H — Turtle and Nediam 2

`s.runs()` works directly on `Str` (because `runs` is polymorphic over `[D]T`),
replacing the 9-line manual run-length loop and eliminating the missing-tail
bug class:

```cp
// DP over maximal equal blocks of binary string s. For each block-index
// parity, a monotone staircase sums the visible dp values over all heights.
@!multitest
type M = Mod<1_000_000_007>

let n: 3..=2_000_000 = read()
let s: Str = read()
let blocks = [r.len for r in s.runs()]
if blocks.len == 1: return print(M(n - 1))

var offset: [0..2]Int = 0
var stk: [0..2]List<(len: Int, val: M)> = []
var stk_sum: [0..2]M = 0
var prev_dp: M = 1

for k in 0..blocks.len - 1:
    let z = k % 2
    offset[z] += 1
    let shifted = blocks[k] - offset[z]
    if k >= 1:
        while let top = stk[z].last, shifted >= top.len:
            stk[z].pop()
            if let below = stk[z].last:
                stk_sum[z] -= M(below.len - top.len) * below.val
        if let below = stk[z].last:
            stk_sum[z] += M(below.len - shifted) * below.val
        stk[z].push((len: shifted, val: prev_dp))
    var cur_dp = stk_sum[z]
    if let top = stk[z].last:
        cur_dp += M(top.len + offset[z]) * top.val
    let base = if z == 0: blocks[0] else: 1
    prev_dp = cur_dp + M(base + offset[z] - 1)

print(prev_dp * M(blocks.last!))
```

### 6.3 Codeforces 2062 E2 — The Game (Hard Version)

`prefix_fold` and `suffix_fold` return `[t.Pre.cuts]MaxInfo<t.Pre>`, so the
maximum outside subtree `r = t.sub(v)` is `combine(pref[r.start], suff[r.end])`
with zero `.ord` arithmetic, and `w.buckets()` groups `V` by weight `W` in one
expression:

```cp
// Flatten the tree by preorder so each subtree is a range r = t.sub(v).
// The max weight outside r is combine(pref[r.start], suff[r.end]) over Cut<Pre>.
use std.seq.{buckets, prefix_fold, suffix_fold}

let empty_max = MaxInfo<t.Pre>(0, None, None, 0)
let mon = Monoid<MaxInfo<t.Pre>>(empty_max, combine)
let leaf = [t.Pre] p => MaxInfo<t.Pre>(w[t.at[p]].ord + 1, p, p, 1)
let pref = prefix_fold(leaf, mon)
let suff = suffix_fold(leaf, mon)
let by_w = w.buckets()

for weight in W.rev:
    let w_val = weight.ord + 1
    for v in by_w[weight]:
        let r = t.sub(v)
        let mx = combine(pref[r.start], suff[r.end])
        .. .
```

### 6.4 Codeforces 2187 F1 — Permutation Composition and Contribution of Minima

`inverse` turns `[Pos]V` into `[V]Pos` with full domain safety and a bijectivity
check, and `prev_less` / `next_less` compute monotone-stack spans as `[Pos]?Pos`
without `-1` or `n` sentinels:

```cp
use std.seq.{inverse, next_less, prev_less}

domain Pos = 0..n
domain Val = 1..=n
let a: [Pos]Val = read()
let b: [Pos]Val = read()

let inv_a = inverse(a)  // [Val]Pos
let p = [Pos] i => inv_a[b[Pos(n - 1 - i.ord)]]
let left = p.prev_less()                // [Pos]?Pos
let right = p.next_less(strict: false)  // [Pos]?Pos
```
