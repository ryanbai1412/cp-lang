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
`parallel_bsearch`) have their own design, `spec/lib/rationale/search.md`.

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
    fn add(mut self, l: L, on: ?Range<Int> = None)
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
fn monge_path<K>(n: Int, w: (Int, Int) -> K) -> (val: List<K>, prev: List<Int>)
    // Online LARSCH: O(n) calls of w, O(n) memory. val[i] = least weight of
    // a path 0 -> i through increasing nodes, prev[i] its last hop (prev[0]
    // = 0). Law: w is Monge (std.spec.monge_break).
fn knuth(n: Int, cost: (Int, Int) -> Int) -> List<List<Int>>
    // O(n^2) calls of cost, O(n^2) memory. dp[i][j] over cuts 0..=n:
    // dp[i][i + 1] = 0, dp[i][j] = cost(i, j) + min over i < k < j of
    // dp[i][k] + dp[k][j]. Law: cost is monotone and satisfies the
    // quadrangle inequality (std.spec.knuth_break).
fn aliens<L>(k: Int, lambdas: Range<L>, solve: L -> (val: L, count: Int))
    -> L
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
time; `knuth` at 1.11x, its split table held as `0..2^31` (4 bytes,
as the C++ `int`; 1.24x when it held 8-byte `Int`s). Two
things in the library make this hold: operations are kept as the value the
caller passed (`pick`), never wrapped in a fresh closure, so the codegen's
devirtualization inlines `min` / `max`; and the hot loops are `unchecked`.

## 3. Domains and index spaces

- Searches return the position type of the range they search. Over a domain
  `V`, `bsearch_first(V.all, ok)` is a `?V`; over cuts, a `?V.cuts`. The
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
let pre = prefix(u)
fn cost(j: Int, i: Int) -> Int = pre.sum(j..i, j..i) / 2
var best: List<?Int> = [if i == 0: 0 else: None for i in 0..=n]
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
let pre = prefix(a)
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

### 6.7 Concave functions as sorted slopes (`cf2084g2`)

`cf2084g2`'s DP over the number of evens `c` is concave, and the port drove
a `LazyTreap` with `apply`/`compose`/`split` lambdas to keep its slopes. The
lambdas are the slope trick, not the problem, so `std.slope` owns them:
`Concave` is an integer function on `lo..=hi` kept as `f(lo)` plus its
slopes `f(c + 1) - f(c)` in falling order, and `Convex` is its negation.

| op | meaning | cost |
|---|---|---|
| `Concave.point(d)` | 0 at `d` only | O(1) |
| `Concave.flat(lo..=hi)` | 0 on `lo..=hi` | O(len log len) expected |
| `f.max_convolve(g)` | `f(c) = max over a + b = c of f(a) + g(b)` | O(\|g\| log n) expected |
| `f.add_quadratic(q)` | `f(c) += q(c)` | O(1) |
| `f.at(c)`, `f.max()` | a value, the maximum | O(c - lo + log n), O(n) |

Laws: the max-plus convolution of concave functions merges their slope
multisets, so the domains add and nothing is recomputed. Adding a quadratic
adds an arithmetic progression by rank: slope `j` gains `q(lo + j + 1) -
q(lo + j)`. `add_quadratic` reads `q` at `lo`, `lo + 1` and `lo + 2`, traps
unless `q(lo + 3)` agrees (so `q` is quadratic) and unless `q'' <= 0`, which
keeps the slopes sorted. `Convex` mirrors every operation (`min_convolve`,
`min`, `q'' >= 0`); this is the `add_left`/`add_right` slope trick of §7 on
integer points, with the breakpoints read as unit-step slopes.

```cp
var dp = Concave.point(0)
for k in 0..n:
    dp.add_quadratic(c => gain(k, c))
    if let is_even = fixed_even[k + 1]:
        dp.max_convolve(Concave.point(Int(is_even)))
    else:
        dp.max_convolve(Concave.flat(0..=1))
print((dp.at(half) + n * (n + 1) * (n + 1) / 2) / 2)
```

Not yet: hinge adds `max(0, c - a)` (a rank-range add on the treap) and
prefix minima, which finish the breakpoint form; they land with the first
port that needs them.

### 6.8 Sampling a DP along a diagonal (`cf2115c`): rejected

`cf2115c`'s dream reads both edge DPs with one helper, `diagonal(size, dp,
step, at)`: roll a DP over rounds left and record `at(dp, r - slack)` for
`r` in `slack..=slack + size`. It stays user code. A search of every port and
dream (`rounds/steps/turns left`, loops that record once a counter passes an
offset) finds no second use; the hits are unrelated (`cf2135b`, `cco02p3`,
`ioi06p4`), so a `std.dp` primitive would be a one-use helper. Its parts are
already general: the rolling step is a value (`dp = round(dp)`, §6.1), and the
read-off is a loop with one `if`. A lazy `successors(first, step)` stream
(spec 22, owned by seq) would express the same as
`successors(dp0, step).skip(slack).take(size + 1).map(at)` once it lands,
which is the general mechanism if a second use appears.

## 7. The maximal version

The most ambitious version of this area makes every DP technique a single
named step that is checked: the law each optimization needs (monotone
predicate, unimodality, Monge, quadrangle, convexity) is a `law` the
debugger and `cpl stress` check on small inputs automatically when the
function is called in debug builds, so a wrong application fails on the
first sample rather than at WA on test 37. It adds the rest of the hard
end (the rest of the slope trick beyond §6.7's `std.slope`: hinge adds and prefix minima, kinetic
segment trees, SMAWK, the monotone CHT deque for O(1) amortized, `Bits<D>`
subset loops, a typed `@rolling` table) and gives `@memo` a bottom-up
`into` table so a top-down DP can be flipped to an iterative one with one
edit.

The next step after this design lands: law checks in debug builds for
`bsearch_*` (already checks the ends), `ternary_*` (sampled unimodality)
and `dnc_layer`/`knuth` (sampled quadrangle inequality), behind the
existing `std.spec` checkers.

## Coverage

The topics a strong contest library covers for DP and search, from the
AtCoder Library, KACTL, cp-algorithms and top competitors' personal
libraries. **have** is on main (in this area or the one named), **new** is
in the plan/STDLIB.md `[new]` block, **out** says why it stays out.

| topic | status | where, or why out |
|---|---|---|
| binary search on an integer answer | have | `bsearch_first`/`bsearch_last`, lint `L0520` |
| binary search on a real answer | have | `bsearch_real` |
| exponential (galloping) search, no upper end | new | `gallop_first` |
| warm-started binary search | new | `bsearch_first(warm:)` |
| ternary search, integers | have | `ternary_max`/`ternary_min` (strict unimodal) |
| ternary / golden-section search, reals | have | `ternary_max_real`/`ternary_min_real` |
| convex minimum with plateaus | out | one line: `bsearch_first(r, i => f(i + 1) >= f(i))`; a name would hide the law |
| fractional programming (Dinkelbach) | new | `dinkelbach` |
| parallel binary search | have | `parallel_bsearch` |
| search on a structure (segment tree descent) | have | `max_right`/`min_left`, ranges lane |
| binary lifting search | have | trees lane (`Lift`) |
| sorted lookup (lower/upper bound) | have | `first_ge`/`first_gt`/`last_le`/`last_lt`, seq lane |
| two pointers | out | a `while` over two cuts says it best; no shape to name |
| memoized recursion | have | `@memo` (core), enum and `?enum` keys |
| rolling DP layers | out | core: `@rolling` or move semantics (§6.1), not a library type |
| top-down to bottom-up flip | out | core: `@memo(into:)` (Powers) |
| sliding-window optimum (monotone queue) | have | `MonoDeque`, containers lane (`std.window`) |
| convex hull trick, Li Chao | have | `LiChao<L, K>`, segment adds |
| convex hull trick, monotone deque | new | `MonotoneHull` |
| convex hull trick, dynamic (KACTL LineContainer) | new | `LineContainer` |
| kinetic segment tree | out | rare in contests; `LiChao` and `LineContainer` cover the uses seen |
| divide and conquer optimization | have | `dnc_layer`; cut form new |
| Knuth optimization | have | `knuth`; cut form `Knuth<D, K>` new |
| Monge shortest path (LARSCH) | have | `monge_path`; online `MongePath` new |
| SMAWK row minima | new | `smawk` |
| monotone minima | have | `monotone_minima` |
| min-plus / max-plus convolution of convex sequences | new | `min_plus_convolve`/`max_plus_convolve` |
| Aliens trick (Lagrangian, WQS) | have | `aliens` |
| slope trick | have | `std.slope` `Concave`/`Convex` |
| slope trick: hinge adds, prefix minima, shifts | new | `add_left`, `add_right`, `prefix_min`, `shift`, `slide_min`, ... |
| longest increasing subsequence | new | `lis`, `lis_len` |
| sum over subsets (zeta / Mobius) | have | `sos`, math lane (`std.ntt`) |
| submask / supermask / fixed-size mask enumeration | new | `submasks`, `supermasks`, `masks_of_size` |
| meet in the middle | have | `subset_sums`; generic `subset_folds`, `meet_in_middle` new |
| subset sum by bitset | have | `Bitset.subset_sums`, containers lane |
| 0/1, bounded and unbounded knapsack | out | the recurrence is the problem; 3 lines over a list, bounded is `MonoDeque` |
| digit DP | out | `@memo` over (position, tight, state) is the clear form (§5.5) |
| interval DP, general O(n^3) | out | user code; `knuth` when the quadrangle law holds |
| bitmask DP (TSP, Hamiltonian paths) | out | `@memo` or a `[1 << n]` table with `submasks` |
| broken-profile DP | out | the profile is problem-specific; user code over masks |
| Steiner tree DP | out | graphs lane: needs shortest paths per mask |
| tree DP, rerooting | out | trees lane (`reroot`) |
| CDQ divide and conquer | out | each use differs in the merge; user code over `Range.halves` |
| LCS, edit distance, Hirschberg | out | strings lane |
| linear recurrences, matrix power | have | `std.matrix`, `std.spec.berlekamp_massey`, `guess_at` |
| law checks for each optimization | have | `std.spec` `monge_break`, `dnc_break`, `knuth_break`, `convexity_break`; `total_monotone_break`, `quadrangle_break` new |
| backtracking with pruning | new | `backtrack`, steered by `Visit.prune()` (decision 126) |
| A*, best-first over states | out | graphs lane: `dijkstra` over implicit states |

Totals: 22 have, 11 new, 15 out (48 topics).

## Conventions

The coherence pass over the area, against plan/STDLIB.md §2.

- **Construction.** The type called like a function: `LiChao<Line,
  Int>(min, 0..n)`, `MonoDeque<Int>(pick: max)`, `MongePath<D, K>(w)`.
  `T.new(...)` is the explicit spelling; named constructors only for an
  alternative form (`Concave.point(d)`, `Concave.flat(r)`). Drift left
  to fix when the [rename] lands: `MonoDeque<Int>.new(min)` in `ucup4s24f`
  and `bench/dp/mono.cp`, `.new(by: min)` in `bench/containers/monodeque.cp`.
- **Argument vocabulary.** A binary chooser (`min`, `max`, any `(K, K) ->
  K`) is `pick:`; a key is `by:`; a predicate is `keep:`; a starting value
  is `init:`. `MonoDeque.new(by:)` takes a chooser and is a [rename] to
  `pick:`.
- **Spaces.** Searches take `r: Range<I>` and return `?I`, never a
  sentinel. Optimizations over a sequence take pieces, `cost: Range<D> ->
  K`, and return tables over the cuts `D.bounds` (decision 124); the
  `n: Int` forms are the [rename] path. Matrix searches (`monotone_minima`,
  `smawk`) keep `rows`/`cols` integers: a matrix is not a cut space.
- **Results.** An optimum and where it is attained is `(val:, at:)`; a
  path is `(val:, prev:)`.
- **Costs.** Counted in calls of the user's operation when that dominates
  (`O(n log n) calls`).
- **Real searches.** `bsearch_real(lo, hi, ...)` and `ternary_*_real`
  take two ends, not a range, because `Range<F64>` does not exist; see
  Powers.

## Cross-area

| name here | collides or duplicates | resolution |
|---|---|---|
| `subset_sums` (`std.dp`, sorted `List<Int>`) | `Bitset.subset_sums` (containers, reachable sums) | both stay: different results; each doc names the other |
| `sos` (math, `std.ntt`) | dreams spell `zeta` (`cf2178i`) | `sos(over:, inverse:)` is the name; no `zeta` |
| `sos(inverse: true)` | `Sieve.mobius` (math, number-theoretic) | the subset form is never called `mobius` |
| `Line` (`std.dp`) | a future geometry line | geometry keeps `HalfPlane`/`Vec`; a geometry line is not named `Line` |
| `at` (evaluate: `Line`, `LiChao`, `Concave`) | `Compressed.at(val)` (seq, a position) | flagged to seq: `Compressed.at` reads as evaluation |
| `MonoDeque`, `WindowFold` (containers) | `sliding_min`/`sliding_max` (seq) | `sliding_*` for a fixed `k` in one call, `MonoDeque` for moving ends |
| `Concave.max_convolve` | `max_plus_convolve` (new, lists) | the method on a treap-kept function, the free fn on sequences |
| `compose` (seq, permutations) | `Action.compose` (algebra) | both kept: different receivers |
| `bsearch_first`/`bsearch_last` | `max_right`/`min_left` (ranges) | same monotone law, structure-specific descent; docs cross-reference |
| `parallel_bsearch` | `std.mo`, `TimeTree.offline_over` | three offline frameworks with different shapes; each doc says which to use |
| `reserve` | `List.reserve` | same meaning everywhere |
| `convexity_break` and friends (`std.spec`) | grid/spec lane (4.15) | the laws are owned here, the module there |

## Powers

Signatures above that today's public language cannot write.

1. **Debug-only law checks.** `knuth`, `dnc_layer`, `smawk`, `ternary_*`,
   `Concave.add_quadratic`, `bsearch_*`. Example: `law quadrangle_break(cost)
   == none` checked in debug builds on sampled inputs and free in release.
   Today `add_quadratic` checks in every build.
2. **Real ranges.** `bsearch_real`, `ternary_*_real`, `dinkelbach` as
   `bsearch_first(0.0..1e9, ok, eps: 1e-9)`: `Range<F64>` with a resolution.
3. **Typed bit sets `Bits<D>`.** `submasks`, `supermasks`,
   `masks_of_size`, `subset_folds` over a domain: `for s in
   submasks(m: Bits<Item>)` yielding `Bits<Item>`, not `Int`. The `Bits<D>`
   forms replace the `Int` ones when the power lands; the two never sit side
   by side (no overloading by argument type, decision 127).
4. **Steered generators** (decision 126). `backtrack` and `Visit.prune()`:
   `for v in backtrack(s0, next): if bad(v.state): v.prune()`. Early exit
   is `break` or `return` in that `for` body, never a `found` flag or a
   closure; the generator reads `prune()` after its `yield` returns.
5. **Memo into a table.** `@memo(into: dp)` so a top-down DP flips to a
   bottom-up table with one edit, and `@rolling` layers (§6.1).
6. **Cut ranges as pieces** (decision 124, accepted, not implemented). Every
   cut form: `knuth(cost: Range<D> -> K)` called as `cost(j..i)` with cuts
   `j, i`.
7. **Default arguments over enums.** `min_plus_convolve(convex: Shape =
   Both)`; to confirm against the checker, else an overload set.

## Dreams

Dreams in `proto/ports/*/dream/` that use this area, and how the spec spells
what they use.

| dream | uses | in the spec |
|---|---|---|
| `agc077a` | `bsearch_last` | have |
| `cf2022c` | `@memo` on enum keys | have (core) |
| `cf2035f` | `bsearch_first` | have |
| `cf2039f1` | `mobius` | math lane's `Sieve.mobius` |
| `cf2057f` | `bsearch_last` | have |
| `cf2084g2` | `Concave` | have |
| `cf2096f` | `MonoDeque` | have (containers) |
| `cf2115c` | DP along a diagonal | rejected, user code (§6.8) |
| `cf2129c3` | `bsearch_first` | have |
| `cf2178d` | `bsearch_last` | have |
| `cf2178i` | `zeta` | spelled `sos(f)` |
| `cf2183h` | `aliens`, `monge_path` | have; the cut form after decision 124 |
| `cf2187d` | `subset_sums` | have |
| `cf2262b` | `bsearch_first`, `first_ge`, `first_gt`, `last_lt` | have |
