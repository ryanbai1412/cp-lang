# Exemplars 16–25: syntax-first [proposed]

Each problem has two versions: **minimal stdlib** and **full stdlib**. The minimal one is written against spec p4 with: `List`, `Deque`, `Str`, `Mint`, `read`/`print`, and a few
folds. No `Graph`, `bfs`, `topo_order` or `components`; the syntax carries the weight. The algorithms are
model-checked against brute forces in `wf/ex16_21_check.py`; the `.cp` code is hand-written and has not run
(no compiler exists yet).

Every graph problem uses this two-line chunk, which a personal library would hold:

```cp
fn adjacency<V>(edges: List<(V, V)>) -> [V]List<V>:
    var adj: [V]List<V> = []
    for (u, v) in edges:
        adj[u].push(v)
        adj[v].push(u)
    return adj
```

---

## Part A: long, Split-like

### 22. Ehab's Last Theorem (CF 1325F)

Connected graph, n ≤ 1e5, m ≤ 2e5, s = ⌈√n⌉. Print a simple cycle of length ≥ s, or an independent set
of exactly s vertices. One DFS tree serves both: a back edge spanning ≥ s − 1 levels is the cycle;
otherwise every edge joins depths at most s − 2 apart, so a depth class mod s − 1 is independent.

**Minimal stdlib:**

```cp
with io(base: 1)

domain V = read(5..=1e5)
let m: (V.count - 1)..=2e5 = read()
let adj = adjacency(read<[m](V, V)>())
let s: 3..=317 = ceil_sqrt(V.count)

var depth: [V]?Int = none, parent: [V]?V = none

fn climb(from: V, to: V) -> List<V>:             // tree path, from → to
    var x = from, path = [from]
    while x != to:
        x = parent[x]!
        path.push(x)
    return path

fn long_cycle(v: V, d: Int) -> ?List<V>:
    depth[v] = d
    for w in adj[v]:
        if let dw = depth[w]:
            if d - dw + 1 >= s: return climb(from: v, to: w)
        else:
            parent[w] = v
            if let c = long_cycle(w, d + 1): return c
    return none

if let cycle = long_cycle(V.first, 0):
    print(2)
    print(cycle.len)
    print(cycle)
else:
    let depth = depth!
    var layer: [s - 1]List<V> = []
    for v in V: layer[depth[v] % (s - 1)].push(v)
    print(1)
    print(layer.max(by: c => c.len)[.. s])
```

**Full stdlib:**

```cp
with io(base: 1)

domain V = read(5..=1e5)
let m: (V.count - 1)..=2e5 = read()
let g = Graph<V>.undirected(edges: read<[m](V, V)>())
let s: 3..=317 = ceil_sqrt(V.count)

let t = dfs(from: [V.first], next: g.adj)                 // [new] the DFS twin of `bfs`
let depth = t.depth!
let span = (v, w) => depth[v] - depth[w] + 1
if let (v, w) = g.arcs.first(where: (v, w) => span(v, w) >= s):
    print(2)
    print(span(v, w))
    print(t.path(from: v, to: w))                         // up the tree
else:
    let layers = V.group(by: v => depth[v] % (s - 1))
    print(1)
    print(layers.max_by(len)[.. s])
```

**Syntax notes**
- `if let dw = depth[w]: … else: …` is the visited/unvisited split with no `-1`.
- `if let c = long_cycle(w, d + 1): return c` is "return the first success". It recurs in every
  search-with-early-exit. It's one line, so I'd keep it rather than add an operator.
- `let depth = depth!` after the DFS: every vertex is set (the graph is connected). Array `!` (§23.1,
  [proposed]) converts `[V]?Int` to `[V]Int` with one check and freezes it, so later uses drop the `!`.

### 23. Directing Edges (CF 1385E)

Multitest. Some edges are directed and some undirected; orient the undirected ones so the graph is acyclic,
or say NO. Topologically sort the directed part; orient every edge forward in that order.

**Minimal stdlib:**

```cp
with io(base: 1)

--- with multitest() ---

domain V = read(2..=2e5)
domain E = read(1..=2e5)
let edges: [E](Bool, V, V) = read()               // directed?, x, y

var indeg: [V]Int = 0, out: [V]List<V> = []
for (directed, x, y) in edges where directed:
    out[x].push(y)
    indeg[y] += 1

var ready: Deque<V> = V.filter(v => indeg[v] == 0)
var order: List<V> = []
while let v = ready.pop_front():
    order.push(v)
    for w in out[v]:
        indeg[w] -= 1
        if indeg[w] == 0: ready.push_back(w)

if order.len < V.count:
    print("NO")
    return

var rank: [V]Int = 0
for (k, v) in order.enumerate(): rank[v] = k

print("YES")
for (_, x, y) in edges:
    print(if rank[x] < rank[y]: (x, y) else: (y, x))
```

**Full stdlib:**

```cp
with io(base: 1)

--- with multitest() ---

domain V = read(2..=2e5)
domain E = read(1..=2e5)
let edges: [E](Bool, V, V) = read()

let g = Graph<V>.directed(edges: edges.filter((d, _, _) => d).map((_, x, y) => (x, y)))
let order = g.topo_order() ?? return print("NO")
let rank = order.inverse()                                // [V]Order
print("YES")
for (_, x, y) in edges:
    print(if rank[x] < rank[y]: (x, y) else: (y, x))
```

**Syntax notes**
- `for (directed, x, y) in edges where directed:` is the filter; `Bool` reads as `0`/`1`.
- `while let v = ready.pop_front():` is the whole worklist loop; no index variable.
- `return` below the wrap line returns from the per-test closure: the next test (16.3).
- **[new] `enumerate()`:** `(Int, T)` pairs for a list's integer positions. `numbered()` makes a fresh
  domain; this doesn't. Without it you write `var k = 0 … k += 1`.
- The final loop prints directed edges unchanged without a branch, because they already go forward.

### 24. Beautiful Graph (CF 1093D)

Multitest. Count ways to write 1, 2 or 3 on each vertex so every edge has an odd sum, mod 998244353. Each
component must be bipartite, with sides a and b; it contributes 2^a + 2^b.

**Minimal stdlib:**

```cp
with io(base: 1)
type Z = Mint(mod: 998244353)

--- with multitest() ---

domain V = read(1..=3e5)
let m: 0..=3e5 = read()
let adj = adjacency(read<[m](V, V)>())

var side: [V]?Bool = none
var ways = Z(1)
for root in V where side[root] == none:
    side[root] = false
    var count: [Bool]Int = 0
    var stack = [root]
    while let v = stack.pop():
        let sv = side[v]!
        count[sv] += 1
        for w in adj[v]:
            if side[w] == none:
                side[w] = not sv
                stack.push(w)
            elif side[w] == sv:
                print(0)
                return
    ways *= Z(2) ** count[false] + Z(2) ** count[true]
print(ways)
```

**Full stdlib:**

```cp
with io(base: 1)
type Z = Mint(mod: 998244353)

--- with multitest() ---

domain V = read(1..=3e5)
let m: 0..=3e5 = read()
let g = Graph<V>.undirected(edges: read<[m](V, V)>())

let side = g.two_coloring() ?? return print(0)            // [new] `?[V]Bool`, none if not bipartite
let (domain C, part) = g.components()
var count: [C][Bool]Int = 0
for v in V: count[part.of[v]][side[v]] += 1
print(C.map(c => Z(2) ** count[c][false] + Z(2) ** count[c][true]).product())
```

**Syntax notes**
- `[Bool]Int` holds the two side counts; there's no `cnt[2]` with 0/1 ints.
- `side[w] == sv` compares `?Bool` with `Bool` structurally; `none == sv` is false, which is what you
  want here.
- **`elif`** [adopted]. The spec never defined `else if`; `elif` (Python) avoids the extra indent level of a nested
  `else: if`.
- `print(0); return` in the middle of a component is safe because input was read up front. The lint for a
  `read` after `return` would fire otherwise.

### 25. Cover it! (CF 1176E)

Multitest, connected graph. Choose at most ⌊n/2⌋ vertices so every other vertex has a chosen neighbour:
BFS layers, then the smaller parity class.

**Minimal stdlib:**

```cp
with io(base: 1)

--- with multitest() ---

domain V = read(2..=2e5)
let m: (V.count - 1)..=2e5 = read()
let adj = adjacency(read<[m](V, V)>())

var depth: [V]?Int = none
depth[V.first] = 0
var queue: Deque<(V, Int)> = [(V.first, 0)]
while let (v, d) = queue.pop_front():
    for w in adj[v] where depth[w] == none:
        depth[w] = d + 1
        queue.push_back((w, d + 1))
let depth = depth!

let (even, odd) = V.split_by(v => depth[v] % 2 == 0)
let pick = if even.len <= odd.len: even else: odd
print(pick.len)
print(pick)
```

**Full stdlib:**

```cp
with io(base: 1)

--- with multitest() ---

domain V = read(2..=2e5)
let m: (V.count - 1)..=2e5 = read()
let g = Graph<V>.undirected(edges: read<[m](V, V)>())

let dist = bfs(from: [V.first], next: g.adj).dist!
let pick = V.split_by(v => dist[v] % 2 == 0).min_by(len)
print(pick.len)
print(pick)
```

**Syntax notes:** `let depth = depth!` closes the fill phase (see 22); the queue carries `d`, so the BFS
needs no `!` either. `split_by` returns both classes at once, instead of two
`filter`s with negated predicates.

---

## Part B: shorter, syntax-dense

### 16. Longest Regular Bracket Sequence (CF 5C)

Find the longest regular substring and how many there are; print `0 1` if none. The stack holds **cuts**:
where the current valid run could start.

**Minimal stdlib:**

```cp
let s: Str(len: 1..=1e6) = read()
var starts = [s.Cut.first]
var best = 0, ways = 1
for (ch, after) in zip(s, s.Cut.rest):
    if ch == '(':
        starts.push(after)
        continue
    starts.pop()
    if let start = starts.last:
        let len = after - start
        if len > best: (best, ways) = (len, 1)
        elif len == best: ways += 1
    else:
        starts.push(after)
print(best, if best == 0: 1 else: ways)
```

**Full stdlib:**

The stack of cuts is already the algorithm. The change is tallying at the end with `max_count`
(the `MaxCount` monoid, §22) instead of maintaining `(best, ways)`:

```cp
let s: Str(len: 1..=1e6) = read()
var starts = [s.Cut.first]
var lens: List<Int> = []
for (ch, after) in zip(s, s.Cut.rest):
    if ch == '(': starts.push(after); continue
    starts.pop()
    if let start = starts.last: lens.push(after - start)
    else: starts.push(after)
let (best, ways) = lens.max_count() ?? (0, 1)
print(best, ways)
```

**Syntax notes**
- `zip(s, s.Cut.rest)` pairs each byte with the cut after it; strict `zip` proves the lengths match.
- **[new] cut difference:** `after - start: Int` is the number of elements between two cuts of one domain.
  Cuts are counts, so subtraction is natural, and it appears in 16, 17 and 21. Today you'd write
  `after.ord - start.ord`.
- `(best, ways) = (len, 1)` is the "new max, reset count" idiom; with `elif` it reads cleanly.
- **[new] `max_count()` / `min_count() -> ?(T, Int)`:** the extremum and how often it occurs, in one pass.
  It replaces `best = max ?? 0` followed by `count(where: l => l == best)`: two passes, a lambda that only
  tests equality, and a `best == 0` special case. The empty case becomes the `??` default, and the
  problem's "0 1" is spelled exactly once. `MaxCount` is also the standard "max and count" segment-tree node.

### 17. Cashback (CF 940E)

Split the array into pieces; a piece of length k loses its ⌊k/c⌋ smallest elements. Minimize the total.
Only pieces of length 1 or c matter, so it's a DP over cuts with a sliding-window minimum.

**Minimal stdlib:**

```cp
let (n, c): (1..=1e5, 1..=1e5) = read()
domain A = n
let a: [A]Int = read()

var dp: [A.Cut]Int = 0, pre: [A.Cut]Int = 0
var mins: Deque<A> = []                           // window positions, values increasing
for x in A:
    pre[x.after] = pre[x.before] + a[x]
    while let y = mins.back where a[y] >= a[x]: mins.pop_back()
    mins.push_back(x)
    dp[x.after] = dp[x.before] + a[x]
    if let start = x.prev(c - 1):                 // the length-c piece ending at x
        while let f = mins.front where f < start: mins.pop_front()
        dp[x.after] min= dp[start.before] + (pre[x.after] - pre[start.before]) - a[mins.front!]
print(dp[A.Cut.last])
```

**Full stdlib:**

```cp
let (n, c): (1..=1e5, 1..=1e5) = read()
domain A = n
let a: [A]Int = read()

let pre = a.prefix_sums()                                 // [A.Cut]Int
let low = a.window_min(c)                                 // [new] [A]?Int: min of the length-c window ending here
var dp: [A.Cut]Int = 0
for x in A:
    dp[x.after] = dp[x.before] + a[x]
    if let start = x.prev(c - 1):
        dp[x.after] min= dp[start.before] + pre[x.after] - pre[start.before] - low[x]!
print(dp[A.Cut.last])
```

**Syntax notes**
- `while let y = mins.back where a[y] >= a[x]:` is the monotonic deque in one line.
- **[new] `x.prev(k) -> ?D`:** k steps back, `none` past the start. It generalizes `.prev` (= `.prev(1)`)
  and is the optional form beside the clamped `back(k)`. Here it doubles as the "window is full" test.
- `dp[start.before]`, `pre[x.after]`: the off-by-ones are visible choices, and `pre[x]` would be a type
  error.

### 18. Classy Numbers (CF 1036C)

Multitest: count integers in [L, R] (≤ 1e18) with at most 3 nonzero digits.

**Minimal stdlib:**

```cp
fn classy_upto(x: 0..=1e18) -> Int:
    fn free(slots: Int, budget: 0..=3) -> Int =
        (0..=min(budget, slots)).map(j => binom(slots, j) * 9 ** j).sum()
    let digits = x.digits()                       // most significant first
    var budget = 3, slots = digits.len
    var total = 0
    for d in digits:
        slots -= 1
        if d == 0: continue
        total += free(slots, budget)              // put 0 here
        if budget == 0: return total
        total += (d - 1) * free(slots, budget - 1)  // put 1 ..= d - 1 here
        budget -= 1                               // put d here and stay tight
    return total + 1                              // x itself

--- with multitest() ---

let (lo, hi): (1..=1e18, 1..=1e18) = read()
print(classy_upto(hi) - classy_upto(lo - 1))
```

**Full stdlib:**

The library version is the textbook digit DP with `@memo`, and needs no combinatorics:

```cp
fn classy_upto(x: 0..=1e18) -> Int:
    let digits = x.digits()
    @memo
    fn count(at: digits.Cut, left: 0..=3, tight: Bool) -> Int:
        let i = at.right ?? return 1
        let top = if tight: digits[i] else: 9
        return (0..=top).filter(d => d == 0 or left > 0)
            .map(d => count(i.after, if d == 0: left else: left - 1, tight and d == top))
            .sum()
    return count(digits.Cut.first, 3, true)

--- with multitest() ---

let (lo, hi): (1..=1e18, 1..=1e18) = read()
print(classy_upto(hi) - classy_upto(lo - 1))
```


**Syntax notes**
- A nested expression-bodied `fn free … =` is the natural helper; it captures nothing.
- `lo - 1` has type `0..=1e18 - 1` by interval arithmetic, so the call needs no check.
- `budget - 1` after `if budget == 0: return` is fine at runtime, but interval arithmetic doesn't see the
  guard: `budget` is a `var`, so it's `Int` and the argument is a checked narrowing. **Friction**, small: flow
  typing for range types is deliberately absent (no prover changes a type).

### 19. Covered Points Count (CF 1000C)

For each k, count integer points covered by exactly k of n segments (coordinates ≤ 1e18).

**Minimal stdlib:**

```cp
domain S = read(1..=2e5)
let segs: [S](Int, Int) = read()

var events: List<(Int, Int)> = []
for (l, r) in segs:
    events.push((l, +1))
    events.push((r + 1, -1))
events.sort()

var cnt: [0..=S.count]Int = 0
var cover = 0
for ((x, d), (y, _)) in events.pairs():
    cover += d
    cnt[cover] += y - x
print(cnt[1 ..])
```

**Full stdlib:**

```cp
domain S = read(1..=2e5)
let segs: [S](Int, Int) = read()

var delta: Map<Int, Int> = 0                              // ordered, default 0
for (l, r) in segs:
    delta[l] += 1
    delta[r + 1] -= 1

var cnt: [0..=S.count]Int = 0
var cover = 0
for ((x, d), (y, _)) in delta.pairs():
    cover += d
    cnt[cover] += y - x
print(cnt[1 ..])
```

**Syntax notes:** `pairs()` plus nested tuple patterns make the sweep three lines. No `prev` variable and
no special first iteration.

### 20. Tree Painting (CF 1187E)

Choose the first black vertex; each step paints a white vertex adjacent to black and scores the size of
its white component. Maximize: over all roots, the sum of subtree sizes. Rerooting: moving the root from
p to child c changes the sum by n − 2·size(c).

**Minimal stdlib:**

```cp
domain V = read(2..=2e5)
let adj = adjacency(read<[V.count - 1](V, V)>())
let n = V.count

var size: [V]Int = 1
fn down(v: V, from: ?V) -> Int:                   // sum of subtree sizes under v
    var total = 0
    for w in adj[v] where w != from:
        total += down(w, v)
        size[v] += size[w]
    return total + size[v]

var best = 0
fn reroot(v: V, from: ?V, score: Int):
    best max= score
    for w in adj[v] where w != from:
        reroot(w, v, score + n - 2 * size[w])

reroot(V.first, none, down(V.first, none))
print(best)
```

**Full stdlib:**

```cp
domain V = read(2..=2e5)
let t = Tree(read<[V.count - 1](V, V)>()).rooted(at: V.first)
let n = V.count

var size: [V]Int = 1
for v in t.at.rev:
    if let p = t.parent[v]: size[p] += size[v]

var score: [V]Int = size.sum()                            // at the root; overwritten below
for v in t.at:
    if let p = t.parent[v]: score[v] = score[p] + n - 2 * size[v]
print(score.max())
```

No recursion, so no depth question at all.

**Syntax notes**
- `where w != from` with `from: ?V`: `none` at the root compares unequal to every `w`, so no sentinel
  parent.
- Two nested functions mutate captured `size` and `best`; that's the chunking style working as intended.
- Hoisting: `reroot` captures `best`, which is initialized before the first call (checked).

### 21. Remove the Substring, hard (CF 1203D2)

t is a subsequence of s (|s| ≤ 2e5). Remove the longest contiguous substring of s so t stays a
subsequence. For every cut k of t: `lo[k]` is the earliest cut of s after greedily matching t's prefix,
and `hi[k]` the latest cut before matching its suffix. The answer is the widest gap `hi[k] − lo[k]`.

**Minimal stdlib:**

```cp
let s: Str(len: 1..=2e5) = read()
let t: Str(len: 1..=s.len) = read()

var lo: [t.Cut]s.Cut = s.Cut.first
var at = s.Cut.first
for (ch, k) in zip(t, t.Cut.rest):
    while s[at.right!] != ch: at = at.fwd(1)
    at = at.fwd(1)
    lo[k] = at

var hi: [t.Cut]s.Cut = s.Cut.last
at = s.Cut.last
for (ch, k) in zip(t, (.. t.Cut.last)).rev():
    while s[at.left!] != ch: at = at.back(1)
    at = at.back(1)
    hi[k] = at

print(t.Cut.map(k => hi[k] - lo[k]).max())
```

**Full stdlib:**

```cp
let s: Str(len: 1..=2e5) = read()
let t: Str(len: 1..=s.len) = read()

let lo = t.scan(s.Cut.first, (at, ch) => s.find(ch, from: at)!.after)
let hi = t.rscan(s.Cut.last, (at, ch) => s.rfind(ch, before: at)!.before)   // [new] scan from the right
print(t.Cut.map(k => hi[k] - lo[k]).max())
```

Both scans are indexed by `t.Cut` (the seed sits at the first or last cut), so `hi[k] - lo[k]` lines up
with no index shifting.

**Syntax notes**
- Indexing by cuts makes the ends uniform: removing a prefix (k = first) or suffix (k = last) is the same
  formula, with no separate `max(right[0], n - 1 - left[m - 1])`.
- `[t.Cut]s.Cut`: an array indexed by one string's cuts holding another's. Both are nominal, so swapping
  `lo[k]` for `lo[at]` is a type error.
- `.max()` is total because `t.Cut` is nonempty.
- `s[at.right!]` traps only if t isn't a subsequence, which the statement guarantees.

---

## Proposals from this round

New full-stdlib APIs, all library: `dfs(from:, next:)` (the twin of `bfs`, with `.depth`/`.parent`/`.path`),
`g.arcs` (both directions of each undirected edge), `g.two_coloring() -> ?[V]Bool`, `a.window_min(k)`,
`t.rscan`, `s.find(ch, from:)` / `s.rfind(ch, before:)`. Open: `?? return print("NO")` relies on `return`
taking a unit expression.

| # | proposal | seen in | layer |
|---|---|---|---|
| 1 | `c' - c: Int` for two cuts of one domain | 16, 17, 21 | library (`Sub` impl on `A.Cut`) |
| 2 | `x.prev(k)`, `x.next(k) -> ?D` | 17 | library |
| 3 | `elif` [adopted] | 16, 24 | core syntax |
| 4 | `xs.enumerate() -> (Int, T)` | 23 | library |
| 5 | `arr!` (the `Unwrap` trait on `[D]?T`) to close a fill phase; `all_some()` as the non-trapping form | 22, 25 | library (§23.1) |

Patterns that held with no new syntax: `while let x = q.pop…()` worklists, `for … where`, `if let … else`
visited splits, `[Bool]T`, `?V` parents, `return` from the top level ending one test, and nested functions mutating
captures.
