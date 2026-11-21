# DP and search: the stdlib design

This is the top-down design of the part of the standard library that holds
searches on answers and the DP toolkit: binary search on an answer, ternary
search, parallel binary search, the DP shapes (subsets, digits, intervals,
trees, layers), the monotone queue, the convex hull trick, the
divide-and-conquer and Knuth optimizations, the Lagrangian ("Aliens")
relaxation, memoized recursion and meet in the middle. It says which contest
patterns the area has to cover, the one API that covers them, the laws each
function relies on, what each costs, and what stays user code. Five real
problems are written against it at the end.

Status markers: **[have]** is in `stdlib/` today, **[new]** is added by
this design, **[core]** is a language feature owned by the core lanes that
this design relies on but does not implement. Claims about the corpus cite
the grep that produced them; claims about contest usage that come from
experience rather than a count are marked *(judgement)*.

## 1. What the area has to cover

### 1.1 Sources

- **The port corpus** (`proto/ports/**/*.cp`, outside `before/` and
  `today/`). Counted on the tree this design was written against:
  `first_ge` 54 uses, `bsearch_first` 30, `last_le` 22, `bsearch_last` 9,
  local `LiChao` 5, local `MonoDeque` 3, local `sos_sub`/`sos_sup` 2,
  `@memo` in 73 files. 28 ports still hand-roll a binary-search loop
  (`while lo < hi`, `while lo + 1 < hi`, `while hi - lo > ...`); among them
  `ecnerwala/cf2135f`, `ecnerwala/cf2178e`, `blind_critic_4/ioi20p1` and
  `scout_batch_21/ioi08p6`.
- **ecnerwala's library** (`proto/ports/ecnerwala/*_orig.cpp`, 279 files).
  32 files index a `dp[...]` table, 9 use `__builtin_popcount` or `1 << n`
  subset loops, 8 use `lower_bound`/`upper_bound`, 2 use `y_combinator`
  recursion, 1 a golden-section search, and none use a convex hull trick,
  Knuth or divide-and-conquer optimization. ecnerwala's DP is mostly plain
  tables with a clever state; the library carries little DP machinery.
- **KACTL** *(judgement, from its table of contents)*:
  `LineContainer` (max of lines, any insertion order, O(log n)),
  `DivideAndConquerDP`, `KnuthDP`, `TernarySearch` (integer, unimodal),
  `GoldenSectionSearch`, `FastKnapsack`, `LIS`, `ConstantIntervals`.
- **AtCoder Library** *(judgement)*: no DP module; its searches are
  `max_right`/`min_left` on segment trees (owned by the range-structure
  lane), `lower_bound` on sorted data, and `convolution`.
- **Codeforces/AtCoder/ICPC usage** *(judgement)*: binary search on the
  answer is the most common technique of the whole area, then layered DP
  with a rolling array, bitmask DP, digit DP, interval DP, tree DP with
  rerooting, sliding-window minimum, and at the hard end CHT/Li Chao,
  divide-and-conquer optimization, Aliens, Knuth, parallel binary search,
  SOS and meet in the middle. EDPC (AtCoder's Educational DP Contest) is
  the canonical syllabus: A–Z touch every DP shape below.
- **Friction reports** (`research/frictions/`, `research/friction-backlog-*`):
  the ones in this area are listed in section 6 with how the design answers
  each.

### 1.2 The patterns

1. **Search an answer on integers**: the first `x` where a monotone `ok(x)`
   holds, over `lo..=hi` up to `1e18`, or over a domain. The predicate is
   the algorithm; the search is bookkeeping that is easy to get off by one.
2. **Search an answer on reals**: a fixed number of halvings of `[lo, hi]`.
3. **Optimize a unimodal function**: integer ternary search on a strictly
   unimodal `f`, golden-section on reals.
4. **Parallel binary search**: `q` independent searches over the same
   timeline of `T` events whose predicate needs the state after a prefix of
   events (DSU, Fenwick). `O((T + q) log T)` instead of `O(q T)`.
5. **DP over a layer**: `next` from `prev`, for `k` layers, keeping two rows.
6. **DP over subsets**: iterate masks in increasing order; sum over subsets
   or supersets (SOS) in `O(n 2^n)`.
7. **DP over digits**: count numbers `<= N` with a digit property; a
   recursion on `(position, state, tight)`.
8. **DP over intervals**: `dp[l][r]` from shorter intervals; with Knuth's
   optimization when `opt` is monotone.
9. **DP over trees**: post-order fold, then rerooting.
10. **Sliding-window optimum**: `min`/`max` over the last `k` positions,
    including `dp[i] = min(dp[j] : i - k <= j < i) + c(i)`.
11. **Convex hull trick**: `dp[i] = min_j (a_j x_i + b_j)`, lines arriving in
    any order, queries anywhere; sometimes segments; sometimes ties broken
    by a second key (Aliens).
12. **Monotone argmin**: `dp[i] = min_{j < i} prev[j] + cost(j, i)` with a
    monotone optimal `j` (divide-and-conquer optimization).
13. **Lagrangian relaxation**: "exactly `k` pieces" through a penalty `λ` per
    piece and a binary search on `λ`, for a convex answer in `k`.
14. **Memoized recursion**: any of the above written top-down.
15. **Meet in the middle**: all subset sums of each half, sorted, then a
    two-pointer or a sorted lookup.

## 2. The API

Two homes. The searches stay in `std.seq` beside `bsearch_first`, since they
are searches on ranges like the sorted lookups next to them. The DP toolkit
is a new module `std.dp`. Neither needs a compiler power: every function is
ordinary CP Lang over closures, generics and lists.

### 2.1 `std.seq`: search on answers

The searches (`bsearch_first`/`_last`, `bsearch_real`, `ternary_*`,
`parallel_bsearch`) have their own design, `design/stdlib/search.md`.

### 2.2 `std.dp` [new]: the DP toolkit

The sliding-window optimum is `std.window.MonoDeque<T>` (containers
area): `push(at, val)` with increasing `at`, `drop_before(at)`, `best`, all
amortized O(1), memory O(window). The DP toolkit reuses it rather than
keeping a second queue under another name.

```cp
struct Line(k: Int, m: Int)
    fn at(self, x: Int) -> Int                  // k * x + m
struct LiChao<L, K>
    fn new(pick: (K, K) -> K, xs: Range<Int>) -> LiChao<L, K>   // O(1)
    fn add(mut self, l: L, on: ?Range<Int> = none)
        // O(log xs.len) for a whole line, O(log^2 xs.len) for a segment
    fn at(self, x: Int) -> ?K                   // O(log xs.len)
    fn line_at(self, x: Int) -> ?L              // O(log xs.len)
    fn clear(mut self)                          // O(1)
```

The convex hull trick. `L` is anything with `at(self, x: Int) -> K`:
`Line` gives `LiChao<Line, Int>(min, 0..=m)`; a struct whose `at` returns
`(cost: Int, used: Int)` gives the tie-broken envelope Aliens needs, with
the same tree. `pick` is `min` or `max`. Nodes are allocated as lines
arrive, so memory is O(lines) words, not O(xs.len), and an array of trees
(`[Station]LiChao<Line, Int>`) costs a few words per empty tree. Law: two
`L`s cross at most once on `xs` (true of lines and of
`(line, tie-break)` pairs). Overflow: `k * x + m` must fit in `Int`.

```cp
fn monotone_minima(rows: Int, cols: Int, f: (Int, Int) -> ?Int)
    -> List<?(val: Int, at: Int)>
    // O((rows + cols) log rows) calls of f. For each row the least f(row,
    // col) and its leftmost col; none where f is none on the whole row.
    // Law: the leftmost argmin is nondecreasing in row (true for a Monge
    // f, checkable with std.spec.monge_break).
fn dnc_layer(prev: List<?Int>, cost: (Int, Int) -> Int) -> List<?Int>
    // O(n log n) calls of cost, n = prev.len. next[i] = min over j < i of
    // prev[j] + cost(j, i); next[0] = none. Law: cost satisfies the
    // quadrangle inequality (std.spec.dnc_break).
fn knuth(n: Int, cost: (Int, Int) -> Int) -> List<List<Int>>
    // O(n^2) calls of cost, O(n^2) memory. dp[i][j] over cuts 0..=n:
    // dp[i][i + 1] = 0, dp[i][j] = cost(i, j) + min over i < k < j of
    // dp[i][k] + dp[k][j]. Law: cost is monotone and satisfies the
    // quadrangle inequality (std.spec.knuth_break).
fn aliens(k: Int, lambdas: Range<Int>, solve: Int -> (val: Int, count: Int))
    -> Int
    // O(log lambdas.len) calls of solve. f(k) for a convex f, where
    // solve(λ) is min over c of f(c) + λ c and the least c attaining it.
    // Law: f is convex in k; λ in lambdas reaches count <= k.
fn subset_sums(xs: List<Int>) -> List<Int>
    // O(2^n) time and memory. Every subset sum of xs, sorted, by merging.
    // Sums over subsets and supersets are the math area's in-place
    // std.ntt.sos_sub / sos_sup (inverse: Bool = false), O(n 2^n).
```

Each DP optimization takes the cost as an operation and returns the plain
table, so the recurrence stays visible in user code and the optimization is
one named call. Each law has a checker in `std.spec` **[have]**
(`monge_break`, `dnc_break`, `knuth_break`, `convexity_break`), so "does
the quadrangle inequality hold here?" is one `cpl stress` away instead of a
proof at 3am.

### 2.3 Names and their neighbours

- `bsearch_first`/`bsearch_last` fix the family; `bsearch_real` is the same
  search on another domain, and `parallel_bsearch` is the textbook name of
  the technique. A contestant who knows one guesses the rest.
- `ternary_max`/`ternary_min` pair like `argmax`/`argmin` and `max_by`/
  `min_by`; `_real` marks the `F64` domain exactly as in `bsearch_real`.
- `pick: min | max` is the stdlib's existing convention for a structure's
  operation (`SegTree<V, Int>(max, a)`, `SparseTable`), so `MonoDeque` and
  `LiChao` take it the same way, and the query on each is `best`/`at`.
- Options are named arguments with defaults (`steps:`, `on:`), never
  `_by`/`_segment` variants.
- The local-library names `LiChaoTag` and `LiChaoF` are retired: the
  tagged and float trees are instances of `LiChao<L, K>`.

### 2.4 Cost against C++ (measured)

`bench/dp/` runs each entry point at judge size beside the same algorithm
in C++. `LiChao`, `dnc_layer` and `MonoDeque` run at or under the C++
time; `knuth` at 1.24x, because its split table holds 8-byte `Int`s
where C++ uses `int` (parity needs `I32` storage from the core). Two
things in the library make this hold: operations are kept as the value the
caller passed (`pick`), never wrapped in a fresh closure, so the codegen's
devirtualization inlines `min` / `max`; and the hot loops are `unchecked`.

## 3. Domains and index spaces

- Searches return the position type of the range they search. Over a domain
  `V`, `bsearch_first(V.all, ok)` is a `?V`; over cuts, a `?V.bounds`. The
  caller never converts between an index and an offset.
- `MonoDeque` and `parallel_bsearch` number positions and queries with
  `Int`: a queue's positions are the loop variable of the DP that feeds it,
  usually `Int`; a domain loop passes `v.to_idx`. *Open question:* make
  them generic over an ordered position type once the core can infer `I`
  from the first `push`.
- `LiChao` is over coordinates (`x` values), which are not an index space:
  `xs: Range<Int>` is a value range like `0..=1e9`, and a compressed
  coordinate space works by passing its indices.
- `knuth` and `dnc_layer` index tables by cuts `0..=n`, the same space as a
  prefix-sum array: `dp[i][j]` is the interval between cut `i` and cut `j`.

## 4. What stays user code, and why

- **The recurrence.** The library never sees a DP's state, only the cost
  operation of an optimization. Writing `dp[k + 1, c] max= dp[k, c] + gain`
  is the editorial.
- **Memoized recursion** is `@memo` **[core]**, which picks a dense table
  from the parameters' types (`fn f(i: 0..=n, tight: Bool)`) or a hash
  table with `@memo(hash)`. Digit DP and interval DP are `@memo` functions
  over `digits(n)` **[have]** and a domain: section 5.5. No `digit_dp`
  combinator: the state differs every time and a combinator would hide it.
- **Tree DP** is a post-order loop or `@memo` over `Rooted` **[have]**; the
  rerooting step is `Rooted.excl_fold` **[have]** in `std.tree`.
- **Layered DP** keeps two rows; a rolling table is core (see 6.1), not a
  library type.
- **Subset DP** iterates masks in increasing order; `Bits<D>` (spec 21.7)
  **[core]** will type them. Only the SOS transforms are library.
- **Meet in the middle** is `subset_sums` of each half and a sorted lookup
  (`first_ge`) or a two-pointer: the combination step is the problem.

## 5. Five problems against the API

Each is written as the solution should read; they are also the module's
tests and examples (`stdlib/dp.cp`), run on the official samples.

### 5.1 EDPC Z: Frog 3 (convex hull trick)

[Problem](https://atcoder.jp/contests/dp/tasks/dp_z). Heights `h` strictly
increase, `n <= 2e5`; a jump from `j` to `i` costs `(h_i - h_j)^2 + C`;
the cheapest path from stone 1 to stone n. Expanding the square,
`dp[i] = h_i^2 + C + min_j (-2 h_j) h_i + (dp[j] + h_j^2)`: a minimum of
lines at `x = h_i`.

```cp
// dp[i] = h_i^2 + C + the least line (-2 h_j, dp[j] + h_j^2), j < i, at h_i.
let (n, c): (1..=200_000, 1..=1e12) = read()
domain Stone = 0..n
let h: [Stone]1..=1_000_000 = read()
var hull = LiChao<Line, Int>(min, 1..=1_000_000)
var dp: [Stone]Int = 0
for i in Stone:
    if i != Stone.first: dp[i] = h[i] * h[i] + c + hull.at(h[i])!
    hull.add(Line(-2 * h[i], dp[i] + h[i] * h[i]))
print(dp[Stone.last])
```

### 5.2 Codeforces 321E: Ciel and Gondolas (divide and conquer)

[Problem](https://codeforces.com/contest/321/problem/E). `n <= 4000`
people in a queue split into `k <= 800` consecutive gondolas; a gondola
costs the sum of the unfamiliarities `u[a][b]` of the pairs in it. The
cost of a group is a rectangle sum of `u`, which satisfies the quadrangle
inequality, so each layer is a monotone argmin.

```cp
// Layer t: best[i] = min over j < i of layer t-1 at j + cost(j, i);
// cost(j, i) = half the u-sum over the square [j, i)^2, a Monge cost.
let (n, k): (1..=4000, 1..=800) = read()
domain P = 0..n
let u: [P, P]0..=9 = read()
let pre = prefix2(u)
fn cost(j: Int, i: Int) -> Int = pre.sum(j..i, j..i) / 2
var best: List<?Int> = [if i == 0: 0 else: none for i in 0..=n]
for _ in 0..k: best = dnc_layer(best, cost)
print(best[n]!)
```

### 5.3 AGC 002 D: Stamp Rally (parallel binary search)

[Problem](https://atcoder.jp/contests/agc002/tasks/agc002_d). Graph with
`n, m <= 1e5`; each query `(x, y, z)` asks for the least `t` such that the
union of the components of `x` and `y` after adding edges `1..=t` has at
least `z` vertices. The predicate is monotone in `t` and needs a DSU after
a prefix of edges: one parallel binary search over the edge timeline.

```cp
// (x, y, z) asks for the first edge t after which the components of x and
// y hold z vertices: monotone in t, so all queries search one timeline.
let (n, m): (1..=100_000, 1..=100_000) = read()
domain V = 1..=n
domain E = 1..=m
let edges: [E](a: V, b: V) = read()
let q: 1..=100_000 = read()
let asks: [0..q](x: V, y: V, z: 1..=n) = read()
var dsu = Dsu<V>()
fn reset(): dsu = Dsu<V>()
fn join(t: E): dsu.unite(edges[t].a, edges[t].b)
fn reaches(i: Int) -> Bool:
    let (x, y) = (dsu.find(asks[i].x), dsu.find(asks[i].y))
    asks[i].z <= dsu.size(x) + (if x == y: 0 else: dsu.size(y))
for t in parallel_bsearch(q, E.all, reset, join, reaches): print(t!)
```

### 5.4 EDPC N: Slimes (interval DP, Knuth)

[Problem](https://atcoder.jp/contests/dp/tasks/dp_n). Merge `n <= 400`
adjacent slimes; merging costs the merged size. `dp[l][r]` over cuts is
`sum(l..r) + min_k dp[l][k] + dp[k][r]`; the sum is monotone and satisfies
the quadrangle inequality, so Knuth's bound applies (and turns `O(n^3)`
into `O(n^2)`, which matters at `n = 5000` variants).

```cp
// dp[l][r] = sum of a over the cuts l..r + the best split k of [l, r).
let n: 2..=400 = read()
domain S = 0..n
let a: [S]1..=1e9 = read()
let pre = prefix1(a)
print(knuth(n, (l, r) => pre.sum(l..r))[0][n])
```

### 5.5 EDPC S: Digit Sum (memoized digit DP, user code)

[Problem](https://atcoder.jp/contests/dp/tasks/dp_s). Count `x` in
`1..=K` (`K < 10^10000`, a string) whose digit sum is a multiple of
`D <= 100`, mod `1e9 + 7`. There is no library call here on purpose: the
state is `(position, residue, still tight)` and `@memo` sizes its table
from the parameter types.

```cp
// ways(i, r, tight): digit strings for positions i.. whose digit sum is r mod d
// (exactly, when tight, below K's suffix).
type Z = Mod<1_000_000_007>
let k: Str = read()
let d: 1..=100 = read()
@memo
fn ways(i: 0..=k.len, r: 0..d, tight: Bool) -> Z:
    if i == k.len: return if r == 0: 1 else: 0
    let top = if tight: digit(k[i]) else: 9
    sum(ways(i + 1, (r + x) % d, tight and x == top) for x in 0..=top)
print(ways(0, 0, true) - 1)
```

## 6. Frictions this design answers

### 6.1 Rolling DP layers (`cf2183e`, `cf2084g1`)

"A rolling DP layer needs two declarations, a `fill` and a `swap`" and "a
DP over placed-so-far carries a rolling array by hand". Both dreams write
the rows once (`var ways, next: [1..=m]Z = 0`, `ways = next.take()`, or a
`@rolling` table). That is a declaration and move-semantics question for
the core lane, not a library type: a `Rolling<T>` struct would hide the
index arithmetic the dream wants visible. Recorded as a dependency, not
built here.

### 6.2 Binary search that needs scratch state (`cf2245g`)

"Binary search over a predicate calling an interactive query needs slice
concatenation" asks for `bsearch` over an index range returning `?I` and
for no copy per probe. `bsearch_first` already returns `?I` over any range;
the copy is a slice question (sequences lane).

### 6.3 Splitting a range at its midpoint (`cf2069f`)

The divide-and-conquer helpers here (`monotone_minima`, `knuth`) own their
splits, so user code never computes a midpoint cut. `Range.halves()` is a
sequences-lane name; recorded there.

### 6.4 Generic Li Chao (backlog 14 P12; `ioi16p6io`, `noi19p1`)

Asked for `LiChao<F, V>` over any line type with an ordered value (Aliens'
`(cost, used)` tie-break) and a forest of trees per station. `LiChao<L, K>`
is exactly the first; the second is `[Station]LiChao<Line, Int>`, cheap
because nodes are allocated lazily.

### 6.5 Optimization argmins (`friction-backlog-batch-14` P13)

Knuth and D&C DP need the winning split, not only the value.
`monotone_minima` returns `(val, at)`; `knuth` keeps `opt` internally and
returns the table (path reconstruction from `opt` is an open question).

### 6.6 Hand-rolled binary-search loops (28 ports)

The integer searches existed; ports kept writing loops because the answer
space was huge (`0..=2e18`) or real. `bsearch_first` handles any `Int`
range in `O(log)` without overflow, and `bsearch_real` covers the rest.

## 7. The maximal version

The most ambitious version of this area makes every DP technique a single
named step that is checked: the law each optimization needs (monotone
predicate, unimodality, Monge, quadrangle, convexity) is a `law` the
debugger and `cpl stress` check on small inputs automatically when the
function is called in debug builds, so a wrong application fails on the
first sample rather than at WA on test 37. It adds the rest of the hard
end (slope trick as a structure with `add_left`/`add_right`/`min`, kinetic
segment trees, SMAWK, the monotone CHT deque for O(1) amortized, `Bits<D>`
subset loops, a typed `@rolling` table) and gives `@memo` a bottom-up
`into` table so a top-down DP can be flipped to an iterative one with one
edit.

The next step after this design lands: law checks in debug builds for
`bsearch_*` (already checks the ends), `ternary_*` (sampled unimodality)
and `dnc_layer`/`knuth` (sampled quadrangle inequality), behind the
existing `std.spec` checkers.
