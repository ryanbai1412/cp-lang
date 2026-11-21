# Grounding: ideal solutions to nine varied problems

This is the first step of the redesign. Before writing the spec, I took nine
problems from ecnerwala's repository and Ryan's DMOJ submissions, chosen to
cover different shapes: grids, implicit trees, sliding windows, interactive
problems, geometry, DP with a data structure, a grader-function problem,
modular counting and trees.

For each one I first asked what the solution *is*, the way an editorial would
say it in one or two sentences, and then wrote the code that says that and
nothing else. The C++ was only used to check the algorithm. Several solutions
use a different algorithm from the original because the language makes a
cleaner one cheap.

The syntax here is a draft. Each problem ends with what it asks of the
language, and the last section collects the design decisions these examples
suggest and the open questions.

The guiding test for every line: *could this line be wrong in a way that
compiles?* If yes, the language should make that mistake impossible or
make the line unnecessary.

---

## 1. Orac and Game of Life (CF 1349C, ecnerwala)

**Problem.** An r × c grid of 0/1 cells. Each step, a cell with at least one
equal-coloured neighbour flips; others stay. Answer q queries "colour of
cell (i, j) after p steps", with p up to 1e18.

**The idea in one sentence.** A cell starts flipping at time t = its BFS
distance from the nearest cell that already has an equal neighbour, and from
then on it alternates.

The C++ is 5.3 KB, of which about 4 KB is a tensor library for 2D arrays
and the rest is index arithmetic, bounds tests and direction tables.

```
let (r, c, q): (1..=1000, 1..=1000, 1..=1e5) = read()
domain Cell = grid(r, c)                       // (row, col), 1-based, with .adj4 / .adj8
let g: [Cell]Bit = read(digits)                // r lines of c characters '0'/'1'

let seeds = Cell.filter(x => x.adj4.any(y => g[y] == g[x]))
let start: [Cell]?Int = bfs(from: seeds, next: x => x.adj4)

repeat q:
    let (x, p): (Cell, 1..=1e18) = read()
    print(match start[x]:
        none => g[x]
        some(t) => g[x] ^ (max(p - t, 0) % 2))
```

**What it asks of the language.**

- **Grid domains.** `grid(r, c)` is a domain of (row, col) pairs. `x.adj4`
  yields only neighbours *inside* the grid, so there is no bounds test or
  direction table to get wrong. Reading a `Cell` reads two 1-based numbers.
- **`bfs` as a stdlib function** returning `[D]?Int` distances from many
  sources. It is the textbook algorithm with an explicit name, not a hidden
  rewrite.
- **`repeat q:`** for "do this q times" without an unused loop variable.
- **`read(digits)`** for grids of characters.
- **Range-typed input** (`p: 1..=1e18`) tells the compiler `p - t` cannot
  overflow.

---

## 2. Drazil Likes Heap (CF 1329C, ecnerwala)

**Problem.** A max-heap of height h with distinct values at nodes
1..2^h − 1. Operation f(i) deletes the value at node i by repeatedly
pulling up the larger child. Do 2^h − 2^g deletions so that the remaining
values sit exactly in the top g levels with minimum sum, and print the sum
and the indices deleted.

**The idea.** Greedily, for each node of the top g levels in index order,
keep deleting it while the hole that deletion would leave ends below level
g. (ecnerwala instead merges sorted lists bottom-up; the greedy is shorter
and well known.)

```
@multitest
let (h, g): (2..=20, 1..=h - 1) = read()
domain Node = heap_tree(levels: h)          // 1..2^h-1 with .children, .depth (root depth 1)
var a: [Node]?Int = read()                  // none = empty slot

fn larger_child(v: Node) -> ?Node =
    v.children.filter(c => a[c] != none).max_by(c => a[c])

fn hole_after_delete(v: Node) -> Node:      // where the empty slot ends up
    var u = v
    while let c = larger_child(u): u = c
    u

fn delete(v: Node):
    var u = v
    while let c = larger_child(u):
        a[u] = a[c]
        u = c
    a[u] = none

var ops: List<Node>
for v in Node if v.depth <= g:
    while hole_after_delete(v).depth > g:
        delete(v)
        ops.push(v)

print(sum(a[v]! for v in Node if v.depth <= g))
print(ops)
```

**What it asks of the language.**

- **Implicit-tree domains.** `heap_tree(levels: h)` gives `2i` and `2i+1`
  names (`children`, `parent`, `depth`). Children that would fall outside
  the tree simply aren't yielded.
- **`?Int` with a free niche** as the "empty" marker instead of 0, so
  "empty" can't be confused with a real value.
- **Dependent range types** in input: `g: 1..=h - 1` refers to `h`.
- **`print(list)`** prints space-separated, so no `" \n"[i+1==n]` trick.

---

## 3. Longest Y (ABC 229 G, ecnerwala)

**Problem.** A string of `Y` and `.`; with at most k adjacent swaps, make
the longest possible block of consecutive `Y`s.

**The idea.** Subtract each Y's rank from its position. Gathering a window
of Ys into a block then costs the sum of distances to the window's median.
Cost is monotone in the window, so take the longest window that costs at
most k.

```
let s: Str = read()
let k: 0..=1e12 = read()

let p = [pos - rank for (rank, pos) in s.positions('Y').enumerate()]
let pre = prefix_sums(p)                         // pre.sum(range) in O(1)

fn cost(w: Range) -> Int:                        // gather p[w] to its median
    let m = w.mid
    (p[m] * (m - w.first) - pre.sum(w.first ..< m)) +
    (pre.sum(m ..= w.last) - p[m] * (w.last - m + 1))

print(longest_window(p.indices, ok: w => cost(w) <= k)?.len ?? 0)
```

**What it asks of the language.**

- **`prefix_sums(...).sum(range)`.** You never write `pre[r+1] - pre[l]`
  yourself, which is the most common off-by-one in CP.
- **`longest_window(seq, ok:)`.** Two pointers, given a predicate that is
  monotone in the window. It returns the window as a range, or `none` when
  even the empty case fails, so the case with no `Y`s at all is handled.
- **`w.mid`, `w.first`, `w.last`** on ranges. The median-index arithmetic
  isn't written by hand.
- **Open question:** the two half-sums are still hand-written. A
  `pre.dist_sum(w, to: p[m])` helper would remove them, but might be too
  specialised.

---

## 4. Guess the Permutation (CF 1588B, ecnerwala), interactive

**Problem.** The hidden permutation is 1..n with two adjacent blocks
[i, j−1] and [j, k] each reversed. A query `? l r` returns the number of
inversions in [l, r]. Find i, j, k in at most 40 queries; n ≤ 1e9.

**The idea.** Prefix inversion counts grow until position k and are flat
after it. Binary search for k. The last step's increase gives j, and one
more pair of queries gives i.

```
@multitest @interactive
let n: 4..=1e9 = read()

fn inv(r: 0..=n) -> Int =                         // inversions in 1..=r
    if r <= 1: 0 else: ask("? 1 {r}", as: Int)

let all = inv(n)
let k = bisect(1..=n, x => inv(x) == all)!       // first x where the prefix is complete
let j = k - (all - inv(k - 1))
let i = j - 1 - (inv(j - 1) - inv(j - 2))
answer("! {i} {j} {k}")
```

**What it asks of the language.**

- **`ask(line, as: T)`** prints, flushes and reads a typed reply in one
  call. `answer(line)` prints and flushes. In `@interactive` files,
  reading `-1` (the judge's "wrong") exits cleanly.
- **`bisect(range, pred)`.** Binary search is named explicitly and returns
  the first value where a false→true predicate holds, or `none`. It avoids
  the classic `lo/hi/mid` off-by-one, and the name says it is logarithmic.
- **`inv(0)` and `inv(1)` need no query.** The function's domain
  `0..=n` makes that case visible.

---

## 5. One Fourth (ABC 250 F, ecnerwala), geometry

**Problem.** A convex polygon with n vertices (|x|, |y| ≤ 4e8). Cut it
along a diagonal and eat one piece with area a. Minimise 8·|S/4 − a|,
where S is the total area.

**The idea.** Fix one end i of the cut. As the other end j moves round,
the area of p[i..=j] only grows, so binary search for where it crosses S/4
and check both neighbours. Everything is kept as integers by working in
units of 8× the area.

```
let n: 3..=1e5 = read()
domain V = ring(n)                                 // cyclic: v.next, v + 2 wrap around
let p: [V]Point<-4e8..=4e8> = read()

let total = sum(cross(p[v], p[v.next]) for v in V)                 // 2S
let edge = prefix_sums(V, v => cross(p[v], p[v.next]))              // cyclic arc sums

fn piece(i: V, j: V) -> Int =                      // 8 × area of p[i], ..., p[j]
    4 * (edge.sum(V.arc(from: i, until: j)) + cross(p[j], p[i]))

var best = total
for i in V:
    let js = V.arc(from: i + 2, through: i - 1)    // valid other ends
    let t = js.bisect_index(j => piece(i, j) >= total)
    for j in js.clipped(t - 1 ..= t):
        best min= abs(total - piece(i, j))
print(best)
```

**What it asks of the language.**

- **Cyclic domains.** In `ring(n)`, `+` and `.next` wrap, so `(i+1)%N`
  appears nowhere. `V.arc(from:, until:/through:)` is a cyclic range with
  the end inclusivity spelled out.
- **`prefix_sums` over a ring** answers arc sums.
- **Overflow proof from coordinate ranges.** The `Point<-4e8..=4e8>` bounds
  let the compiler prove `4 * (...)` fits in 64 bits (|8a| ≤ about 5.1e18).
  ecnerwala's version computes `4 * tot`, which would overflow under
  slightly larger bounds; here the compiler checks it.
- **`clipped(range)`** takes the neighbours t−1 and t without special-casing
  either end.
- **`min=`** is compound assignment for min. This choice is a deliberate
  departure from ecnerwala's O(n) two-pointer to an O(n log n) bisect: it
  states the monotonicity instead of maintaining it by hand. Both should be
  expressible; this one is harder to get wrong.

---

## 6. Tourism (CCC 2019 S4, Ryan)

**Problem.** n attractions in order, at most k per day, using the minimum
number of days ⌈n/k⌉. A day scores the maximum attraction it visits.
Maximise the total.

**The idea.** dp over prefixes: a prefix of b attractions must use exactly
⌈b/k⌉ days. dp[b] is the max of dp[j] + max(a(j..b]) over cut points j
that are one day earlier and at most k back. "max of dp[j] + max(a(j..b])"
is a known pattern: a monotone stack turns the running maximum into range
additions on a max segment tree.

Ryan's C++ is 4.6 KB with a hand-made deque-of-deques structure.

```
let (n, k): (1..=1e6, 1..=n) = read()
domain I = 1..=n
let a: [I]1..=1e9 = read()
domain Cut = I.bounds                              // 0..=n: number of attractions before the cut

fn day(b: Cut) -> Int = ceil_div(b.count, k)
fn day_cuts(d: Int) -> Range<Cut> =                // cuts that end day d
    Cut.clip((d - 1) * k + 1 ..= d * k) if d > 0 else Cut.first..=Cut.first

var dp: [Cut]Int = 0
var best = SegTree<Cut>(max, update: add)          // best[j] = dp[j] + max a(j..b]
var stack: List<(height: Int, cuts: Range<Cut>)>

for x in I:
    let b = x.after
    var cuts = x.before ..= x.before
    best[x.before] = dp[x.before] + a[x]
    while let top = stack.last() if top.height <= a[x]:
        best.update(top.cuts, a[x] - top.height)
        cuts = top.cuts.first ..= cuts.last
        stack.pop()
    stack.push((a[x], cuts))
    dp[b] = best.max(day_cuts(day(b) - 1) & Cut.clip(b - k ..< b))

print(dp[Cut.last])
```

**What it asks of the language.**

- **Positions vs cuts.** `I` is the attractions and `Cut = I.bounds` the
  gaps between them, with `x.before` and `x.after` moving from one to the
  other. `dp[x]` with an attraction doesn't compile, which is exactly the
  confusion between `dp[i]` and `dp[i+1]`.
- **Range intersection `&` and `Cut.clip(...)`.** The window is stated as
  "cuts ending the previous day" ∩ "within k", instead of a
  `max(i-K, (d-2)K+1, 0)` formula.
- **Friction (real):** `b.count` and `b - k` drop out of the domain into
  plain `Int` and come back through `clip`. That's honest, but it's the
  one place an off-by-one can still hide. Cut arithmetic needs a sharper
  rule (see open questions).
- **`while let ... if ...`** (a pattern with a guard) for the monotone
  stack.

---

## 7. Horses (IOI 2015, Ryan), grader-function problem

**Problem.** n years. Horses multiply by x[i] in year i and can be sold
for y[i] each. The answer is max over i of (x[0]·…·x[i])·y[i] mod 1e9+7,
maintained under point updates to x and y. The grader calls `init`,
`updateX` and `updateY`.

**The idea.** Only positions with x > 1 change the prefix product, and
after about 30 of them from the right the product exceeds any y. Walk those
positions right to left, keeping the best value relative to the current
prefix product; stop when it passes 1e9.

```
@grader(lang: ioi)                                 // the grader calls the exported fns
type M = Mod<1e9+7>

domain Year = 0..<n                                // n is fixed by init
var x: [Year]1..=1e9
var y: [Year]1..=1e9
var big: SortedSet<Year>                           // years with x > 1
var prod = SegTree<Year>(M.mul)
var top = SegTree<Year>(max)

fn answer() -> Int:
    var rel = 0                                    // best value ÷ prefix product here
    var end = Year.end
    for s in big.rev().chain([Year.first]):
        rel = max(rel, top.max(s ..< end))
        if s == Year.first or rel * x[s] > 1e9:
            return (prod.fold(Year.first ..= s) * rel).int
        rel *= x[s]
        end = s

export fn init(n0: Int, x0: [Year]Int, y0: [Year]Int) -> Int:
    n = n0; x = x0; y = y0
    big = Year.filter(i => x[i] > 1).collect()
    prod = SegTree(x.map(M)); top = SegTree(y)
    answer()

export fn update_x(i: Year, v: 1..=1e9) -> Int:
    big.set(i, v > 1); x[i] = v; prod[i] = M(v)
    answer()

export fn update_y(i: Year, v: 1..=1e9) -> Int:
    y[i] = v; top[i] = v
    answer()
```

**What it asks of the language.**

- **Grader mode.** `export fn` matches the grader's names (with snake_case
  mapped to `updateX`) and parameter types. A `Year` parameter means the
  grader's raw int is range-checked on entry.
- **Domains sized at run time by `init`.** Friction: `n` is assigned inside
  `init`, but `domain Year` is declared at the top. Grader problems need
  "module state initialised once", and the right construct is an open
  question.
- **`big.set(i, flag)`** inserts or removes depending on a boolean, so you
  don't write `if old > 1: erase; ...; if new > 1: insert`.
- **The `rel` recurrence** fits in an `Int` because it's capped at 1e9
  before multiplying by x ≤ 1e9. Range analysis can't see that cap through
  `return`, so this is a case where the compiler keeps a checked
  multiplication. That's acceptable.

---

## 8. Bouquet (ABC 156 D, ecnerwala), modular counting

**Problem.** Count non-empty subsets of n ≤ 1e9 flowers whose size is
neither a nor b (a, b ≤ 2e5), mod 1e9+7.

```
let (n, a, b): (2..=1e9, 1..=min(n, 2e5), a + 1..=min(n, 2e5)) = read()
type M = Mod<1e9+7>
print(M(2) ** n - 1 - M.falling(n, a) / M.fact(a) - M.falling(n, b) / M.fact(b))
```

**What it asks of the language.**

- **`Mod<P>`** with `**`, `/` (inverse, P known prime at compile time), and
  integer literals converting implicitly *only* into `Mod`.
- **`M.falling(n, k)`** instead of `binom(n, k)`, because n is huge and k
  is small. The cost (O(k)) is visible in the name, and a table-based
  `binom` would need n ≤ table size. Nothing picks an algorithm in secret.

---

## 9. Tree Distances II (CSES 1133)

**Problem.** For every node of a tree, the sum of distances to all other
nodes.

**The idea.** Rerooting: the answer at the root is the sum of depths, and
moving the root across edge (p, c) changes it by n − 2·size[c].

```
let n: 1..=2e5 = read()
domain V = 1..=n
let t = Tree<V>(read(n - 1)).rooted(at: V.first)

let size = t.subtree_sizes()
var ans: [V]Int
ans[t.root] = sum(t.depth[v] for v in V)
for (p, c) in t.down_edges():                      // preorder, so ans[p] is ready
    ans[c] = ans[p] + n - 2 * size[c]
print(ans)
```

**What it asks of the language.**

- **`Tree<V>(edges).rooted(at:)`** gives `parent`, `depth`, `preorder` and
  `down_edges()` without writing a DFS. A deep recursion that blows the
  stack on a path graph is the classic bug this avoids.
- **`down_edges()` order guarantees.** The comment states the invariant
  the loop relies on, and the stdlib documents it. This is the kind of
  ordering fact a tree API must promise explicitly.

---

## What these nine suggest for the spec

1. **Most bugs live in index translation, and the fix is domains plus
   named views, not arithmetic.** Grids (`adj4`), implicit trees
   (`children`), rings (`next`, `arc`), cuts (`before`/`after`), prefix sums
   (`sum(range)`), windows (`longest_window`) and binary search (`bisect`)
   each delete a family of `±1` and `% n` bugs. The language core needs
   only domains and ranges; each shape is a small stdlib domain.
2. **Name the algorithm, never pick it secretly.** `bfs`, `bisect`,
   `longest_window`, `falling` and `SegTree` are explicit. The compiler
   specializes them but never swaps them for a different algorithm.
3. **Input types carry the proofs.** Range-typed reads give overflow
   freedom (problems 1, 5, 6), validation, storage width and stress-test
   generators from one annotation.
4. **Small syntax that earns its place:** `repeat q:`, `min=`/`max=`,
   `while let ... if ...`, `match` with `some(t)`, `?T` with `!` and `??`,
   `print` of lists, string interpolation in `ask`.
5. **Program modes are file decorators:** `@multitest`, `@interactive`,
   `@grader`. Each changes how the file is driven, never what the code
   means.

## Open questions this round raised

1. **Cut arithmetic (Tourism).** Should `b - k` on a bounds domain produce
   a clipped range endpoint directly (`Cut.clip(b - k ..< b)` becomes
   `b.back(k) ..< b`)? Or should `b.count` stay the only way out?
2. **Grader state (Horses).** How is a domain whose size is set by `init`
   declared? One option is a `state:` block run once by `init`.
3. **Specialised helpers.** Are `dist_sum` or a "max dp + range max"
   structure stdlib material, or too narrow? My lean is no: keep the stdlib
   at reusable shapes, not problem patterns.
4. **Two-pointer vs bisect (One Fourth).** Is preferring the clearer
   O(n log n) version the language's default, or should two pointers be
   just as easy to write? I want both to be pleasant.
