# CP Lang: ten more blind dreams

Picked at random from the blind dreams (the `dream.cp` files), keeping the hard ones. Each is written from the C++ and the idea without checking the spec. **vs spec** compares each dream line by line with its spec-compliant port: what each wrote, which form won and why, and what is still open. Statuses are as of the latest push (tagged variants, `mut` lambda parameters and enum/product `.at` now check).

## cco20p4: CCO 2020 "Travelling Salesperson"

**Problem.** Every pair of n cities is joined by a red or a blue road. For each starting city, print a closed walk through every city whose road colours change at most once.

**Sketch.** In a complete graph one of the two colour graphs is connected. Walk a DFS tree of that colour there and back (an Euler tour, all one colour) and rotate the tour to each start.

```rust
# CCO 2020 P4 Travelling Salesperson: every pair of cities is joined by a red or blue road.
# For each start, give a closed walk visiting every city that changes colour at most once.
# One of the two colour graphs of a complete graph is connected; walk its DFS tree there and back
# (an Euler tour, a single colour) and rotate it to each start.
enum Colour = 'R' Red | 'B' Blue
domain V = [read Int]
let road = read lower_triangle [V, V]Colour   # row v lists v's roads to earlier cities
let tour = [Red, Blue].first_some(c =>
    let t = dfs(u => V.filter(v => v != u and road[u, v] == c), from: V.first).euler_tour()
    if t.distinct().len == V.count: some(t.but_last) else: none)!
for v in V:
    let walk = tour.rotated(to: tour.index_of(v)!)
    print(walk.len)
    print(walk)
```

**Spec port** (`spec.cp`, passes `cpl check`)

```rust
// cco20p4: every pair of n towns is joined by a red or a blue road. One colour connects all
// towns; tour a spanning tree of it, a closed walk, and print it rotated to start at each town.
let n = read<2..=2_000>()
domain V = 0..n
let rows = read<[V.drop(1)]Str>()
fn red(u: V, v: V) -> Bool = rows[max(u, v)][Int(min(u, v))] == 'R'
fn in_colour(c: Bool) = (u: V) => V.filter(v => v != u and red(u, v) == c)
let c = reachable(in_colour(true), from: [V.first!]).count() == n
var walk = List(Rooted(root: V.first!, next: in_colour(c)).euler_tour())
walk.pop()
for v in V:
    print(walk.len)
    print(walk.rotated_left(walk.keys.first(i => walk[i] == v)!))
```

**vs spec**

| Dream | Port (`spec.cp`) | Verdict |
|---|---|---|
| `enum Colour = 'R' Red \| 'B' Blue` | `fn red(u, v) -> Bool` over raw `Str` rows | Parser gap at the time; `@tag("R") Red` now parses and checks |
| `read lower_triangle [V, V]Colour` | `read<[V.drop(1)]Str>()` plus `rows[max(u, v)][Int(min(u, v))]` | Rejected |
| `[Red, Blue].first_some(c => ...)` | `let c = reachable(in_colour(true), ...).count() == n` | Port is better |
| `dfs(...).euler_tour()` | `Rooted(root:, next:).euler_tour()` | Spec form |
| `tour.rotated(to: tour.index_of(v)!)` | `walk.rotated_left(walk.keys.first(i => walk[i] == v)!)` | Rejected `rotated(to:)` |

- **Read mode rejected:** only 2 of the 100 problems (here and egoi23p4) read a triangle. A read mode is a core-syntax cost, so the cost lands in the port as a `min`/`max` index. If a third problem shows up, revisit as a library `read_triangle<T>()` rather than syntax.
- **Port is better on colour choice:** in a complete graph, if red is not connected then blue is. So one reachability test picks the colour, while the dream tours both and checks distinctness.
- **`rotated(to:)` rejected:** it's `rotated_left` plus a search. But `walk.keys.first(i => walk[i] == v)!` is long. The general fix is `walk.index_of(v)`, which the dream also wanted and is worth adding to `Seq` (open).
- **Follow-up:** now that tagged variants check, the port should go back to `enum Colour` and read `[V.drop(1)]List<Colour>`.

## cco21p5: CCO 2021 "Bread First Search"

**Problem.** Add the fewest edges to a graph so that 0, 1, …, n−1 is a valid BFS order from 0.

**Sketch.** BFS layers are consecutive blocks. A block starting at i pays one new edge for each vertex in it whose earliest neighbour is not before i. The next block may start at cut c only if no vertex from c on was found before i. DP from the right with a range-add, range-min segment tree over cuts.

```rust
// CCO 2021 "Bread First Search": add the fewest edges so that 0, 1, .., n-1
// is a BFS order from 0. Layers are consecutive blocks. A block starting at i
// pays one edge per vertex in it whose earliest neighbour is not before i, and
// the next block may start at cut c only if no vertex from c on has an earliest
// neighbour before i (it would have been found sooner).
let (n, m) = read<(1..=2e5, 0..=2e5)>()
domain V = [n]
var earliest: [V]?V = []
for e in read<[m](u: V, v: V)>():
    let (lo, hi) = minmax(e.u, e.v)
    earliest[hi] = min(earliest[hi], some(lo), none: last)
let found_by = V.group(by: v => earliest[v])
let later = earliest.suffix_min(none: last)

var best = SegTree(min, [V.cuts] c => if c == V.cuts.last!: 0 else: inf, act: add)
for i in V.drop(1).rev:
    for v in found_by[some(i)]:
        best.apply(v.r.., 1)
    if earliest[i] == none:
        best.apply(i.r.., 1)
    let next = V.cuts.first_true(c => later[c].is_none_or(e => e >= i))!
    best[i.l] = best.fold(max(next, i.r)..) 
print(best[V.first!.next!.l])
```

**Spec port** (`spec.cp`, passes `cpl check`)

```rust
// cco21p5 Bread First Search: low[v] is v's smallest earlier neighbour, or v itself. dp[c], the
// fewest edges to add when the vertices after cut c still have to be placed, picks the next
// block; each later vertex whose low is still too small costs one, kept as range adds over the
// cuts on a min segment tree. The block may run up to the first cut after which every low is
// at least v, found by binary search on the suffix minimum.
let (n, m) = read<(2..=200_000, 0..=200_000)>()
domain V = 1..=n
var low: [V]V = [V] v => v
for (u, v) in read<[m](V, V)>():
    low[max(u, v)] min= min(u, v)
var opened: [V]List<V> = []
for v in V if low[v] != v:
    opened[low[v]].push(v)
let suffix = low.rev.scan(min)
var dp = SegTree(min, [V.cuts] c => if c == V.cuts.last!: 0 else: inf, act: add)
for v in V.drop(1).rev:
    for a in opened[v]:
        dp.apply(a.r.., 1)
    var cur = dp.fold(suffix.partition_point(s => s < v)..)
    if low[v] == v:
        cur += 1
        dp.apply(v.r.., 1)
    dp.set(v.l, cur)
print(dp.get(V.first!.r))
```

**vs spec**

- **Matches the spec:** the dream checks as written. Domains, cuts (`v.r..`, `V.cuts.first_true`), `group(by:)`, `suffix_min` and `SegTree(min, ..., act: add)` all exist in the form the dream guessed.
- **What it needed:** `inf` as a documented `Int` constant in the prelude.
- **Lesson:** cut-indexed DP over a segment tree is the area where the spec matched blind intuition best. The cuts/positions split paid off here: `best[i.l]` and `best.apply(v.r.., 1)` have no ±1 anywhere.

## apio23p1: APIO 2023 "Cyberland"

**Problem.** Reach city H from 0. Some cities reset the elapsed time to 0, some halve it (at most k halvings), and arriving at H ends the trip. Minimise the time.

**Sketch.** Layered Dijkstra: each layer is one more halving, and more than about 70 never helps. Cities that reset are extra sources, and H is a stop vertex: you can reach it but never leave it.

```rust
// APIO 2023 "Cyberland": reach H from 0; some cities reset the elapsed time to
// 0, some halve it (at most k uses, and more than ~70 never helps). Arriving at
// H ends the trip. Layered Dijkstra: each layer is one more halving.
enum Power = Reset | Plain | Halve

@grader
fn solve(n: Int, m: Int, k: Int, h: Int, x: List<Int>, y: List<Int>, c: List<Int>, arr: List<Int>) -> F64:
    domain V = [n]
    let power = [V] v => Power.at(arr[v])!
    let goal = V(h)
    let roads = Graph<V>.undirected([x.keys] e => (u: V(x[e]), v: V(y[e]), w: F64(c[e])))
    let trip = roads.only(from: u => u != goal)
    let seen = reachable(trip, from: [V(0)])
    if not seen[goal]:
        return -1
    let starts = chain([V(0)], V.filter(v => seen[v] and power[v] == Reset))
    var dist = dijkstra(trip, from: starts.map(v => (v, 0.0))).dist
    for _ in 0..min(k, 70):
        let halved = V.filter(v => power[v] == Halve).flat_map(v =>
            dist[v].map(d => trip[v].map(a => (a.to, d / 2 + a.w))) ?? [])
        dist = dijkstra(trip, from: chain(dist.items.filter_map((v, d) => d.map(d => (v, d))), halved)).dist
    dist[goal]!
```

**Spec port** (`spec.cp`, passes `cpl check`)

```rust
// APIO 2023 "Cyberland": reach H from 0; some cities reset the elapsed time to
// 0, some halve it (at most k uses, and more than ~70 never helps). Arriving at
// H ends the trip. Layered Dijkstra: each layer is one more halving.

@grader
fn solve(n: Int, m: Int, k: Int, h: Int, x: List<Int>, y: List<Int>, c: List<Int>, arr: List<Int>) -> F64:
    domain V = [n]
    let goal = V(h)
    let roads = Graph<V>.undirected([x.keys] e => (u: V(x[e]), v: V(y[e]), w: F64(c[e])))
    let trip = roads.only(from: u => u != goal)
    let seen = reachable(trip, from: [V(0)])
    if not seen[goal]:
        return -1
    let starts = chain([V(0)], V.filter(v => seen[v] and arr[Int(v)] == 0))
    var dist = dijkstra(trip, from: starts.map(v => (v, 0.0))).dist
    for _ in 0..min(k, 70):
        let halved = V.filter(v => arr[Int(v)] == 2).flat_map(v =>
            dist[v].map(d => trip[v].map(a => (a.to, d / 2 + a.w))) ?? [])
        dist = dijkstra(trip, from: chain(dist.items.filter_map((v, d) => d.map(d => (v, d))), halved)).dist
    dist[goal]!
```

**vs spec**

| Dream | Port | Verdict |
|---|---|---|
| `roads.only(from: u => u != goal)` | same | **Spec change** |
| `enum Power = Reset \| Plain \| Halve`, `Power.at(arr[v])!` | `arr[Int(v)] == 0`, `arr[Int(v)] == 2` | Port worse: enum `.at` was a gap |
| `flat_map` over `dist[v].map(...) ?? []` | same | Dream sloppiness kept |

- **Spec change, `g.only(from: f)`:** "you can reach H but never leave it" is a filter on which vertices may emit arcs. `g.only(keep:)` already filtered vertices, and `from:` adds the outgoing-only filter. It is general: absorbing states, sinks and goal vertices all use it.
- **Port fell back to magic numbers:** `Power.at` didn't check, so the port compares the raw grader array to 0 and 2. Enum `.at` checks now, so the port should go back to `power[v] == Power.Reset`.
- **Halving step:** both versions build the next layer's sources with `flat_map` over an optional. A comprehension (`[(a.to, d / 2 + a.w) for v in V if power[v] == Halve, let d = dist[v]?, a in trip[v]]`) reads better. That needs `let x = e?` guards in comprehensions, which is noted as a candidate.

## coci21c2p5: COCI 2021/22 #2 "Osumnjičeni"

**Problem.** Suspects stand in a row, each with a height range. A lineup is a contiguous run whose ranges pairwise don't overlap. For each query l..=r, what is the fewest lineups that cover suspects l..=r?

**Sketch.** Two pointers with a range-add, max segment tree over compressed heights find the longest good run starting at each suspect. Binary lifting over "next start" then counts the greedy jumps.

```rust
// COCI 2021/22 C2 P5 Osumnjičeni: suspects stand in order with height ranges. A lineup is
// a contiguous run whose ranges pairwise don't overlap. For each query l..=r, how few lineups
// cover the suspects l..=r?
domain S = [read Int]
let height = read [S](lo: Int, hi: Int)
let (domain H, _) = compress(height.flat_map(h => [h.lo, h.hi]))
let span = height.map(h => H(h.lo)..=H(h.hi))

let cover = track(SegTree(max, [H]0, act: add),
    add: (mut t, s) => t.apply(span[s], 1),
    remove: (mut t, s) => t.apply(span[s], -1))
let lineup = S.longest_from(cover, ok: t => t.all <= 1)   // longest good run starting at each s
let jump = Lift.of(s => lineup[s].end.r)

for (l, r) in read [read Int](S, S):
    print(1 + jump.climb(l, ok: s => s <= r))
```

**Spec port** (`spec.cp`, passes `cpl check`)

```rust
// coci21c2p5 Osumnjičeni: how many groups of consecutive suspects, each with pairwise disjoint
// height intervals, cover l..=r. Two pointers keyed by left end give the longest good group
// from each suspect; greedy lineups jump from one group's end to the next start.
let n = read<1..=200_000>()
domain S = 1..=n
let iv = read<[S](l: 1..=1_000_000_000, r: 1..=1_000_000_000)>()
let (domain C, xs) = compress(chain(iv.map(x => x.l), iv.map(x => x.r)))
fn heights(s: S) -> Range<C> = xs.at(iv[s].l)..=xs.at(iv[s].r)
let held = track(SegTree(max, [C] c => 0, act: add),
    add: (mut cover, s) => cover.apply(heights(s), 1),
    remove: (mut cover, s) => cover.apply(heights(s), -1))
let group = iv.keys.maximal(held, ok: h => h.state.all <= 1, anchor: Anchor.Start)
let jumps = Lift.of(s => group[s].last!.next)
for (l, r) in read<[read<1..=200_000>()](S, S)>():
    print(jumps.climb(l, ok: s => s <= r).steps + 1)
```

**vs spec**

| Dream | Port | Verdict |
|---|---|---|
| `S.longest_from(cover, ok:)` | `iv.keys.maximal(held, ok:, anchor: Anchor.Start)` | **Spec change (decision 155)** |
| `Lift.of(s => lineup[s].end.r)` | `Lift.of(s => group[s].last!.next)` | Dream bug |
| `(mut t, s) => t.apply(...)` | same | Parser gap, now checks |
| `compress(height.flat_map(...))` and `H(h.lo)` | `compress(chain(...))` and `xs.at(iv[s].l)` | Spec form |
| `1 + jump.climb(l, ok:)` | `jumps.climb(l, ok:).steps + 1` | Spec form |

- **Why not `longest_from`:** "the longest good window starting at s" is the same two-pointer pass as `maximal`, only keyed by start instead of end. One `anchor:` option covers it without a second function, and every `maximal`/`minimal` (ring forms too) gets it.
- **Dream bug:** `.end.r` is a cut, not a position, and the jump needs the next suspect, `last!.next`. The type system caught this: `Lift<S>` wants `S -> ?S`, not a cut.
- **`track(state, add:, remove:)`:** both versions already agreed on it. A segment tree as a two-pointer state needed no special support.

## cmt21r2ztr: Median Constraints

**Problem.** Find a[1..n] in 1..=1e9 so that median(a[i], a[j], a[k]) = x for each query, or say NO.

**Sketch.** For every (v, x) a query names, make two 2-SAT literals: "a[v] < x" and "a[v] > x". At most one of a query's three values is below x and at most one is above. Along one vertex's thresholds in increasing order the facts are monotone. Solve, then read each value off its literals.

```rust
# Median constraints: find a[1..n] in 1..=1e9 with median(a[i], a[j], a[k]) = x for each query, or
# say NO. For every (v, x) a query names, keep two facts: "a[v] < x" and "a[v] > x". At most one of
# a query's three values is below x and at most one above. Along one vertex's thresholds in
# increasing x, the facts are monotone: "a[v] > x" for one threshold means "a[v] >= next", and so on.
# That is 2-SAT.
domain V = [read Int]
let asks = read [read Int] (i: V, j: V, k: V, x: Int)
let pins = [V] v => asks.filter(q => v in [q.i, q.j, q.k]).map(.x).distinct().sorted()
var sat = TwoSat()
let below = [V] v => pins[v].keyed(by: x => x).map(_ => sat.var())
let above = [V] v => pins[v].keyed(by: x => x).map(_ => sat.var())
for q in asks:
    for (u, w) in [q.i, q.j, q.k].pairs():
        sat.at_most_one(below[u][q.x], below[w][q.x])
        sat.at_most_one(above[u][q.x], above[w][q.x])
for v in V:
    for x in pins[v]: sat.at_most_one(below[v][x], above[v][x])
    for (x, y) in pins[v].with_next():
        sat.exactly_one(above[v][x], below[v][y])
match sat.solve():
    none => print("NO")
    some(truth) =>
        print("YES")
        print(V.map(v =>
            let lo = max(1, pins[v].filter(x => not truth[below[v][x]]).map(x => if truth[above[v][x]]: x + 1 else: x).max() ?? 1)
            lo))
```

**Spec port** (`spec.cp`, passes `cpl check`)

```rust
// cmt21r2ztr: find a_1..a_n with median(a_i, a_j, a_k) = x for every query. Each (vertex, x)
// that some query names is a pin with two variables, "a_v < x" and "a_v > x". A median of x
// lets at most one of the three be below and at most one above; along one vertex's pins in
// increasing x, "above the last" and "below the next" are exactly one of the two.
enum Side: Below, Above
let (n, m) = read<(1..=100_000, 1..=100_000)>()
domain V = 1..=n
let qs = read<[m](i: V, j: V, k: V, x: 1..=1_000_000_000)>()
let (domain P, pins) = compress(qs.flat_map(q => [q.i, q.j, q.k].map(v => (v: v, x: q.x))))
fn on(p: P, s: Side) = Lit((p, s), true)
var sat = TwoSat<(P, Side)>()
for q in qs:
    let ps = [q.i, q.j, q.k].map(v => pins.at((v: v, x: q.x)))
    for s in Side:
        sat.at_most_one(ps.map(p => on(p, s)).collect())
for p in P:
    sat.add_or(on(p, Below).flip, on(p, Above).flip)
for (p, q) in P.adjacent if pins.get(p).v == pins.get(q).v:
    sat.add_xor(on(p, Above), on(q, Below))
if let value = sat.solve():
    print("YES")
    var a: [V]Int = 1
    for p in P:
        let pin = pins.get(p)
        if value[(p, Above)]:
            a[pin.v] max= pin.x + 1
        elif not value[(p, Below)]:
            a[pin.v] max= pin.x
    print(a)
else:
    print("NO")
```

**vs spec**

| Dream | Port | Verdict |
|---|---|---|
| `below`/`above`: per-vertex `[V]` maps of `sat.var()` | `compress` (v, x) pins to `domain P`; `TwoSat<(P, Side)>` | Rejected the dream's shape |
| `[q.i, q.j, q.k].pairs()` + `at_most_one(a, b)` | `sat.at_most_one(list)` | Port is better |
| `pins[v].with_next()` per vertex | `P.adjacent if pins.get(p).v == pins.get(q).v` | Spec form |
| `match sat.solve(): none => ...` | `if let value = sat.solve(): ... else:` | Equivalent |

- **Keyed literals rejected:** the dream builds two nested maps of fresh variables. Compressing the (vertex, threshold) pairs gives one domain `P`, and a literal is just `(p, Side)`, so `TwoSat` is typed by its literal domain. Because `compress` sorts, one vertex's thresholds sit next to each other, and the monotone chain is `P.adjacent` filtered to the same vertex.
- **`pairs()` (decision 154):** the dream meant every pair of the three vertices, but at the time `pairs()` meant adjacent pairs, so the dream silently dropped (i, k). Four dreams made the same misreading, which is why `pairs()` now means every pair and adjacent pairs are `a.adjacent`. The port avoids the issue with `at_most_one` over a list.
- **`enum Side: Below, Above`:** the port names the two facts with an enum instead of two parallel arrays. Product domains of an enum (`(P, Side)`) worked here without extra support.

## dmpg16g2: DMPG 2016 "G2"

**Problem.** An n×n grid of letters A–E and dots. Count the subrectangles that contain all five letters.

**Sketch.** Inclusion-exclusion over which letters are forced absent. Fix the bottom row and move the top row down. Letters whose last sighting in a column is above the top drop out. For each subset, a DSU over clear column runs keeps Σ len·(len+1)/2 as runs only merge.

```rust
// DMPG 2016 G2: an n×n grid of letters A–E and dots. Count subrectangles holding all five letters.
// Inclusion–exclusion over the set of letters forced absent. Fix the bottom row; as the top row
// moves down, letters whose last sighting in a column is above the top drop out, and each
// subset's count of all-clear column runs only grows: a DSU of clear runs keeps Σ len·(len+1)/2.
enum Letter: A = 'A', B = 'B', C = 'C', D = 'D', E = 'E'
domain R = [read Int]
domain Col = [R.len]
let grid = read [R, Col]?Letter

struct ClearRuns(blocked: [Col]Int):
    runs: Dsu<Col> = Dsu()
    pairs: Int = 0
    fn init(mut self):
        for run in self.blocked.runs(by: b => b == 0).filter(r => self.blocked[r.first!] == 0):
            self.pairs += run.len * (run.len + 1) / 2
            for (x, y) in run.keys.adjacent: self.runs.union(x, y)
    fn unblock(mut self, c: Col):
        self.blocked[c] -= 1
        if self.blocked[c] > 0: return
        self.pairs += 1
        for d in [c.prev, c.next]:
            if d is some(d) and self.blocked[d] == 0 and not self.runs.same(c, d):
                self.pairs += self.runs.size(c) * self.runs.size(d)
                self.runs.union(c, d)

var total = 0
var last: [Col, Letter]?R = none
for bottom in R:
    for c in Col: if grid[bottom, c] is some(l): last[c, l] = some(bottom)
    var count = [for S in Letter.subsets: ClearRuns([for c in Col: S.count(l => last[c, l] != none)])]
    let leaving = R.group(last.items.filter_map(((c, l), r) => r.map(r => (r, (c, l)))))
    for top in ..=bottom:
        total += count.items.sum(of: (S, runs) => (-1) ** S.len * runs.pairs)
        for (c, l) in leaving[top]:
            for S in Letter.subsets.filter(S => l in S): count[S].unblock(c)
print(total)
```

**Spec port** (`spec.cp`, passes `cpl check`, except `Letter.subsets`)

```rust
// dmpg16g2: count the sub-rectangles holding all five letters, by inclusion-exclusion over the
// letters left out. Fix the bottom row; for each set of letters, a column is free while none of
// them occurs in it between the top and bottom rows. Raising the top frees columns one at a time,
// and the free runs merge in a Dsu, so each set keeps its count of all-free column spans.
let n = read<1..=500>()
domain Row = [n]
domain Col = [n]
let g = read<[Row, Col]Char>()
enum Letter: A, B, C, D, E
fn spans(k: Int) -> Int = k * (k + 1) / 2
struct FreeSpans:
    var blocked: [Col]Int
    var runs: Dsu<Col> = Dsu<Col>()
    var total = 0

    fn new(blocked: [Col]Int) -> FreeSpans:
        var s = FreeSpans(blocked: [Col] c => blocked[c] + 1)
        for c in Col:
            s.free(c)
        s

    fn free(mut self, c: Col):
        self.blocked[c] -= 1
        if self.blocked[c] > 0:
            return
        for d in [c.prev, c.next]:
            if let side = d and self.blocked[side] == 0:
                self.total -= spans(self.runs.size(side))
                self.runs.unite(side, c)
        self.total += spans(self.runs.size(c))
var last: [Col, Letter]?Row = None
var count = 0
for bot in Row:
    for c in Col:
        if let ch = Letter.at(Int(g[bot, c]) - Int('A')):
            last[c, ch] = bot
    var free = [Letter.subsets] s => FreeSpans.new([Col] c => s.items.count(ch => last[c, ch] != None))
    let leaving = last.buckets()
    for top in ..=bot:
        count += Letter.subsets.sum(of: s => if s.len % 2 == 0: free[s].total else: -free[s].total)
        for (c, ch) in leaving[Some(top)]:
            for s in Letter.subsets.filter(s => ch in s):
                free[s].free(c)
print(count)
```

**vs spec**

| Dream | Port | Verdict |
|---|---|---|
| `enum Letter: A = 'A', ...` + `read [R, Col]?Letter` | `read<[Row, Col]Char>()` + `Letter.at(Int(ch) - Int('A'))` | Port worse |
| `Letter.subsets`, `[for S in Letter.subsets: ...]` | `[Letter.subsets]FreeSpans` | Both need it; still a gap |
| `R.group(pairs)` into `[R]List` | `last.buckets()`, read as `leaving[Some(top)]` | **Spec change (decision 158)**; first rejected, then adopted |
| `count[S]` on a list | `free[s]` on `[Letter.subsets]` | Dream bug |
| `count.items.sum(of: (S, runs) => (-1) ** S.len * runs.pairs)` | `Letter.subsets.sum(of: s => if s.len % 2 == 0: ... else: ...)` | Port fixed to `sum(of:)` |

- **Last failing port:** `Letter.subsets`, a domain of subsets of an enum, is the one port that fails `cpl check` today. The spec has the `Subset<D>` value type but not the domain over all subsets. It's next on the list.
- **Reading letters:** the dream reads `?Letter` straight from the grid, with `.` as `None`. The port reads `Char` and does `Int(ch) - Int('A')` arithmetic, which is a C++ habit. With `@tag("A") A` variants checking now, reading `[Row, Col]?Letter` should be the spec form, but how `None` maps to `.` needs a decision.
- **Grouping by a domain key (decision 158):** I first rejected `R.group` as a one-problem helper, and the port paid 5 lines for a push loop because `group(by:)` returns a `HashMap`. The general rule is that library results stay keyed by the input's domain. Overloading `group` on "is `K` a domain" is ruled out (no overloading by type), so the domain-keyed form is `a.buckets()`: each value maps to the keys holding it. `last.buckets()` is `[?Row]List<(Col, Letter)>`, and `preimages` folds into it.
- **Struct construction:** the spec has field defaults but no constructor hooks (no side effects in constructors), so the dream's `init` is out. The port now builds from defaults and frees every column through the normal method, with no restore-afterwards trick.
- **Length:** 40 lines after the header, down from 49. The rest of the gap is the `Char` read and the spec's explicit struct fields.
- **Dream bug:** `count` was a list indexed by a subset. Arrays over `Letter.subsets` are the right shape, and the type checker catches the list version.

## cco19p5: CCO 2019 "Marshmallow Molecules"

**Problem.** For every i and every pair of its forward neighbours j < k, the closure adds the edge j → k. Count the edges of the final graph.

**Sketch.** Process i in order. i's forward set, minus i, passes to its smallest member; merge small to large.

```rust
# CCO 2019 P5 Marshmallow Molecules: for every i and pair of its forward neighbours j < k, the
# closure adds j -> k. Count the edges of the final graph.
# Processing i in order, i's forward set (minus itself) passes to its smallest member; merge
# small-to-large.
domain V = [read Int]
var out = [V] _ => SortedSet<V>()
for (u, v) in read [read Int] (V, V): out[u].insert(v)
var edges = 0
for i in V:
    out[i].remove(i)
    edges += out[i].len
    if let some(f) = out[i].first: out[f].absorb(mut out[i])
print(edges)
```

**Spec port** (`spec.cp`, passes `cpl check`)

```rust
// cco19p5: walking vertices in order, every out-neighbour set of v merges into that of its
// smallest neighbour f (the closure the problem asks for); the answer counts each set's size
// at the moment it is merged. Small-to-large merging keeps it O(m log^2 m).
let (n, m) = read<(1..=200_000, 0..=200_000)>()
domain V = 1..=n
var out: [V]SortedSet<V> = [V] v => SortedSet()
for (u, v) in read<[m](V, V)>():
    out[u].add(v)
var total = 0
for v in V:
    out[v].remove(v)
    total += out[v].len
    if let f = out[v].first:
        var moved = out[v].take()
        out[f].absorb(mut moved)
print(total)
```

**vs spec**

| Dream | Port | Verdict |
|---|---|---|
| `out[f].absorb(mut out[i])` | `var moved = out[v].take()` then `out[f].absorb(mut moved)` | Ownership rule holds |
| `if let some(f) = out[i].first` | `if let f = out[v].first` | Spec form |

- **Ownership:** two `mut` lends into the same array (`out[f]` and `out[i]`) are rejected, even though f ≠ i here. The checker can't prove disjoint indices in general, and we decided against a disjointness analysis to keep checking roughly linear. `take()` moves the set out, which is O(1) and states the intent: `out[i]` is empty afterwards anyway.

## egoi23p4: EGOI 2023 "Bikes vs Cars"

**Problem.** Rebuild a weighted graph from two matrices: for each pair, the best bottleneck over paths (max of path minimum) and the worst (min of path maximum). Print its edges, or NO.

**Sketch.** Both matrices must already be closed under their semiring. A pair can carry an edge only where the two agree it may. If any graph works, the max and min spanning trees of those candidates are the whole answer.

```rust
// EGOI 2023 P4: rebuild a weighted graph from two matrices: for each pair, the best bottleneck
// over paths (max of path minimum) and k minus the worst (min of path maximum). Print its edges
// or NO. Both matrices must already be closed under their semiring; a pair can carry an edge
// only where the two agree it may; the max and min spanning trees of those candidates are the
// whole answer if any graph is.
domain V = [read Int]
let k = read Int
let widest = read lower_triangle [V, V]Int
let narrowest = (read lower_triangle [V, V]Int).map(b => k - b)

let fits = (g: Mat<Int>) => g == g.closure(Semiring(max, min)) 
if not fits(widest) or not narrowest.closed(Semiring(min, max)):
    print("NO")
    return

let candidates = [for (u, v) in V.pairs() if widest[u, v] >= narrowest[u, v]
                  for w in [widest[u, v], narrowest[u, v]]: (u:, v:, w:)]
let edges = kruskal(candidates, by: desc(.w)) ++ kruskal(candidates, by: .w)
let g = Graph.undirected(edges)
if g.bottleneck(Semiring(max, min)) != widest or g.bottleneck(Semiring(min, max)) != narrowest:
    print("NO")
    return
print(edges.len)
for e in edges: print(e.u, e.v, e.w)
```

**Spec port** (`spec.cp`, passes `cpl check`)

```rust
// egoi23p4: rebuild a weighted graph from its all-pairs widest-path widths (bikes) and its
// all-pairs narrowest-bottleneck widths (cars, given as k minus the width). Both matrices must
// already be closed under their path algebra; an edge u-v may carry only weights between the two
// bottlenecks, so try both ends, check the closures, and print the union of a minimum and a
// maximum spanning forest of the candidates.
let (n, k) = read<(1..=500, 1..=1_000_000_000)>()
domain V = [n]
fn read_triangle(flip: Bool) -> [V, V]?Int:
    var m: [V, V]?Int = None
    for i in V:
        for j in ..i:
            let x = read<0..=1_000_000_000>()
            let w = if flip: k - x else: x
            (m[i, j], m[j, i]) = (Some(w), Some(w))
    m
let narrow = read_triangle(true)
let wide = read_triangle(false)
let widest = Semiring(max, min)
let narrowest = Semiring(min, max)
fn fits(u: V, v: V) -> Bool = (wide[u, v] ?? -1) >= (narrow[u, v] ?? inf)
fn only_fits(d: [V, V]?Int) -> [V, V]?Int = [V, V] (u, v) => if u != v and fits(u, v): d[u, v] else: None
fn closes_to(d: [V, V]?Int, over: Semiring, want: [V, V]?Int) -> Bool:
    let c = d.closure(over: over)!
    (V, V).all((u, v) => u == v or c[u, v] == want[u, v])
if not (closes_to(wide, widest, wide) and closes_to(narrow, narrowest, narrow)
        and closes_to(only_fits(wide), widest, wide) and closes_to(only_fits(narrow), narrowest, narrow)):
    print("NO")
else:
    let edges = [(u: u, v: v, w: w!) for (u, v) in V.pairs() if fits(u, v) for w in [wide[u, v], narrow[u, v]]]
    let low = kruskal(edges).take
    let high = kruskal(edges, by: e => desc(e.w)).take
    let keep = edges.keys.filter(e => low[e] or high[e]).collect()
    print(keep.len)
    for e in keep:
        print(edges[e].u, edges[e].v, edges[e].w)
```

**vs spec**

| Dream | Port | Verdict |
|---|---|---|
| `read lower_triangle [V, V]Int` (twice) | hand-written `read_triangle(flip)` (9 lines) | Rejected, but costly here |
| `g == g.closure(Semiring(max, min))` | `d.closure(over: widest)!` in a 2-line `closes_to` | **Spec change (decision 160)**; first rejected, then adopted |
| `g.bottleneck(Semiring(...))` | the same `closes_to` on the candidate matrix | Rejected `bottleneck` |
| `kruskal(candidates, by: desc(.w)) ++ kruskal(...)` | `kruskal(edges, by: e => desc(e.w)).take` | Idiom fix in the port |
| `[for (u, v) in V.pairs() if ... for w in [...]: (u:, v:, w:)]` | the same comprehension | Port fixed |

- **Closure as a value (decision 160):** I first rejected `closure` as a duplicate of `floyd_warshall`. That was wrong: it is the value counterpart of an in-place algorithm, the same pairing as `sort`/`sorted`. `d.closure(over:)` returns `?[V, V]?W`, and the check reads `(V, V).all((u, v) => u == v or c[u, v] == want[u, v])` (decision 159: inline product domains and tuple-unpacking lambdas).
- **Candidates:** with `pairs()` meaning every pair, the dream's comprehension is legal spec, and the port uses it.
- **Still open:** the triangle read. I'd use a general ragged read (`read<[i in V][..i]Int>()`) over a `lower_triangle` mode. That changes the read grammar, so it needs a decision first.
- **Length:** 30 lines after the header, down from 38. The 9-line `read_triangle` is most of what's left.
- **`desc(.w)`:** the port first negated weights, and the dream showed `by: desc(.w)` is the spec idiom.

## noi02p3: NOI 2002 "Greedy Head"

**Problem.** m heads eat all n fruits of a tree. The big head eats exactly k, including the root, and every head eats at least one. An edge costs its weight when the big head eats both ends, and also when the small head does but only if m = 2. Minimise the cost, or print -1.

**Sketch.** Tree knapsack: for each vertex and each eater of it, the cost by how many big-eaten vertices lie in its subtree. Children merge by min-plus convolution.

```rust
// NOI 2002 Greedy Head: m heads eat all n fruits of a tree; the big head eats exactly k
// including the root, every head eats at least one. Pay an edge's weight when the big head eats
// both ends, and (only when m = 2) when the small head does. Minimise the cost, or -1.
// Tree knapsack: per vertex and whether the big head ate it, the cost by how many big-eaten
// vertices lie in its subtree; children merge by min-plus convolution.
let (n, m, k) = read (Int, Int, Int)
domain V = 1..=n
let t = Rooted(Graph.undirected(read [n - 1](u: V, v: V, w: Int)), root: V.first!)
if m - 1 + k > n: print(-1); return
enum Eater: Big, Small
fn pays(a: Eater, b: Eater) -> Bool = a == b and (a == Big or m == 2)

let cost = t.up((v, kids) =>
    [for me in Eater:
        kids.fold(if me == Big: [inf, 0] else: [0], (acc, (arc, sub)) =>
            convolve(acc, [for j in sub[Big].keys:
                min(for them in Eater: sub[them][j] + (if pays(me, them): arc.w else: 0))],
                over: Semiring(min, +)))])
print(cost[t.root][Big][k])
```

**Spec port** (`spec.cp`, passes `cpl check`)

```rust
// noi02p3 Nine-Headed Dragon: the big head eats exactly k fruits including the root, every other
// head at least one; a branch costs when one head eats both ends. With three or more heads the
// small heads alternate freely, so only "big or small" matters per fruit: a tree knapsack over
// how many fruits the big head eats, merged child by child as a min-plus convolution.
let (n, heads, k) = read<(1..=300, 2..=300, 1..=300)>()
domain F = 1..=n
let branches = read<[n - 1](a: F, b: F, cost: 0..=100_000)>()
let t = Rooted(Graph.undirected(branches), root: F.first!)
let tropical = Semiring(min, +)
enum Head: Small, Big
fn clash(w: Head, wc: Head, c: F) -> Int:
    if w == wc and (w == Head.Big or heads == 2): t.up_arc(c)!.cost else: 0
// eat(v)[w][x]: the cheapest for v's subtree when head w eats v and the big head eats x of its
// fruits.
fn eat(v: F) -> [Head]List<?Int>:
    var best: [Head]List<?Int> = [Head] w => if w == Head.Big: [None, Some(0)] else: [Some(0), None]
    for c in t.children(v):
        let sub = eat(c)
        let via = (w, wc) => convolve(best[w], sub[wc].map(x => x + clash(w, wc, c)).collect(), over: tropical, shape: Curved.Neither)
        best = [Head] w => zip(via(w, Head.Small), via(w, Head.Big)).map(((p, q)) => min(p, q)).collect()
    best
print(if heads - 1 + k > n: -1 else: eat(t.root)[Head.Big][k]!)
```

**vs spec**

| Dream | Port | Verdict |
|---|---|---|
| `t.up((v, kids) => ...)` fold | recursive `fn eat(v)` over `t.children(v)` | Both fine |
| `arc.w` from the fold's `(arc, sub)` | `t.up_arc(c)!.cost` | Spec form (`Rooted.up_arc`) |
| `enum Eater: Big, Small`, `pays(a, b)` | `enum Head: Small, Big`, `clash(w, wc, c)` | Same idea |
| `convolve(..., over: Semiring(min, +))` | same, plus `shape: Curved.Neither` | Spec form |

- **Matches the spec:** the spec already had every piece. Tree knapsack is `Rooted` plus `convolve(over: tropical)`, and the edge cost comes from the arc (`up_arc(c)!.cost`), not a side array.
- **Fold or recursion:** `t.up` is shorter, and the named recursion is easier to debug and to add early exits to. Both are idiomatic, so neither changed.
- **Small friction:** `shape: Curved.Neither` is noise when the curve shape is unknown. It should be the default (open).

## ioi19p6: IOI 2019 "Sky Walking"

**Problem.** Buildings stand at positions x with heights h, and skywalks at height y join buildings l and r. Find the shortest walk from the base of building s to the base of building g.

**Sketch.** Cut every skywalk that passes over s or g at the nearest building on each side tall enough to hold it. Then the only useful stops are the walk ends and where each end's building meets the walk just below it. Run Dijkstra on that graph.

```rust
// IOI 2019 "Sky Walking": buildings at x with heights h, skywalks at height y
// between buildings l and r; shortest walk from the base of s to the base of g.
pub fn min_distance(x: List<Int>, h: List<Int>, l: List<Int>, r: List<Int>, y: List<Int>, s: Int, g: Int) -> Int:
    domain B = [x.len]
    let tall = SparseTable(max, [B] b => h[b])
    let walks = List.zip(l, r, y).map((a, b, z) => (span: B(a)..=B(b), y: z))

    // a walk over s or g is cut at the nearest building on each side tall enough to hold it
    fn cut(w, at: B) -> List<(span: RangeInclusive<B>, y: Int)>:
        if not (w.span.l < at < w.span.r):
            return [w]
        let left = (w.span.l..=at).last_true(b => tall.fold(b..=at) >= w.y)!
        let right = (at..=w.span.r).first_true(b => tall.fold(at..=b) >= w.y)!
        [(span: w.span.l..=left, y: w.y), (span: left..=right, y: w.y), (span: right..=w.span.r, y: w.y)]
            .filter(p => p.span.l < p.span.r).collect()
    let walks = walks.flat_map(w => cut(w, B(s))).flat_map(w => cut(w, B(g))).collect()

    // stops: walk ends, plus where each end's building meets the walk just below
    let below = walks.sorted(by: .y).sweep_below(at: (w, b) => w.span.contains(b) and h[b] >= w.y)
    var stops = Set<(B, Int)>([(B(s), 0), (B(g), 0)])
    for w in walks:
        for b in [w.span.l, w.span.r]:
            stops.add((b, w.y))
            for under in below(b, w.y):
                stops.add((b, under.y))

    domain Stop = stops.keys
    let along = walks.flat_map(w => stops.range((w.span.l, w.y)..=(w.span.r, w.y)).filter(p => p.1 == w.y).pairs())
    let up = stops.group(by: .0).values.flat_map(ps => ps.sorted(by: .1).pairs())
    let g2 = Graph.undirected(chain(along, up).map((p, q) => (u: Stop(p), v: Stop(q), w: abs(x[p.0] - x[q.0]) + abs(p.1 - q.1))))
    dijkstra(g2, from: [(Stop((B(s), 0)), 0)]).dist[Stop((B(g), 0))] ?? -1
```

**Spec port** (`spec.cp`, passes `cpl check`)

```rust
// IOI 2019 "Sky Walking": buildings at x with heights h, skywalks at height y
// between buildings l and r; shortest walk from the base of s to the base of g.
// Walks over s or g are cut at the nearest building on each side tall enough
// to hold them; then the only useful stops are walk ends and, at each end's
// building, the highest walk passing just below it. Dijkstra over those stops.
@grader
fn min_distance(x: List<Int>, h: List<Int>, l: List<Int>, r: List<Int>, y: List<Int>, s: Int, g: Int) -> Int:
    domain B = [x.len]
    let tall = SparseTable(max, [B] b => h[Int(b)])
    let (start, goal) = (B(s), B(g))

    fn cut(w: (span: Range<B>, y: Int), at: B) -> List<(span: Range<B>, y: Int)>:
        if not (w.span.first! < at and at < w.span.last!):
            return [w]
        let left = (w.span.first!..=at).last_true(b => tall.fold(b..=at) >= w.y)!
        let right = (at..=w.span.last!).first_true(b => tall.fold(at..=b) >= w.y)!
        [(span: w.span.first!..=left, y: w.y), (span: left..=right, y: w.y), (span: right..=w.span.last!, y: w.y)]
            .filter(p => p.span.len > 1).collect()

    let raw = [(span: B(l[i])..=B(r[i]), y: y[i]) for i in 0..l.len]
    let walks = raw.flat_map(w => cut(w, start)).flat_map(w => cut(w, goal)).collect()

    var stops: List<(b: B, y: Int)> = [(b: start, y: 0), (b: goal, y: 0)]
    var on: List<List<B>> = [List() for _ in walks]
    let starting = walks.keys.collect().group(by: i => walks[i].span.first!)
    let ending = walks.keys.collect().group(by: i => walks[i].span.last!)
    var active = SortedBag<(y: Int, walk: Int)>()
    for b in B:
        for i in starting.get(b) ?? List():
            active.add((y: walks[i].y, walk: i))
        for i in (starting.get(b) ?? List()) + (ending.get(b) ?? List()):
            stops.push((b: b, y: walks[i].y))
            on[i].push(b)
            if let Some(under) = active.last_true(a => a.y < walks[i].y):
                stops.push((b: b, y: under.y))
                on[under.walk].push(b)
        for i in ending.get(b) ?? List():
            active.remove((y: walks[i].y, walk: i))

    let (domain Stop, at) = compress(stops)
    var links: List<(u: Stop, v: Stop, w: Int)> = []
    for i in walks.keys:
        for (p, q) in on[i].sorted().dedup().adjacent:
            links.push((u: at.at((b: p, y: walks[i].y)), v: at.at((b: q, y: walks[i].y)), w: x[Int(q)] - x[Int(p)]))
    for (p, q) in Stop.with_next:
        if let Some(q) = q:
            if at.get(p).b == at.get(q).b:
                links.push((u: p, v: q, w: at.get(q).y - at.get(p).y))

    let net = Graph<Stop>.undirected([links.keys] e => links[e])
    dijkstra(net, from: [(at.at((b: start, y: 0)), 0)]).get(at.at((b: goal, y: 0))) ?? -1
```

**vs spec**

| Dream | Port | Verdict |
|---|---|---|
| `walks.sorted(by: .y).sweep_below(at: ...)` | hand sweep with `SortedBag` and start/end buckets (about 15 lines) | Rejected `sweep_below` |
| `Set<(B, Int)>` + `domain Stop = stops.keys` | `compress(stops)` → `domain Stop` | Spec form |
| `stops.range(...).pairs()` along a walk | `on[i].sorted().dedup().adjacent` | Dream bug (`pairs` vs `adjacent`) |
| `span: RangeInclusive<B>`, `w.span.l` | `span: Range<B>`, `w.span.first!` | Spec form |

- **`sweep_below` rejected:** "for each point, the highest interval strictly below it" is one problem's query. The general pieces exist: a sorted bag swept over `B` with start/end buckets. That is why the port is about 15 lines longer than the dream.
- **A possible general piece:** a sweep that processes interval starts, then point queries, then ends at each coordinate comes up in several geometry and skyline problems. A `sweep(events, at:)` helper is a candidate, not adopted.
- **Dream bug (decision 154):** the dream's `.pairs()` meant neighbours along a walk. Under the new meaning it would link every pair of stops, which is O(k²) and wrong, so the spec form is `.adjacent`.
- **`compress` as the domain builder:** `let (domain Stop, at) = compress(stops)` replaced the dream's `stops.keys`, giving `at.at(...)` lookups. This is the same idiom as cmt21r2ztr.

