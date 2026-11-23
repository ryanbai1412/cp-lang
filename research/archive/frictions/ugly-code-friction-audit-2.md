# Zero-Tolerance Friction Audit 2: DP, Graph, Tree, and Data-Structure Ports (CP Lang 0.2.2)

This audit inspects DP, graph, tree, and data-structure ports across `proto/ports/` to a zero-tolerance ergonomic bar. It complements `research/archive/frictions/ugly-code-friction-audit-1.md` (which focused on I/O, formatting, casts, and general ad-hoc boilerplate across 101 files) by drilling into the algorithmic core of CP Lang `0.2.2`: domains in dynamic programming, shortest paths, rooted trees, heavy-light decomposition, segment trees, lazy propagation, Fenwick trees, DSU, heaps, arenas, bitmasks, and lexical borrowing.

## Scope and Methodology

We read **193 real `.cp` ports in full** across top-level `proto/ports/` (43 files), `hillclimb_dp_graph/` (15 files), `scout_batch_24` (10 advanced DS files), `scout_batch_26` (10 tree files), `scout_batch_27` (10 graph files), `scout_batch_28` (10 hard DP files), `scout_batch_30` (10 multi-topic files), `scout_batch_34`, `36`, `38`, `42`, `45`, and `48` (30 mid-series Olympiad/DMOJ files), `scout_batch_49`, `50`, `51`, and `52` (45 recent IOI/NOI/EGOI/APIO/CTSC ports), and `blind_critic_5` (10 draft-versus-runnable pairs). Every recurring pattern was cross-checked against a corpus-wide scan of all 988 active `.cp` files in `proto/ports/` (excluding `/before/`, `/bugs/`, and `/harness/`, with 630 in the recent/canonical subset) and verified line-by-line against `stdlib/*.cp` and `docs/one-page-cheat-sheet.md` on `main`.

Nothing was waved away as "good enough" or "minor." Wherever a contestant would write cleaner, faster-to-type code in C++17 with a personal CP template — or where CP Lang `0.2.2` forces a type-system escape, a double `!`, a 15-line hand-rolled loop, or a compiler workaround — it is ranked below.

Each of the 18 recurring patterns quotes exact verbatim snippets with file paths and line numbers, gives the ideal 1–2 line CP Lang `0.2.2` replacement, and classifies the fix under Ryan's userland-first rule and small concept budget: **Userland `stdlib/`**, **Language core**, or **Compiler/prelude parity** (features already promised by `0.2.2` that `crates/` does not yet lower cleanly).

---

## 1. Cross-Domain, Sub-Domain, and Compressed-Segment `.idx` / `.to_idx` / `.ord` / `.label` Escapes

**Classification:** Language core (narrow sub-domain widening and boundary arithmetic) + Userland `stdlib/seq.cp` (`Compressed.segments`)

**Corpus incidence:** `.idx`, `.to_idx`, `.ord`, or `.label` appears in **299 of 988** active ports (**30.3%**, and **172 of 630** recent ports, **27.3%**); `D.from_idx(...)` or `D.nth(...)` appears in **55 of 988** (**5.6%**).

Typed domains are CP Lang `0.2.2`'s headline safety guarantee, yet algorithmic code breaks out of them constantly at four exact seams: indexing a parent domain with a sub-domain index, converting a grid cell `(Row, Col)` into a node ID, mapping compressed coordinates to elementary segments, and computing distances or offsets on `Bounds<D>`.

Sub-domain to parent-domain indexing forces `.label` wrapping on every tree with root `0` or `1`:

```cp
// proto/ports/btoi17p6.cp:25-27
for k in Kid.rev:                       // children are finished before parents
    let below = pick[V(k.label)]
    pick[parent[k]] = join(pick[parent[k]], (below.count, below.near + 1))
```

```cp
// proto/ports/hillclimb_dp_graph/noi15p2.cp:18-20
let parent: [Dependent]Pkg = read(gen: random_parents)
var needed_by: [Pkg]List<Pkg> = []
for d in Dependent: needed_by[parent[d]].push(Pkg(d.label))
```

Coordinate-compressed sweep-line segment trees suffer a brutal three-hop `.to_idx` + `from_idx` + `!` cascade just to get elementary intervals:

```cp
// proto/ports/blind_critic_5/bsspc21s4_draft.cp:127-133
for r in rects:
    let yl = YSeg.from_idx(cy.at(r.y1).to_idx)!
    let yr = YSeg.from_idx(cy.at(r.y2).to_idx - 1)!
    events[cx.at(r.x1)].push((yl, yr, 1))
    events[cx.at(r.x2)].push((yl, yr, -1))

let leaves: [YSeg]Node = [YSeg] i => Node.single(cy[cy.D.from_idx(i.to_idx)!], cy[cy.D.from_idx(i.to_idx + 1)!], false)
```

Grid-to-graph dual graphs and `Bounds<D>` offset math drop straight back into raw integer division, modulo, and `.ord`:

```cp
// proto/ports/scout_batch_27/noi10p3.cp:34-39
let (i, j) = (B(u.to_idx / n), B(u.to_idx % n))
let (top, bottom, left, right) = (X(i.to_idx), X(i.to_idx + 1), X(j.to_idx), X(j.to_idx + 1))
var to: List<(to: Node, cost: Int)> = []
to.push((to: if i.to_idx == 0: out else: block(i.to_idx - 1, j.to_idx), cost: east[top, j]))
to.push((to: if j.to_idx == n - 1: out else: block(i.to_idx, j.to_idx + 1), cost: south[i, right]))
```

```cp
// proto/ports/scout_batch_24/ioi07p3.cp:61-62
let tie_hi = H.bounds.nth(left_end.ord + rem)
st.apply(left_end..tie_hi, 1)
```

```cp
// proto/ports/hillclimb_dp_graph/noi15p2.cp:32-34
for (l, r) in tree.chains(Pkg.first, pkg):
    fresh += r.idx - l.idx + 1 - installed.sum(l, r)
    installed.assign(l, r, 1)
```

Every one of those expressions is strictly worse to type and harder to read than C++'s `r - l + 1`, `pick[k]`, or `ys[i + 1] - ys[i]`.

**Ideal CP Lang `0.2.2` replacement:**

```cp
// 1. Sub-domain declared as a slice of its parent widens automatically:
domain Dependent = Pkg.tail              // widens to Pkg; no Pkg(d.label)
let tree = Rooted.from_parents(parent)

// 2. Compressed exposes elementary segments directly (stdlib):
for r in rects: events[cx.at(r.x1)].push((cy.span(r.y1, r.y2), 1))
let leaves = cy.segments((lo, hi) => Node.single(lo, hi, false))

// 3. Domain distance and inclusive range length stay typed:
fresh += (l..=r).len - installed.sum(l..=r)
let tie_hi = (left_end + rem)!           // Bounds<H> supports + Int -> ?Bounds<H> and b - a -> Int
```

**Why this split respects the concept budget:** `Compressed.segments` / `Compressed.span` and `Range<D>.len` are pure userland `stdlib/`. In the core, only two small unified rules are needed: (a) allow `b - a -> Int` (signed distance between two indices of the same domain or `Bounds<D>`) and `b + k -> ?Bounds<D>` uniformly on both `D` and `Bounds<D>`, and (b) allow a sub-domain declared directly from a parent (`domain Kid = V.tail` or `domain Sub = V(1..)`) to coerce to `V` with zero runtime check. That eliminates the majority of `.idx`/`.ord`/`.label` escapes without weakening domain disjointness between unrelated domains (`Row` vs `Col`).

---

## 2. Forced `!` Unwraps on Provably Valid Steps, Non-Empty Ranges, and Aggregates

**Classification:** Language core (simple local narrowing for `l < r`, `for m in l..r`, and non-empty slices) + Userland `stdlib/seq.cp` (non-empty `Window<D>` bounds)

**Corpus incidence:** Postfix `!` appears in **366 of 988** active ports (**37.0%**, and **228 of 630** recent ports, **36.2%**).

A contestant expects `!` to mean "warning: I am asserting something unproved that could crash." Instead, CP Lang `0.2.2` forces `!` on the safest loops in competitive programming, creating alert fatigue and double-`!` eyesores like `(st_a.last! + 1)!`.

Interval DP hits this on every single recurrence:

```cp
// proto/ports/dpl.cp:10-12
@memo
fn lead(l: I, r: I) -> Int:
    if l == r: return a[l]
    max(a[l] - lead(l.next!, r), a[r] - lead(l, r.prev!))
```

```cp
// proto/ports/dpn.cp:10-13
@memo
fn cost(l: I, r: I) -> Int:
    if l == r: return 0
    min(cost(l, m) + cost(m.next!, r) for m in l..r)! + pre.sum(l..=r)
```

Monotonic stacks and 2D sliding windows hit double-`!` even though the guard or window size already proved non-emptiness:

```cp
// proto/ports/scout_batch_24/noip22p4.cp:84-86
while not st_a.is_empty and a[st_a.last!] < a[r]:
    st_a.pop()
let la = if st_a.is_empty: I.first else: (st_a.last! + 1)!
```

```cp
// proto/ports/scout_batch_38/apio09p1.cp:14-18
for rw in R.windows(k).rev:
    let b = rw.first!.l                       // FRICTION 38.2: non-empty window .first is ?R
    if let nxt = rw.first! + 1: down[b] = down[nxt.l]
    for cw in C.windows(k):
        down[b] max= pre.sum(rw, cw)
```

In `dpn.cp`, `m` ranges over the half-open `l..r`, which literally means `m < r` — so `m.next` is mathematically guaranteed to exist in `I`. In `apio09p1.cp`, `R.windows(k)` with `k >= 1` yields non-empty windows, yet `rw.first` returns `?R`. In `noip22p4.cp`, `st_a.last` is strictly less than `r` because `st_a` only holds earlier indices, yet the user writes `(st_a.last! + 1)!`.

**Ideal CP Lang `0.2.2` replacement:**

```cp
// Interval DP (dpn.cp): m in l..r proves m.next is I, and non-empty split guarantees min:
min(cost(l, m) + cost(m.next, r) for m in l..r) + pre.sum(l..=r)

// Monotonic stack (noip22p4.cp):
let la = st_a.last?.next ?? I.first

// Fixed-width windows (apio09p1.cp): Window<D> has non-optional .lo/.hi/.l/.r:
for rw in R.windows(k).rev:
    down[rw.l] = rw.next?.l |> (b => down[b]) ?? 0
    for cw in C.windows(k): down[rw.l] max= pre.sum(rw, cw)
```

**Why this split respects the concept budget:** `Window<D>` carrying non-optional `.first`/`.last`/`.l`/`.r` (since `windows(k)` for `k >= 1` is non-empty by construction) and `prev_greater` / `prev_smaller` monotonic-stack helpers are pure userland `stdlib/`. In the language core, two small, zero-syntax local facts — (1) `m` from `l..r` has upper witness `r` so `m.next` has type `D`, and (2) `if not xs.is_empty:` narrows `xs.last` / `xs.first` (or `xs.last?.next` optional chaining) — erase hundreds of noisy `!`s without adding a single keyword or weakening soundness.

---

## 3. Companion-State Relaxation & Missing Expression-Form Tropical Optionals

**Classification:** Language core (`min=` / `max=` returning `Bool`, plus lifting `min`/`max`/`+`/`+=` over `?Int`/`?F64`)

**Corpus incidence:** Manual `if d[v] == none or cand < d[v]!:` companion relaxations appear in **30 of 988** ports (**23 of 630** recent ports, **3.7%**); manual `opt_min` / `beats` / `if let w = dp[v]: dp[v] = w + dx` wrappers recur across DP and lazy-tree ports.

CP Lang's `min=` and `max=` on `?Int` are great when you only want the scalar optimum, and completely abandon you the instant you also maintain a path count, parent pointer, or tie-breaker — which happens in almost half of real shortest-path and DP problems.

```cp
// proto/ports/scout_batch_27/noi07p1.cp:38-42
if dist[s, t] == none or a + b < dist[s, t]!:
    dist[s, t] = a + b
    ways[s, t] = ways[s, k] * ways[k, t]
elif a + b == dist[s, t]!:
    ways[s, t] += ways[s, k] * ways[k, t]
```

```cp
// proto/ports/blind_critic_5/noi04p3_draft.cp (and noi04p3.cp):
if dp[s] == none or cand > dp[s]!:
    dp[s] = cand
    best_dir[s] = dir
    best_prev[s] = prev
```

Even worse, `min` and `max` do not work on `?Int` in expression position! Ports that pass a tropical monoid into `LazySegTree` or combine two optional DP values have to write throwaway helper functions around `min=`:

```cp
// proto/ports/scout_batch_30/cco21p5.cp:23-28
fn opt_min(a: ?Int, b: ?Int) -> ?Int:
    var out = a
    out min= b
    out

let add_min = Action<?Int, Int>(0, (f, g) => f + g, (f, x, _) => x + f, over: Monoid<?Int>(none, opt_min))
```

```cp
// proto/ports/scout_batch_36/cco19p4.cp:34-37
fn beats(a: ?F64, b: ?F64) -> Bool:
    if let x = a:
        if let y = b: x > y else: true
    else: false
```

And adding a scalar shift to every reachable DP state cannot use `+=` on `?Int`, forcing an `if let` unwrap-and-writeback:

```cp
// proto/ports/scout_batch_28/ioi04p2.cp (shift all reachable states by dx):
for v in Coord:
    if let w = on_x[v]: on_x[v] = w + dx
```

**Ideal CP Lang `0.2.2` replacement:**

```cp
// 1. chmin / chmax (or min= / max= returning Bool) for companion state:
if dist[s, t].chmin(a + b): ways[s, t] = ways[s, k] * ways[k, t]
elif dist[s, t] == a + b: ways[s, t] += ways[s, k] * ways[k, t]

// 2. Expression-form min/max and tropical += on ?T:
let add_min = Action<?Int, Int>(0, +, (f, x) => x + f, over: Monoid(none, min))
for v in Coord: on_x[v] += dx       // none + dx == none
```

**Why this belongs in the core:** `0.2.2` already gives `min=` and `max=` tropical semantics on `?T`, and `cco21p5.cp` even expects `x + f` to lift over `?Int`. Stopping halfway — supporting `x min= y` as a statement while rejecting `min(x, y)` as an expression, `x += dx` as a statement, and `if x.chmin(y):` as a boolean condition — is an inconsistency trap. Completing the tropical optional rule makes `?Int` a true zero-sentinel replacement for `INF` everywhere (DP, Dijkstra, Floyd-Warshall, Li Chao, lazy segment trees) with zero new concepts.

---

## 4. Manual Adjacency Lists & `Graph` Construction Friction

**Classification:** Userland `stdlib/graph.cp` and `stdlib/tree.cp`

**Corpus incidence:** Manual `var adj: [V]List<...>` appears in **162 of 988** active ports (**16.4%**) and **107 of 630** recent ports (**17.0%**) — more than twice as often as `Graph` itself (71 files in the DP/graph/tree/DS census).

Every contestant knows why they bypass `Graph<V, E>`:
1. `Graph.undirected(edges)` forces a throwaway edge domain `domain E = 0..m` even when edges have no IDs, and iterating `g[u]` yields `(to: V, e: E)` instead of bare `V` (so you write `for (v, _) in g[u]:` or `g.adj(u)`).
2. Weighted edges read as `(V, V, Int)` have no one-line `Graph.weighted_undirected(edges)` constructor that yields `(v, w)` directly when you iterate `g[u]`.
3. Reverse/transpose graphs require building a whole reversed edge array by hand.
4. Parent-array trees (`let parent: [Kid]V = read()`) cannot be passed straight to `Rooted` without manually constructing an adjacency list.

Look at how repetitive this is across recent ports:

```cp
// proto/ports/btoi17p3.cp:
var adj: [City]List<City> = []
for (u, v) in tracks:
    adj[u].push(v)
    adj[v].push(u)
let tree = Rooted(adj, City.first)
```

```cp
// proto/ports/scout_batch_34/dmopc20c6p3.cp:
let edges: [E](u: V, v: V) = read()
let g = Graph.directed(edges)
let rev_edges: [E](u: V, v: V) = [E] e => (u: edges[e].v, v: edges[e].u)
let rg = Graph.directed(rev_edges)
```

```cp
// proto/ports/scout_batch_52/ctsc04p4.cp (and scout_batch_26/ceoi22p3.cp, ioi09p7.cp):
var kids: [V]List<V> = [V] _ => []
for v in V if v != V.first: kids[parent[v]].push(v)
let t = Rooted(kids, V.first)
```

```cp
// proto/ports/scout_batch_51/egoi22p3.cp:
let g = Graph.undirected([(V.from_idx(u - 1)!, V.from_idx(v - 1)!) for (u, v) in edges])
```

**Ideal CP Lang `0.2.2` replacement:**

```cp
// Unweighted / weighted CSR graph without a throwaway edge domain:
let g = Graph.undirected(tracks)              // g[u] yields V; g.edges(u) yields (to, e)
let wg = Graph.undirected(weighted_edges)     // wg[u] yields (V, W) when edges carry weight W
let rg = g.reversed()                         // O(V + E) CSR transpose

// Tree directly from parent array or undirected edges:
let tree = Rooted.from_parents(parent)
let tree = Rooted.from_edges(tracks, root: City.first)
```

**Why this belongs in userland `stdlib/`:** `stdlib/graph.cp` and `stdlib/tree.cp` already hold `Graph` and `Rooted`. Adding `.reversed()`, unweighted/weighted overloads without requiring an explicit `E` domain, `Graph.builder<V>()` for dynamically constructed graphs, and `Rooted.from_parents` / `Rooted.from_edges` eliminates over 100 boilerplate adjacency loops with zero compiler changes.

---

## 5. Verbose Rolling-Buffer DP Swaps & Placeholder Array Initialization

**Classification:** Userland `stdlib/seq.cp` (`Rolling<D, T>` / `Buffer` helper) + minimal Language core (`[D]T` default-element / sparse-override init)

**Corpus incidence:** Per-step `var next: [...]` / `ndp` / `nxt` allocations appear in **50** DP/graph/tree/DS ports (30 explicit `var (next|ndp|nxt): [...]` declarations); dummy placeholder initializations like `[V] _ => Region.first` appear across dozens of graph/DS ports.

Two DP irritations recur constantly. First, rolling-buffer transitions either reallocate a fresh array inside every outer loop step (hurting cache locality and heap traffic at max-N) or manually manage two buffers plus a fill-reset loop and swap:

```cp
// proto/ports/dpt.cp:13-27
var ways: [Rank]Z = Z(0)
ways[Rank.first] = Z(1)
for s in Sign:
    let live = Rank.first..=Rank(s.label)   // ranks the longer prefix can end at
    var next: [Rank]Z = Z(0)
    ...
    ways = next
```

```cp
// proto/ports/scout_batch_34/dmopc21c2p4.cp:
var dp: [Carry]?Int = none
dp[Neutral] = 0
for i in I:
    var ndp: [Carry]?Int = none
    ...
    dp = ndp
```

```cp
// proto/ports/scout_batch_36/ccc19s5hard.cp:
var cur: List<Int> = [0; total]
var nxt: List<Int> = [0; total]
...
(cur, nxt) = (nxt, cur)
```

Second, initializing a DP table with one seed entry takes two statements (`var dp: [C]Z = Z(0); dp[C.first] = Z(1)`), and declaring an array of domain indices whose entries will all be filled immediately forces a fake placeholder (`Region.first`, `Card.first`) that masks uninitialized-slot bugs:

```cp
// proto/ports/dpm.cp:12-13
var ways: [C]Z = Z(0)
ways[C.first] = Z(1)
```

```cp
// proto/ports/scout_batch_26/ioi09p7.cp:
var home: [V]Region = [V] _ => Region.first
```

**Ideal CP Lang `0.2.2` replacement:**

```cp
// Zero-allocation rolling buffer (reuses two slabs, resets next in-place):
var dp = Rolling<Carry, ?Int>(none)
dp[Neutral] = 0
for i in I:
    dp.step(none) (cur, mut nxt) => ...       // swaps slabs, clears nxt in-place with zero alloc

// Seeded array literal / sparse override:
var ways: [C]Z = [C.first: Z(1), ..Z(0)]
```

**Why this split respects the concept budget:** `Rolling<D, T>` is a 15-line userland `stdlib/` struct wrapping two `[D]T` buffers with `swap` + `fill` (zero compiler cost, zero per-step allocation in emitted C++17!). Only the optional `[k: v, ..default]` array-override literal touches syntax, and even that can wait if `Array.seeded(default, k, v)` in `stdlib/` proves short enough.

---

## 6. Clamped Domain Windows, Skipping Endpoints, and Awkward Range Reversal

**Classification:** Userland `stdlib/seq.cp` & `Domain` / `Index` methods

**Corpus incidence:** `I.first` / `I.last` appears in **195 of 613** DP/graph/tree/DS ports (**31.8%**), often just to skip one end, clamp a sliding-window radius, or reverse a full domain.

Sliding-window DP and range-query problems routinely need "all indices within radius `k` before `p`," "skip the first element of `V`," or "walk `Pos` backwards." Without clamping and tail/init helpers on `D` and `Range<D>`, ports fall straight back to `.to_idx` + `max(0, ...)` + `D.nth(...)` or ugly `if v == V.first: continue`:

```cp
// proto/ports/scout_batch_24/cco18p5.cp:44-48
fn left_best(p: I) -> ?Peak:
    let pi = p.to_idx
    if pi == 0: return none
    let lo = max(0, pi - k + 1)
    val_st.fold(I.nth(lo)..p)
```

```cp
// proto/ports/dmpg16s5.cp:18-19
for p in Pos:
    best max= pre[p.r] - lowest.query(Cut(max(p.idx + 1 - k, 0)), p.l)
```

```cp
// proto/ports/dpm.cp:17
for c in C: ways[c] = pre[c.r] - pre[(c - cap ?? C.first).l]
```

```cp
// proto/ports/ccc10s4.cp:15
for pen in Pen.first.next!..:
```

```cp
// proto/ports/scout_batch_26/noip19p2.cp (and scout_batch_52/ioi14p1.cp):
for v in V:
    if v == V.first: continue
```

```cp
// proto/ports/blind_critic_5/noi22p5_draft.cp (and scout_batch_42/mccc4j3.cp):
for p in (Pos.first..=Pos.last).rev:
let total = pre.sum(V.first..=V.last)
```

Note also the subtle inconsistency: `V.rev` works on a domain, yet contestants still write `(Pos.first..=Pos.last).rev` and `pre.sum(V.first..=V.last)` instead of `Pos.rev` and `pre.sum(..)` because full-range `..` and `D.tail` / `D.init` are missing or under-documented.

**Ideal CP Lang `0.2.2` replacement:**

```cp
// Clamped domain shift (`sat_sub` / `sat_add`, or `p.back(k)` / `p.ahead(k)`):
val_st.fold(p.back(k - 1)..p)                 // returns empty range when p == I.first!
best max= pre[p.r] - lowest.fold(p.back(k - 1).l..=p.l)
for c in C: ways[c] = pre[c.r] - pre[c.back(cap).l]

// Skipping endpoints without .first.next! or if-continue:
for pen in Pen.tail:                          // all elements after Pen.first (empty if len <= 1)
for p in Pos.rev:                             // and pre.sum(..) for full domain
```

**Why this belongs in userland `stdlib/`:** `.back(k)` / `.ahead(k)` (clamping to `D.first` / `D.last`), `D.tail` (`1..len`), `D.init` (`0..len-1`), and `Range` empty-on-inverted behavior are pure `stdlib/` methods on `Index` and `Domain`. They delete an entire class of off-by-one bugs and `.to_idx` escapes in sliding-window DP and tree loops.

---

## 7. Rerooting Tree DP & Sibling Prefix/Suffix Sweeps

**Classification:** Userland `stdlib/tree.cp` (`Rooted.reroot` and `without_each`)

**Corpus incidence:** Every non-invertible rerooting DP (trees mod composite `m`, max-over-children rerooting, centroid/diameter rerooting) repeats the same 15–20 lines of `down` bottom-up fold + `earlier`/`later` sibling sweeps.

Look at `dpv.cp` — supposed to be a showcase tree DP in `hillclimb_dp_graph/`:

```cp
// proto/ports/hillclimb_dp_graph/dpv.cp:17-34
var down: [V]Int = 1
for v in tree.preorder.rev:
    if let p = tree.parent(v): down[p] = down[p] * (down[v] + 1) % m

var up: [V]Int = 1
for v in tree.preorder:
    var earlier = up[v]
    for c in tree.children(v):
        up[c] = earlier
        earlier = earlier * (down[c] + 1) % m
    var later = 1
    for c in tree.children(v).rev:
        up[c] = (up[c] * later + 1) % m
        later = later * (down[c] + 1) % m
```

Every contestant has written that exact forward-and-backward sibling sweep dozens of times and still introduces off-by-one bugs between `up[v]`, the identity element, and applying the edge-lift (`+ 1`) before vs after multiplying `later`. Even when you only need a non-rerooting product over children, `stdlib/seq.cp` lacks `product(...)` (seen in `scout_batch_49/ioi22p4.cp`: `for v in kids[u]: all[u] *= all[v]`).

**Ideal CP Lang `0.2.2` replacement:**

```cp
// Userland Rooted.reroot(monoid, lift_edge): handles both down and up in O(V) without division!
let ans = tree.reroot(
    id: 1,
    merge: (a, b) => a * b % m,
    lift: (v, x) => (x + 1) % m,
)
for v in V: print(ans[v])
```

And for custom rerooting where parent/child payloads differ, a standalone `exclude_self(xs, id, merge)` helper in `stdlib/seq.cp` yields the prefix-merged-with-suffix value for every element of any slice in 2 lines.

**Why this belongs in userland `stdlib/`:** `Rooted.reroot` is ~25 lines of plain userland CP Lang inside `stdlib/tree.cp` using `tree.preorder` and `tree.children(v)`. It turns a classic 20-line bug-prone pattern into 4 crystal-clear lines and adds zero compiler complexity.

---

## 8. HLD, Centroid, and Preorder-Domain Friction in `Rooted<V>`

**Classification:** Userland `stdlib/tree.cp`

**Corpus incidence:** `Rooted` appears in **64** DP/graph/tree/DS ports, and advanced tree ports repeatedly poke into `Rooted`'s private fields (`t.at`, `t.tin`, `t.heavy`), double-root to find a centroid, or re-wrap `tree.pos(u)` into a hand-declared `domain Pre = 0..n`.

Four concrete tree smells stood out across `scout_batch_26`, `scout_batch_27`, and `scout_batch_52`:

1. **Double-rooting to find a centroid:**
```cp
// proto/ports/scout_batch_26/noip19p6.cp:29-34
let first = Rooted(u => g.adj(u), V.first)
var c = V.first
while let h = first.heavy[c]:
    if 2 * first.size(h) <= n: break
    c = h
let t = Rooted(u => g.adj(u), c)
```

2. **Reaching into internal `Rooted` arrays (`t.at`, `t.heavy`) just to initialize a `t.Pre`-indexed segment tree:**
```cp
// proto/ports/scout_batch_26/sdoi17r1p2.cp:28-31
var f = LazySegTree<t.Pre, Int, Int>(AddMax, [t.Pre] p => t.depth(t.at[p]) + 1)
var same: [V]?V = [V] _ => none
// 1 where same[v] is not the heavy child
var off = SegTree<t.Pre, Int>(+, [t.Pre] p => if t.heavy[t.at[p]] == none: 0 else: 1)
```

3. **Wrapping `tree.pos(u).idx` into a manual `Pre` domain because `t.Pre` isn't consistently exposed or accepted by `Fenwick` / arrays:**
```cp
// proto/ports/scout_batch_52/iati21a13.cp:60-66
for u in V if init_on[u] == 1:
    active[Pre(tree.pos(u).idx)] = true
    bit.add(Pre(tree.pos(u).idx), 1)
...
let p = Pre(tree.pos(u).idx)
```

4. **Missing `subtree_edges` / `edge_pos(u)` and `Rooted.centroids()` / `Rooted.on_path(u, v)`:** Edge-weighted HLD constantly has to remember "skip `lca(u, v)` by taking `(l + 1)..=r` on the final chain," which triggers another `.next!` or `.idx` escape.

**Ideal CP Lang `0.2.2` replacement:**

```cp
// 1. Built-in centroid + re-rooting helper:
let t = Rooted.at_centroid(g)                  // or g.centroids()

// 2. Map vertex values directly into preorder domain (no t.at[p] leakage):
var f = LazySegTree(AddMax, t.in_preorder(u => t.depth(u) + 1))

// 3. Direct vertex-keyed subtree/path methods and edge-HLD flag:
var bit = Fenwick<t.Pre, Int>()
bit.add(t.pos(u), 1)                           // t.pos(u) IS already t.Pre; never Pre(...idx)
let ans = t.fold_edge_path(u, v, st)           // automatically excludes LCA without l.next!
```

**Why this belongs in userland `stdlib/`:** Every piece (`centroids`, `at_centroid`, `in_preorder`, `fold_edge_path`, `apply_edge_path`, and exposing `t.Pre` consistently across `Rooted`, `SegTree`, `LazySegTree`, and `Fenwick`) lives 100% inside `stdlib/tree.cp`.

---

## 9. Point-Modify & Stock-Monoid/Action Gaps in `SegTree` and `LazySegTree`

**Classification:** Userland `stdlib/segtree.cp`, `stdlib/lazy.cp`, and `stdlib/algebra.cp`

**Corpus incidence:** `SegTree` / `LazySegTree` appear in **81** DP/graph/tree/DS ports (**13.2%**), and almost every non-trivial one either works around a missing `.update(i, f)` method or hand-writes the same 4 stock monoids/actions.

First, **`SegTree` has `.set(i, v)` and `.get(i)` but NO `.update(i, f)`** (ironically, `LazySegTree` in `stdlib/lazy.cp` has `update(i, f)` while `SegTree` in `stdlib/segtree.cp` does not!). So point increments and point `max=` either duplicate the index expression or keep a whole shadow array `cur`:

```cp
// proto/ports/scout_batch_26/dmopc20c7p4.cp:27
if let p = t.parent(u): near.set(t.pos(p), near.get(t.pos(p)) + (if safe[u]: 1 else: -1))
```

```cp
// proto/ports/scout_batch_51/apio23p2.cp:50-51
cur[y] max= b.x
st.set(y, cur[y])
```

Second, **four universal CP monoids/actions are missing from `stdlib/algebra.cp`**, forcing 6–12 lines of boilerplate per file:
- `ArgMax<D, T>` / `ArgMin<D, T>` (`proto/ports/scout_batch_24/cco18p5.cp`: *"WISH A14: Every argmax segment tree repeats this 6-line optional-pair `Monoid`"*)
- `AddMinCount` (`proto/ports/scout_batch_49/ioi18p2.cp`: `let add = Action(0, +, (d, x) => (x.least + d, x.times))`)
- `AssignAdd` (`proto/ports/scout_batch_48/btoi06p5.cp:34-39`):

```cp
// proto/ports/scout_batch_48/btoi06p5.cp:34-39
type Tag = (set: ?Int, add: Int)
let assign_add = Action<(Int, Sym), Tag>(
    id: (none, 0),
    compose: (old, new) => if new.set != none: new else: (old.set, old.add + new.add),
    apply: (v, t) => ((t.set ?? v.0) + t.add, v.1))
var cost = LazySegTree<Sym, (Int, Sym)>(assign_add, (min, (FAR, Sym.last)), s => (0, s))
```

- `LazySegTree` constant-fill constructor (`proto/ports/blind_critic_5/noi22p5_draft.cp` writes `LazySegTree<Val, Int>(AddMin, [Val] _ => 0)` instead of `LazySegTree<Val>(AddMin, 0)`).

Third, **matrix / struct monoids copy by value on every merge** (`proto/ports/scout_batch_24/ioi13p3io.cp`, `scout_batch_52/ioi13p3.cp`): a `Monoid` returning `Mat` allocates/copies a `200 x 200` table inside `merge(a, b)` because there is no `merge_into(mut dst, a, b)` hook.

**Ideal CP Lang `0.2.2` replacement:**

```cp
// 1. SegTree point modification:
if let p = t.parent(u): near.add(t.pos(p), if safe[u]: 1 else: -1)
st.update(y, v => max(v, b.x))                // or st.chmax(y, b.x)

// 2. Stock algebra + fill constructor:
var val_st = SegTree<I>(ArgMax, a)
var cost = LazySegTree<Sym>(AssignAddMin, (0, s => s))
var st = LazySegTree<Val>(AddMin, 0)
```

**Why this belongs in userland `stdlib/`:** Every single fix is a tiny addition to `stdlib/segtree.cp`, `stdlib/lazy.cp`, and `stdlib/algebra.cp` — high leverage, zero syntax growth.

---

## 10. Hand-Rolled `o * 2` / `o * 2 + 1` Segment Trees for Non-Pushdown, Time-Rollback, Li Chao, and 2D Trees

**Classification:** Userland `stdlib/` (`CoverTree`, `LiChao`, `TimeSegTree`, and `SegTree.descend` / `SegTree.visit`)

**Corpus incidence:** Raw binary-heap tree indexing (`o * 2` / `o * 2 + 1` or `2 * p`) appears in **31 of 630** recent ports (**4.9%**), almost exclusively in four recurring data structures that `SegTree<D, T>` and `LazySegTree<D, T, F>` cannot express cleanly.

1. **Cover-count / Klee's measure trees** (no pushdown; recomputing from children when `cnt` drops to 0 breaks `Action<T, F>`):
```cp
// proto/ports/scout_batch_24/cco16p5.cp:20-25
// WISH A23: Cover-Count Segment Tree structurally breaks `LazySegTree<D, T, F>`!
// Updates are paired `+1`/`-1` over ranges with NO pushdown: a node's covered
// length is `if cnt[p] > 0: total_len[p] else if leaf: 0 else: cov[2p] + cov[2p+1]`.
// Because `apply(-1, x)` cannot know children's `cov` from `x` alone when `cnt`
// drops from 1 to 0, it is NOT an `Action<T, F>` and forces a hand-rolled
// recursive segment tree with raw `2*p` / `2*p + 1` indexing.
```

2. **Segment-tree-over-time (offline dynamic connectivity / rollback):**
```cp
// proto/ports/scout_batch_36/cco20p5.cp:30-32 (and scout_batch_24/noi20p2.cp)
let mid = (l + r) / 2
if ql <= mid: add_span(o * 2, l, mid, ql, qr, iv)
if mid < qr: add_span(o * 2 + 1, mid + 1, r, ql, qr, iv)
```

3. **Li Chao trees (convex-hull / line-envelope DP):**
```cp
// proto/ports/scout_batch_36/cco19p4.cp:39-60
struct LiChao:
    size: Int
    lines: List<?Line>
    // 35 lines of manual mid = (l + r) / 2, o * 2, o * 2 + 1
```

4. **Segment-tree-of-buckets / fractional cascading** (`proto/ports/scout_batch_30/noi19p4.cp`, `scout_batch_52/iati21a13.cp`): pushing an event onto the `O(log n)` canonical range-cover nodes of a domain.

**Ideal CP Lang `0.2.2` replacement:**

```cp
// 1. Cover-count tree over compressed elementary segments:
var cov = CoverTree(cy.lengths())
cov.add(cy.span(r.y1, r.y2), +1)
ans += cov.covered * width

// 2. Offline time-range DFS without writing a single o * 2:
var timeline = TimeSegTree<Q, Edge>()
timeline.add(l..=r, edge)
timeline.run(on_enter: e => dsu.unite(e), on_leave: _ => dsu.undo(), on_leaf: t => ...)

// 3. Li Chao tree for convex-hull DP:
var hull = LiChao<X, Int>.max()
hull.add(Line(m, b))
dp[i] = hull.query(x[i])
```

**Why this belongs in userland `stdlib/`:** None of these needs a language feature. Three compact, battle-tested `stdlib/` modules (`CoverTree`, `TimeSegTree`, `LiChao`) plus a `Domain.cover_nodes(range)` / `SegTree` range-decompose helper wipe out 80%+ of all manual `o * 2` segment trees in Olympiad ports.

---

## 11. Arena / `Pool<Node>` Treaps & Sparse Trees: Missing Free-List, `pool[u].` Noise, and `I32` Suffixes

**Classification:** Userland `stdlib/` (`Pool.free` / `Treap<T>` / `SparseSegTree`) + Language core (evaluate whether `with mut pool[u]:` or local `mut` bindings are worth it)

**Corpus incidence:** `Pool<Node>` / `arena[u]` / `push_id` appears in **10** heavy DS ports (`noi05p2`, `noi20p2`, `egoi22p4`, `noi21p5`, `noi20p5`, etc.), and every single one turns into noisy low-level pointer code.

Look at what happens in `scout_batch_24/noi05p2.cp`, `scout_batch_51/egoi22p4.cp`, and `scout_batch_52/noi21p5.cp`:
1. **`Pool<Node>` has no `.free(id)` / recycle stack**, so deleting 4,000,000 treap nodes (`noi05p2.cp`) forces every contestant to hand-roll a `free_ids: List<Node.Id>` stack and an `alloc()` wrapper — plus a hand-rolled xorshift PRNG for treap priorities!
2. **Lexical no-stored-borrows forces field-copying before recursion** (`let (pl, pr) = (pool[pu].l, pool[pu].r)`) and repeating `pool[u].` 30–50 times per function.
3. **32-bit memory-tight nodes spray `I32(...)` and `0i32` casts** across every expression (`noi05p2.cp`, `noip22p4.cp`'s `0u64`).

```cp
// proto/ports/scout_batch_52/noi21p5.cp (typical arena split boilerplate):
fn split(t: ?Id, k: Int) -> (?Id, ?Id):
    let u = t ?? return (none, none)
    push(u)
    let left_sz = sz(pool[u].l)
    if left_sz >= k:
        let (a, b) = split(pool[u].l, k)
        pool[u].l = b
        pull(u)
        (a, u)
```

**Ideal CP Lang `0.2.2` replacement:**

```cp
// 1. Pool<T> ships with O(1) .alloc(val) / .free(id) recycling and stdlib Treap<T, F>:
var tr = Treap<Int, RevAssign>(initial)       // covers 90% of sequence-treap problems in 1 line!
let (a, mid, c) = tr.split3(l..=r)
tr.apply(mid, Reverse)

// 2. For custom pool trees (persistent / mergeable segtree, link-cut tree), Pool.free(id):
let u = pool.alloc(Node(val: x, prio: rand_u64()))
pool.free(u)
```

**Why this belongs in userland `stdlib/`:** Keep the language core's no-stored-borrows guarantee untouched — it is a core pillar of zero-UB trust. Instead, ship `Pool.free(id)` + a fast `rand_u64()` in `stdlib/` and a clean userland `Treap<T, F>` (implicit sequence treap with Monoid + Action + reversible flag) so contestants almost never write `split`/`merge`/`push`/`pull` under contest pressure.

---

## 12. Custom Dijkstra, 0-1 BFS, and Multi-Dimensional Shortest Paths Bypassing `stdlib/graph.cp`

**Classification:** Userland `stdlib/graph.cp`

**Corpus incidence:** Manual `MinHeap` + `if d != dist[u]: continue` Dijkstra loops appear in **35 of 988** ports (**20 of 630** recent ports, **3.2%**); manual BFS queues with `var head = 0; head += 1` appear in **25** DP/graph/tree/DS ports (**4.1%**).

Why do contestants ignore `stdlib/graph.cp`'s `dijkstra` and `bfs`? Inspecting `stdlib/graph.cp` makes it obvious:
1. `dijkstra(next, from, w)` only accepts a single domain `V` (`[V]?Int`), so any **layered state graph** `(v: V, used: K)` (`cco22p3.cp`, `noi23p4.cp`, `ioi11p4.cp`) or **multi-source** start (`ioi11p4.cp`, `cco24p1.cp`) cannot call it.
2. `dijkstra` returns only `[V]?Int` — if you need the shortest-path tree (`parent: [V]?V`), path counts (`ways`), or custom second-shortest relaxation (`ioi11p4.cp`), you must rewrite the entire heap loop from scratch.
3. There is **no `bfs01` (0-1 BFS via `Deque`)** in `stdlib/graph.cp`, so every 0-1 shortest path hand-writes a `Deque` loop.

```cp
// proto/ports/cco24p1.cp (and hillclimb_dp_graph/ioi11p4.cp, scout_batch_30/cco22p3.cp):
var pq = MinHeap<(Int, V)>()
for v in V:
    best[v] = a[v]
    pq.push((-best[v], v))
while let (neg_d, u) = pq.pop():
    let d = -neg_d
    if d != best[u]: continue
    for (v, w) in g[u]:
        if best[v] < d - w:
            best[v] = d - w
            pq.push((-best[v], v))
```

```cp
// proto/ports/scout_batch_52/ctsc04p4.cp (manual BFS/DFS queue when only a stack/queue walk is needed):
var stk: List<V> = [root]
d[root] = 0
while let u = stk.pop():
    for (v, w) in g[u]:
        if d[v] == none:
            d[v] = d[u]! + w
            stk.push(v)
```

**Ideal CP Lang `0.2.2` replacement:**

```cp
// Multi-source / max-heap / parent-tracking / 0-1 BFS all in stdlib/graph.cp:
let dist = dijkstra_from(g, seeds: [V] v => a[v], dir: Max)
let (dist, parent) = dijkstra_tree(g, from: s)
let dist = bfs01(g, from: s)                  // uses Deque push_front / push_back
let dist = dijkstra_state<State>(from: start, next: s => ...)  // works on any Index tuple (V, K)
```

**Why this belongs in userland `stdlib/`:** `stdlib/graph.cp` already owns `dijkstra` and `bfs`. Generalizing `dijkstra` to accept any `Index` type (including tuples `(V, K)`!), multi-source seeds, optional `parent` return, and adding `bfs01` removes ~40 manual priority-queue/deque loops without touching the compiler.

---

## 13. DSU Extensions: Directed / Kruskal Reconstruction Trees & Small-to-Large `.absorb(mut other)`

**Classification:** Userland `stdlib/dsu.cp` and container `.absorb(mut other)` across `stdlib/` (`Heap`, `HashMap`, `Deque`, `List`)

**Corpus incidence:** Hand-written `find` loops for Kruskal reconstruction trees (`ioi18p3.cp`, `noi19p4.cp`) and small-to-large `if a.len < b.len: swap(mut a, mut b)` + drain loops (`ioi11p2.cp`, `btoi04p3.cp`, `iati21a12.cp`, `noi22p1.cp`, `noi20p2.cp`) appear across advanced tree and DS ports.

`Dsu<V>` always picks the union leader by size/rank, which breaks two classic CP patterns: **Kruskal reconstruction trees** (where the new vertex `v` must become the root of every merged component) and **next-alive / interval-skipping DSU** (`unite_into(i, i.next)`). So contestants rewrite `find` with path compression by hand:

```cp
// proto/ports/scout_batch_26/ioi18p3.cp:31-38
fn reach_tree(order: List<V>, earlier: (V, V) -> Bool):
    var top: [V]V = [V] v => v
    var kids: [V]List<(to: V)> = [V] _ => []
    fn find(v: V) -> V:
        var x = v
        while top[x] != x:
            top[x] = top[top[x]]
            x = top[x]
        x
```

Meanwhile, **small-to-large (DSU-on-tree / slope-trick heap merging)** repeats the exact same swap-and-drain dance on heaps, maps, and custom structures:

```cp
// proto/ports/scout_batch_38/btoi04p3.cp:22-23
if l.heap.len < r.heap.len: swap(mut l.heap, mut r.heap)
while let v = r.heap.pop(): l.heap.push(v)
```

```cp
// proto/ports/scout_batch_45/iati21a12.cp:
if pile[p].lots.len < pile[u].lots.len: swap(mut pile[p], mut pile[u])
while let (value, count) = pile[u].pop(): pile[p].push(value, count)
```

```cp
// proto/ports/ioi11p2.cp:24-29
var there = take(reach[child])
if there.len > here.len: swap(mut here, mut there)
for (far, hops) in there:
    if let other = here.get(k + 2 * far_v - far):
        fewest min= hops + other - 2 * deep_v
for (far, hops) in there: here[far] min= hops
```

**Ideal CP Lang `0.2.2` replacement:**

```cp
// 1. Directed union on Dsu<V> (makes `parent` the new leader):
var dsu = Dsu<V>()
if let r = dsu.unite_into(parent: v, child: u): kids[v].push(r)

// 2. Small-to-large container absorb:
l.heap.absorb(mut r.heap)                     // O(min(|l|, |r|) log n), swaps internally
here.absorb(mut reach[child], (a, b) => min(a, b))
```

**Why this belongs in userland `stdlib/`:** `Dsu.unite_into(parent, child)` is 6 lines in `stdlib/dsu.cp` (path-compresses `child` and sets `p[r] = parent` without union-by-size). `.absorb(mut other)` on `MinHeap`, `MaxHeap`, `HashMap`, and `Set` turns every small-to-large merge into a single readable line with optimal `O(n log n)` amortized complexity.

---

## 14. Negated-Key `MinHeap` Hacks, Tuple `.0`/`.1` Indexing, and Manual ODT Interval Splits

**Classification:** Userland `stdlib/heap.cp` & `stdlib/map.cp` (`Intervals<K, V>`) + Language core (`while let (a, b) = ...` parity)

**Corpus incidence:** Anonymous tuple `.0`/`.1`/`.2` indexing appears in **112 of 988** ports (**11.3%**, and **81 of 630** recent ports, **12.9%**); negated-key `MinHeap` hacks appear in `cco24p1.cp`, `noi07p6.cp`, `cco20p5.cp`, and more.

Even though `MaxHeap<T>` exists in `stdlib/heap.cp`, contestants still push negated keys into `MinHeap` whenever they need a **mixed sort order** (e.g., max depth, then min vertex ID) or because they forget `MaxHeap` exists — and negating a domain index forces an ugly `.idx` round-trip!

```cp
// proto/ports/scout_batch_30/noi07p6.cp:27-34
let u = leaves[i]
pq.push((-tree.depth(u), -u.idx))

var vis: [Node]Bool = false
var moves: List<(Node, Node)> = []
while let top = pq.pop():
    let u = Node(-top.1 + 1)
    if vis[u]: continue
```

Notice two separate smells in those 7 lines: `-u.idx` reconstructed via `Node(-top.1 + 1)` (a textbook 0-based vs 1-based bug factory!), AND `top.1` instead of destructuring `while let (depth, u) = pq.pop():`.

Similarly, ordered-map interval maintenance (Old Driver Tree / Chtholly Tree) appears across Olympiad DS ports (`scout_batch_51/egoi22p4.cp`, `scout_batch_38/apio09p2.cp`, `scout_batch_24/noi07p4.cp`) and always requires 15–25 lines of error-prone `last_le` / `last_lt` split-and-erase code.

**Ideal CP Lang `0.2.2` replacement:**

```cp
// 1. Keyed/reverse-order heap wrapper (`Rev(x)` in stdlib/heap.cp, like Rust's Reverse):
var pq = MaxHeap<(Int, Node)>()               // or MinHeap<(Rev<Int>, Node)> for mixed order!
while let (_, u) = pq.pop():

// 2. Userland Intervals<K, V> (ODT) over Map in stdlib:
var runs = Intervals<Pos, City>(initial)
runs.assign(l..=r, city, on_erase: (seg, old_city) => ...)
```

**Why this split respects the concept budget:** `Rev<T>` is a 4-line `Ord` wrapper in `stdlib/heap.cp` that works with `MinHeap`, `sort`, and `SortedBag` simultaneously, completely eliminating `-u.idx` hacks. `Intervals<K, V>` is a ~40-line userland `stdlib/` wrapper over `Map`. Ensure `while let (a, b) = expr:` tuple destructuring works everywhere `let (a, b)` works in the compiler.

---

## 15. Manual Binary Search, Functional-Graph Binary Lifting, and Top-2 Trackers

**Classification:** Userland `stdlib/seq.cp` & `stdlib/graph.cp`

**Corpus incidence:** Hand-written `while lo < hi:` binary searches appear in **52 of 988** ports (**5.3%**, and **32 of 630** recent ports, **5.1%**); manual top-2 trackers (`best`, `second`) appear in **5** major tree/DP/graph ports (`dmpg15g6`, `noip19p6`, `ioi11p1`, `noip14p2`, `ioi09p7`); manual `LOG = 18` jump tables appear whenever binary lifting runs on a functional graph rather than a `Rooted` tree.

Even in recent `scout_batch_26` and `scout_batch_30` ports, contestants write 6-line binary searches just to count how many sorted positions fall in `[l, r)` or to search an implicit monotone predicate:

```cp
// proto/ports/scout_batch_26/ioi09p7.cp:
fn before(x: Region, p: Int) -> Int:
    var (lo, hi) = (0, spots[x].len)
    while lo < hi:
        let mid = (lo + hi) / 2
        if spots[x][mid] < p: lo = mid + 1
        else: hi = mid
    lo
```

`Rooted<V>` has `.lift_while`, yet functional graphs / greedy interval jumping (`apio09p2.cp`) have no `JumpTable<D>` helper, forcing a hardcoded `const LOG = 18` every time:

```cp
// proto/ports/scout_batch_38/apio09p2.cp:34-40
// FRICTION 38.6: functional-graph binary lifting has no stdlib helper
// (unlike Rooted<V>.lift_while on trees), so every jump table repeats LOG + loop.
const LOG = 18
var lift: [0..=LOG, I]?I = none
for i in I: lift[0, i] = next_from(segs[i].r, I.first, I.last)
for k in 1..=LOG:
    for i in I:
        if let mid = lift[k - 1, i]: lift[k, i] = lift[k - 1, mid]
```

And tracking the top-2 distinct candidates (best and second-best child/edge/frequency) is written by hand with error-prone tie-breaking every time:

```cp
// proto/ports/scout_batch_27/ioi11p1.cp (and scout_batch_26/noip19p6.cp, scout_batch_24/dmpg15g6.cp):
var best: [F]?Trail = none
var second: [F]?Trail = none
let t = if s.to_idx % 2 == 1: second[f] ?? best[f]! else: best[f]!
```

**Ideal CP Lang `0.2.2` replacement:**

```cp
// 1. Sorted-slice helpers + binary search in stdlib/seq.cp:
let cnt = spots[x].count_lt(p)                // and .count_in(l..=r), .lower_bound(p)
let ans = first_true(lo..=hi, x => ok(x))

// 2. Functional-graph JumpTable<D> in stdlib/graph.cp:
let jump = JumpTable<I>(i => next_from(segs[i].r))
let (steps, end) = jump.steps_while(start, i => segs[i].r <= limit)

// 3. Top2<K, V> (distinct-key top 2) in stdlib/seq.cp:
var top: [F]Top2<Trail, Int> = Top2()
let t = top[f].best_except(blocked)
```

**Why this belongs in userland `stdlib/`:** `first_true` / `last_true`, `count_lt` / `count_in`, `JumpTable<D>`, and `Top2<K, V>` are pure, tiny, zero-overhead `stdlib/` utilities that eliminate three of the most repetitive algorithmic templates in CP.

---

## 16. Subset / Bitmask DP Friction: Missing `.first` Bit Peeling, Non-Index Fixed Arrays, and Manual Bit Shifts

**Classification:** Userland `stdlib/subset.cp` (`Subset.first`, `Subset.full`, `Int.bit(b)`) + Language core (allow fixed small arrays `[K]T` to derive `Index` / `@memo` keys)

**Corpus incidence:** `Subset<D>` appears in **33** ports and manual bit shifts (`>>`, `<<`, `1 <<`) appear in **44** DP/graph/tree/DS ports (**7.2%**).

Three specific bitmask/state-DP warts recur:

1. **`Subset<D>` has no `.first` (lowest set element via hardware `ctz`) and `Subset<D>.full` is missing or undocumented**, so O(2^n) subset-sum DP either scans linearly or abandons `Subset<D>`, and building the full mask takes a loop:
```cp
// proto/ports/scout_batch_38/btoi08p6.cp:10-12
// FRICTION 38.5: Subset<C> has no .first (lowest set element / ctz) or .empty,
// so O(2^n) subset-sum DP cannot peel one bit off `s: Subset<C>` without an
// O(n) scan; we double a list instead.
```
```cp
// proto/ports/scout_batch_42/mccc3j5.cp:
var full = Subset<V>()
for v in V: full.add(v)
```

2. **Bit-level DP/DS still drops to `.to_idx` shifts** (`scout_batch_50/noi22p1.cp`, `scout_batch_30/noi06p1.cp`):
```cp
// proto/ports/scout_batch_50/noi22p1.cp:
for b in Bit: self.ones[b] += d * (x >> b.to_idx & 1)
```

3. **Small fixed-size arrays `[0..5]0..=5` cannot be used as `@memo` keys** (`proto/ports/scout_batch_49/ioi01p3.cp`), forcing a 5-tuple unpacked to a `List` and repacked on every transition:
```cp
// proto/ports/scout_batch_49/ioi01p3.cp:14-21
fn ways(shape: Shape) -> Int:
    let len = [shape.0, shape.1, shape.2, shape.3, shape.4]
    let letter = sum(len)
    if letter == 25: return 1
    var total = 0
    for r in 0..5 if len[r] < 5 and (r == 0 or len[r - 1] > len[r]):
        if (pin[letter] ?? r * 5 + len[r]) == r * 5 + len[r]:
            var grown = len.copy()
```

**Ideal CP Lang `0.2.2` replacement:**

```cp
// 1. O(1) lowest-bit peeling and full mask on Subset<D> (stdlib):
let c = s.first!                              // lowers to __builtin_ctzll
sum_a[s] = sum_a[s - c] + a[c]
let full = Subset<V>.full

// 2. Typed bit test on Int (stdlib):
for b in Bit if x.bit(b): self.ones[b] += d

// 3. Fixed small array as a valid @memo key (core Index derivation):
@memo
fn ways(len: [0..5]0..=5) -> Int:
```

**Why this split respects the concept budget:** `Subset.first`, `Subset.full`, `Subset.is_empty`, and `Int.bit(b)` are pure `stdlib/`. Extending `Index` derivation to fixed-length arrays `[K]T` where `K` is a compile-time constant and `T: Index` uses the exact same product-space indexing rule tuples already use — zero new user-facing concepts, huge win for profile/broken-profile/Young-tableau DP.

---

## 17. Borrowing, `for x in mut xs`, Same-Container Swaps, and Scoped Backtracking

**Classification:** Language core (`for x in mut xs` and `swap_at(i, j)` / `with`/`do` block scope for `defer`) + Userland `stdlib/seq.cp` (`Perm.swap_pos`)

**Corpus incidence:** Explicit `mut fn` declarations appear in **7** heavy DS ports (`cco18p5`, `ceoi22p1`, `dmpg15g6`, `noi05p2`, `noi20p2`, etc.); index-loop workarounds for mutating elements in place and manual two-array inverse-permutation swaps appear across `scout_batch_34`, `38`, and `49`.

Four small borrow/mutation rough edges slow down contestants relative to C++'s `for (auto& c : cands)` and `swap(p[i], p[j])`:

1. **No `for c in mut list:` mutable iteration**, forcing raw `0..list.len` index loops:
```cp
// proto/ports/scout_batch_49/ioi12p2.cp:34-35
// FRICTION: no `for c in mut list`; elements are reached by position to mutate them
for i in 0..self.cands.len: self.cands[i].link(u, v)
```

2. **Swapping two elements of the same array / maintaining a permutation and its inverse** (`dmopc21c2p2.cp`, `ioi15p5.cp`) either trips over same-container `mut` aliasing (`if i == j: return`) or repeats 6 lines of inverse bookkeeping:
```cp
// proto/ports/scout_batch_34/dmopc21c2p2.cp:
fn swap_pos(i: Pos, j: Pos):
    if i == j: return
    let (u, v) = (perm[i], perm[j])
    perm[i] = v
    perm[j] = u
    inv[u] = j
    inv[v] = i
```

3. **No anonymous block scope for `defer` in sequential backtracking branches**, forcing manual push/pop pairs (`scout_batch_38/btoi17p4.cp:37-43`):
```cp
// proto/ports/scout_batch_38/btoi17p4.cp:37-43
// FRICTION 38.9: no anonymous block for `defer`, so sequential
// backtracking branches cannot scope `defer` without a helper fn.
state[v] = In
incl.push(v)
dfs()
incl.pop()
state[v] = Out
```

4. **Nested `mut fn` ceremony in stateful helpers** (`scout_batch_24/cco18p5.cp`, `dmpg15g6.cp`): local helper functions capturing outer `var`s require annotating `mut fn` even though top-level scripts already allow nested `fn`s to read/mutate outer `var`s freely (making the rule inconsistent between top-level and inside a function).

**Ideal CP Lang `0.2.2` replacement:**

```cp
// 1. Mutable element iteration (matches C++ `for (auto& c : cands)`):
for c in mut self.cands: c.link(u, v)

// 2. Built-in container swap + inverse-tracking Perm in stdlib:
perm.swap(i, j)                               // safe even when i == j; Perm<D> updates inv too!

// 3. Scoped rollback block (or Journaled/undo from plan/14-state.md):
undo:
    state[v] = In; incl.push(v); dfs()
```

**Why this split respects the concept budget:** `Array.swap(i, j)` and `Perm.swap(i, j)` are trivial `stdlib/` methods (backed by one safe internal primitive). Adding `for x in mut xs:` reuses the existing `mut` keyword with zero new concepts. Unifying nested `fn` capture rules (or keeping `mut fn` strictly documented and uniform) removes surprise borrow-checker rejections under contest pressure.

---

## 18. Live Compiler & Prelude Gaps Forcing Ugly Workarounds in Runnable `0.2.2` Ports

**Classification:** Compiler & prelude parity (`crates/` + `stdlib/` — **zero new language concepts!**)

**Corpus incidence:** Every runnable port in `proto/ports/blind_critic_5/` (`ioi19p5.cp`, `noi04p3.cp`, `noi08p4.cp`, `noi22p5.cp`) carries a header listing the exact `0.2.2` features that the contestant had to rip out and rewrite in ugly C-style code just to compile and run with `cpl` today.

This is the single highest-impact gap between the *designed* CP Lang `0.2.2` and the *runnable* CP Lang `0.2.2`:

1. **`(mp[k] ??= []).push(x)` rejected by parser** (`blind_critic_5/ioi19p5.cp:9,65`):
```cp
// proto/ports/blind_critic_5/ioi19p5.cp:65-69 (forced by parser gap)
if not mp.contains(key):
    var empty: List<Int> = []
    mp[key] = empty
mp[key].push(r * cols + c)
```

2. **Nested `@memo` unsupported**, turning a 12-line memoized tree DP into manual `memo_done` / `memo_val` flat arrays and clear loops (`blind_critic_5/noi08p4.cp:75-92`):
```cp
// proto/ports/blind_critic_5/noi08p4.cp:75-81
var memo_done: List<Bool> = []
var memo_val: List<List<Float>> = []
...
fn dp(u: Int, cdist: Int) -> List<Float>:
    let slot = u * stride + cdist
```

3. **Multi-dimensional optional domain DP tables (`[Col.bounds, ?Col, VDir]?Int = none`) degraded to flat `List<?Int>` push loops** (`blind_critic_5/noi04p3.cp:72-78`):
```cp
// proto/ports/blind_critic_5/noi04p3.cp:72-78
var dp: List<?Int> = []
var nxt_dir: List<Int> = []
var nxt_enc: List<Int> = []
for _ in 0..((m + 1) * states):
    dp.push(none)
    nxt_dir.push(0)
    nxt_enc.push(0)
```

4. **Missing runnable `Fenwick`, `LazySegTree`, and `SortedBag` wiring** forced `blind_critic_5/noi22p5.cp` to hand-write a 25-line `struct Fenwick` and a 70-line `struct LazySegTree` from scratch just to pass the compiler — ballooning a 90-line clean `0.2.2` draft into a 210-line C-style file!

5. **Minor lowering/runtime leaks** that still show up in ports: `popcount(mask.idx)` instead of `mask.len` (`research/archive/frictions/friction-backlog-022-migration.md`) and manual `" ".join` string-builder loops (`proto/ports/scout_batch_27/noi10p4.cp`).

**Ideal CP Lang `0.2.2` replacement:**

```cp
// All four already exist in the 0.2.2 design — just make the Rust frontend/prelude run them:
(mp[key] ??= []).push(r * cols + c)
@memo fn dp(u: V, cdist: Dist) -> [K]Float: ...
var dp: [Col.bounds, ?Col, VDir]?Int = none
var bit = Fenwick<Pos, Int>(); var st = LazySegTree<Val>(AddMin, 0)
```

**Why this comes first:** Shipping compiler/prelude parity for features `0.2.2` *already designed* costs **zero concept budget** and instantly deletes the ugliest 50% of code in runnable ports.

---

## Classification Summary and Recommended Execution Order

Respecting Ryan's userland-first principle and one-page concept budget, the 18 findings partition cleanly without bloating the language core:

- **Purely Userland `stdlib/` (10 patterns):** #4 (`Graph` unweighted/weighted/reversed & `Rooted.from_parents`), #6 (clamped `.back(k)`/`.ahead(k)`, `D.tail`/`D.init`), #7 (`Rooted.reroot` & `exclude_self`), #8 (`Rooted.at_centroid`, `in_preorder`, `fold_edge_path`, unified `t.Pre`), #9 (`SegTree.update`/`.add`, `ArgMax`/`AddMinCount`/`AssignAdd`, fill constructors), #10 (`CoverTree`, `TimeSegTree`, `LiChao`), #11 (`Pool.free`, `Treap<T, F>`), #12 (`dijkstra` over any `Index` tuple + multi-source + `dijkstra_tree` + `bfs01`), #13 (`Dsu.unite_into` + container `.absorb(mut other)`), and #15 (`count_lt`/`count_in`/`first_true`, `JumpTable<D>`, `Top2<K, V>`).
- **Compiler & Prelude Parity — zero new syntax (1 pattern):** #18 (`(mp[k] ??= []).push(x)`, nested `@memo`, multi-D `?Int` domain tables, runnable `Fenwick`/`LazySegTree`/`SortedBag` prelude wiring, `Subset.len` lowering).
- **Small Language Core Refinements (7 targeted lifts across #1, #2, #3, #5, #14, #16, #17 — zero new keywords):**
  1. Complete tropical `?T` lifting: `chmin`/`chmax` (or `min=`/`max=` returning `Bool`), expression-form `min(?T, ?T)`/`max(?T, ?T)`, and `?Int += Int` (#3).
  2. Uniform index arithmetic (`b - a -> Int`, `b + k -> ?Bounds<D>`) and zero-cost sub-domain widening (`domain Kid = V.tail` coerces to `V`) (#1).
  3. Local non-empty narrowing for `for m in l..r:` (`m.next` is `D`) and `if not xs.is_empty:` / optional chaining `xs.last?.next` (#2).
  4. `for x in mut xs:` mutable iteration and unified `while let (a, b) = ...` destructuring (#14, #17).
  5. Derive `Index` for compile-time-sized arrays `[K]T` so small fixed state vectors work directly as `@memo` keys (#16).

**Highest-leverage next wave (in order):**
1. Close #18 (compiler & prelude parity) so runnable ports stop falling back to C-style arrays and hand-rolled `Fenwick`/`LazySegTree`.
2. Ship the tiny core consistency fixes (#3 tropical `?T`, #1 sub-domain widening & `Bounds` distance, #2 `m in l..r` narrowing, #17 `for x in mut xs`) — zero new keywords, massive `!` and `.idx` elimination across >30% of the corpus.
3. Land the high-frequency `stdlib/` ergonomics wave (#4 `Graph`/`Rooted` constructors, #6 `.back(k)`/`D.tail`, #7 `Rooted.reroot`, #8 `Rooted` centroid/preorder, #9 `SegTree.update` + stock monoids, #12 generalized `dijkstra` + `bfs01`, #13 `Dsu.unite_into` + `.absorb`, #16 `Subset.first`/`Subset.full`).
4. Add the Olympiad-grade userland structures (#10 `CoverTree`/`TimeSegTree`/`LiChao`, #11 `Pool.free`/`Treap`, #14 `Rev`/`Intervals`, #15 `JumpTable`/`Top2`).

---

## Corpus Files Audited in Full (193 `.cp` Files)

- **Top-level `proto/ports/` (43):** `btoi17p3.cp`, `btoi17p6.cp`, `btoi18p6.cp`, `ccc00s4.cp`, `ccc07s4.cp`, `ccc10s4.cp`, `ccc15s4.cp`, `cco18p1.cp`, `cco24p1.cp`, `ceoi09p3.cp`, `ceoi18p1.cp`, `ceoi19pp1.cp`, `coci06c1p4.cp`, `coci08c6p4.cp`, `coci13c2p4.cp`, `coci15c2p4.cp`, `coci15c4p4.cp`, `coci18c1p3.cp`, `coci20c6p3.cp`, `coci22c2p3.cp`, `dmpg16s5.cp`, `dpa.cp`, `dpb.cp`, `dpc.cp`, `dpd.cp`, `dpe.cp`, `dpf.cp`, `dpg.cp`, `dph.cp`, `dpi.cp`, `dpj.cp`, `dpk.cp`, `dpl.cp`, `dpm.cp`, `dpn.cp`, `dpo.cp`, `dpp.cp`, `dpq.cp`, `dpr.cp`, `dps.cp`, `dpt.cp`, `ioi11p2.cp`, `nccc10s2.cp`.
- **`proto/ports/hillclimb_dp_graph/` (15):** `dpo.cp`, `dpp.cp`, `dpv.cp`, `ioi11p4.cp`, `ioi14p2.cp`, `ioi14p3.cp`, `noi05p2.cp`, `noi10p3.cp`, `noi15p2.cp`, `noi18p1.cp`, `noip14p2.cp`, `noip18p3.cp`, `usaco19decg3.cp`, `usaco20janp3.cp`, `usaco21janp2.cp`.
- **`proto/ports/scout_batch_24/` (10):** `cco16p5.cp`, `cco18p5.cp`, `ceoi22p1.cp`, `dmpg15g6.cp`, `ioi07p3.cp`, `ioi13p3io.cp`, `noi05p2.cp`, `noi07p4.cp`, `noi20p2.cp`, `noip22p4.cp`.
- **`proto/ports/scout_batch_26/` (10):** `ceoi22p3.cp`, `dmopc20c7p4.cp`, `ioi09p7.cp`, `ioi18p3.cp`, `noi14p4.cp`, `noi20p2.cp`, `noip19p2.cp`, `noip19p6.cp`, `sdoi17r1p2.cp`, `usaco22decp3.cp`.
- **`proto/ports/scout_batch_27/` (10):** `cco19p2.cp`, `ceoi17p1.cp`, `ceoi17p4.cp`, `ioi11p1.cp`, `ioi18p3.cp`, `noi07p1.cp`, `noi10p3.cp`, `noi10p4.cp`, `noi18p1.cp`, `noip14p2.cp`.
- **`proto/ports/scout_batch_28/` (10):** `ccc15s5.cp`, `cco17p3.cp`, `cco20p2.cp`, `ceoi18p2.cp`, `dwite10c3p3.cp`, `ioi04p2.cp`, `ioi09p4.cp`, `ioi11p5.cp`, `noi08p2.cp`, `usaco19febg3.cp`.
- **`proto/ports/scout_batch_30/` (10):** `cco16p6.cp`, `cco17p2.cp`, `cco21p5.cp`, `cco22p3.cp`, `coci21c1p5.cp`, `noi06p1.cp`, `noi07p6.cp`, `noi19p4.cp`, `noip18p2.cp`, `usaco21decg3.cp`.
- **`proto/ports/scout_batch_34, 36, 38, 42, 45, 48/` (30):** `dmopc20c6p3.cp`, `dmopc21c2p2.cp`, `dmopc21c2p4.cp`, `dmopc21c4p2.cp`, `dmopc22c5p4.cp`, `ccc19s5hard.cp`, `cco19p4.cp`, `cco20p5.cp`, `cco21p2.cp`, `cco22p1.cp`, `apio09p1.cp`, `apio09p2.cp`, `btoi04p3.cp`, `btoi08p6.cp`, `btoi17p4.cp`, `mccc3j3.cp`, `mccc3j5.cp`, `mccc4j3.cp`, `mccc4j5.cp`, `mccc5j4.cp`, `iati21a12.cp`, `iati21a21.cp`, `iati21a22.cp`, `iati22a11.cp`, `iati22a23.cp`, `btoi06p5.cp`, `btoi11p6.cp`, `btoi13p2.cp`, `btoi14p4.cp`, `btoi16p3.cp`.
- **`proto/ports/scout_batch_49, 50, 51, 52/` (45):** `ioi00p2.cp`, `ioi01p3.cp`, `ioi02p4.cp`, `ioi04p6.cp`, `ioi05p6.cp`, `ioi06p4.cp`, `ioi12p2.cp`, `ioi15p5.cp`, `ioi17p2.cp`, `ioi18p2.cp`, `ioi22p4.cp`, `ioi99p6.cp`, `noi11p4.cp`, `noi13p2.cp`, `noi15p3.cp`, `noi16p1.cp`, `noi18p2.cp`, `noi19p1.cp`, `noi20p5.cp`, `noi21p2.cp`, `noi22p1.cp`, `noip15p6.cp`, `noip17p6.cp`, `noip20p4.cp`, `apio23p2.cp`, `btoi21p1.cp`, `btoi22p4.cp`, `ceoi20p5.cp`, `egoi21p2.cp`, `egoi22p3.cp`, `egoi22p4.cp`, `egoi22p7.cp`, `joisc21d1p3.cp`, `joisc22d2p1.cp`, `ctsc04p4.cp`, `iati21a13.cp`, `ioi13p3.cp`, `ioi14p1.cp`, `noi11p2.cp`, `noi21p5.cp`, `noi22p4.cp`, `noi23p4.cp`, `noip18p6.cp`, `usaco22febb3.cp`, `usaco23jang2.cp`.
- **`proto/ports/blind_critic_5/` (10):** `bsspc21s4_draft.cp`, `bsspc21s4.cp`, `ioi19p5_draft.cp`, `ioi19p5.cp`, `noi04p3_draft.cp`, `noi04p3.cp`, `noi08p4_draft.cp`, `noi08p4.cp`, `noi22p5_draft.cp`, `noi22p5.cp`.
