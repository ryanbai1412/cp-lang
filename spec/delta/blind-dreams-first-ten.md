# CP Lang: ten ideal dreams

Each dream is the program I would most want to write, drafted from the idea without checking it against the spec. Under each one, **vs spec** lists every place it departs from the v0.3.0 spec: new syntax, a new or changed API, or a bug in `spec.cp` (the spec-compliant solution) where the spec already allows the dream's form.

## btoi20p3: BOI 2020 "Joker"

**Problem.** A graph has n vertices and m numbered streets. Each query (l, r) closes streets l..=r. Does what's left still have an odd cycle, i.e. is it non-bipartite?

**Sketch.** The streets left open after closing l..=r are one wrapping window of the street ring, from r+1 round to l-1, and "holds an odd cycle" survives growing that window. So a single two-pointer pass round the ring (`ring.minimal`) finds, for each end, the shortest window ending there that still holds an odd cycle. Query (l, r) asks whether that window fits in the m - len(l..=r) open streets. The state is a parity DSU, which can only undo its newest union; `undoing` (the queue-undo trick) turns it into a `Track` that drops from the front, with each union re-applied O(log m) times amortized.

```cp
let (n, m, q) = read<(1..=2e5, 1..=2e5, 1..=2e5)>()
domain V = 1..=n
domain Street = 1..=m
let street = read<[Street](V, V)>()
let odd = street.ring.minimal(
    undoing(ParityDsu<V>(), add: (mut d, (u, v)) => d.unite(u, v)),
    ok: d => not d.consistent,
)
for closed in read<[q]Range<Street>>():
    let shortest = odd[street.ring.prev(closed.l)]
    print(yes(shortest.is_some_and(w => w.len <= m - closed.len)))
```

**vs spec**

- **New syntax:** patterns in lambda parameters, e.g. `(mut d, (u, v)) =>`. Today parameters are names only.
- **API change:** `ok` gets the wrapped structure (`d.consistent`), not the adapter (`u.state.consistent`).
- **New:** `?T.is_some_and(p)`, replacing `.map(p) ?? false`.
- **Already in the spec:** `read<[q]Range<Street>>` reads `l r` as a range, so `spec.cp` should use it.

## cco21p4: CCO 2021 "Travelling Merchant"

**Problem.** Each directed trade edge u→v needs at least r money to use and pays out p. For every city, find the least starting money that lets you keep trading forever, or -1 if no amount does.

**Sketch.** First peel off cities that only lead to dead ends, using topological peeling on the reversed graph. Then settle answers from the largest requirement down. Each arc carries its road's `r`, `p` and id `e`, on both `g` and `g.transpose()`. A max-heap holds candidate needs, and popping a road retires it. When a city's last live road retires, the popped value is its answer, and every road into it is pushed again with `max(a.r, need - a.p)`.

```cp
let (n, m) = read<(1..=2e5, 1..=2e5)>()
domain V = 1..=n
let roads = read<[m](u: V, v: V, r: 0..=1e9, p: 0..=1e9)>()
let doomed = V.mask(Graph.directed(roads).transpose().peel())
let g = Graph.directed(roads).only(v => not doomed[v])
let into = g.transpose()
var left = [V] u => g.deg(u)
var need = Settle<roads.keys, Int>(best: max)
for u in V:
    for a in g.out(u):
        need.offer(a.e, a.r)
var ans: [V]?Int = none
while let (road, x) = need.pop():
    let u = roads[road].u
    left[u] -= 1
    if left[u] == 0:
        ans[u] = x
        for a in into.out(u):
            need.offer(a.e, max(a.r, x - a.p))
print(ans, none: -1)
```

**vs spec**

- **New:** `Settle<K, P>(best:)`, a keyed frontier. Each key pops exactly once, at the best priority offered before it popped, and later offers are ignored. It replaces the `retired` side array and the lazy-deletion `continue`. Dijkstra and Prim use the same shape. `IndexedHeap` is close but min-only, and a popped key can come back.
- **New:** `g.only(keep)`, an induced-subgraph view. It replaces the four `if alive[a.to]` guards.
- **New:** `V.mask(vs)` builds a `[V]Bool` from a list or generator.
- **New:** `print(xs, none: -1)` prints absent values with a stand-in, instead of `ans.map(a => a ?? -1)`.

## ioi16p6: IOI 2016 "Aliens"

**Problem.** Points on an m×m grid must each be covered by one of at most k squares whose diagonal lies on the main diagonal. Minimise the total covered area, counting overlaps once.

**Sketch.** A point needs the diagonal span min(r,c)..=max(r,c). Drop spans contained in others; a photo then covers a contiguous run of the rest. The Aliens trick (`aliens`) replaces the photo limit with a penalty per photo. Each run's cost is a line in the left end of its first span, so the DP is a monotone hull whose lines carry their photo counts as tags.

```cp
let (n, m, k) = read<(1..=1e5, 1..=1e6, 1..=1e5)>()
domain Diag = 0..m
let spans = read<[n](r: Diag, c: Diag)>()
    .map(p => min(p.r, p.c)..=max(p.r, p.c))
    .outermost()

fn cover(fee: Int) -> (val: Int, count: Int):
    var hull = MonotoneHull<Int>()
    var best = (val: 0, count: 0)
    for (prev, s) in spans.with_prev:
        let (lo, x) = (Int(s.l), Int(s.r))
        let shared = prev.map(p => p.clip(s).len) ?? 0
        hull.add(Line(-2 * lo, best.val + lo * lo - shared * shared + fee), tag: best.count + 1)
        let b = hull.best_next(x)!
        best = (val: b.val + x * x, count: b.tag)
    best

print(aliens(k, cover, lambdas: 0..=m * m))
```

**vs spec**

- **New:** `rs.outermost()` on any sequence of ranges keeps the ranges no other range contains, sorted. It replaces three lines: sort by `(l, desc(r))`, a `scan(max)` of right ends, then a filter.
- **Otherwise the same as `spec.cp`.** The hull and aliens parts already read the way I'd want.

## noi09p4: NOI 2009 "Plants vs Zombies"

**Problem.** Plants on a grid each have a score, positive or negative, and a list of cells they protect. Each plant is also protected by the plant to its right. A plant can be eaten only once all its protectors are eaten. Maximise the total score eaten.

**Sketch.** Plants on a protection cycle, or protected from one, can never be eaten, and peeling removes exactly the others. Among the edible plants, an eaten set must be closed under "protected by", so the answer is a maximum-weight closure, computed by min cut.

```cp
let (rows, cols) = read<(1..=20, 1..=30)>()
domain Cell = ([rows], [cols])
let plant = read<[Cell](score: -1e4..=1e4, protects: List<Cell>)>()
fn protects(p: Cell) = chain(plant[p].protects, p.step(Dir4.left))
let edible = Cell.mask(peel(protects))
let eaten = max_closure(
    Cell,
    before: p => protects(p).filter(q => edible[q]),
    profit: p => if edible[p]: plant[p].score else: 0,
)
print(eaten.value)
```

**vs spec**

- **API change:** `max_closure(V, before:, profit:)` takes the same graph-shaped arguments as every other graph algorithm. Today it takes `[V]Int` plus an edge list, so the dream had to build `needs` by hand.
- **Small checker change:** `peel(protects)` infers the domain from the parameter type, so the `peel::<Cell>` turbofish isn't needed.
- **New:** `Cell.mask(...)`, the same as in cco21p4.
- **Already in the spec:** a record with `protects: List<Cell>` reads a count-prefixed list, so `spec.cp`'s read-then-push loop is a `spec.cp` idiom bug.

## etsr23r3p3: NOI 2018 "Return" (online)

**Problem.** Roads have a length and an altitude. A query (v, p) floods every road with altitude ≤ p. You drive for free from v over unflooded roads, then walk home to vertex 1. Minimise the walking distance. Queries are forced online.

**Sketch.** Walking distance is the shortest distance home (Dijkstra) from any vertex the car can reach. Merge roads from the highest down into a Kruskal reconstruction tree: the vertices reachable above level p are one subtree, found by binary lifting. A bottom-up `min` over the tree gives each subtree's best walk.

```cp
@!multitest
let (n, m) = read<(1..=2e5, 1..=4e5)>()
domain V = 1..=n
let roads = read<[m](u: V, v: V, len: 1..=1e4, alt: 1..=1e9)>()
let home = dijkstra(Graph.undirected(roads).weighted(.len), from: [V.first!]).dist
let (domain N, kt) = kruskal_tree(roads, by: desc(.alt))
let least = kt.tree.up(min, leaf: v => home[v]!)
let (q, k, s) = read<(1..=4e5, 0..=1, 0..=1e9)>()
var last = 0
for (v0, p0) in read<[q](V, 0..=s)>():
    let v = V.ring.step(v0, k * last)
    let p = (0..=s).ring.step(p0, k * last)
    last = least[kt.top(v, ok: road => road.alt > p)]
    print(last)
```

**vs spec**

- **New:** `D.ring.step(x, d)`, cyclic shifting within a domain. It replaces `V((v0 + k * last - 1) % n + 1)` and `(p0 + k * last) % (s + 1)`.
- **API change:** `kt.top(v, ok:)` tests the merging road's own fields. Today it's `kt.reach(v, limit: desc(p + 1))`, a threshold trick.
- **API change:** `up(min, leaf:)` seeds the leaves. It replaces the `walk` side array and its fill loop.
- **API change:** `from:` accepts bare sources at distance zero (`[V.first!]` instead of `[(V.first!, 0)]`), and `desc(.alt)` composes with field shorthand.

## coci21c2p4: COCI 2021/22 #2 "Magneti"

**Problem.** Place n magnets with radii r_i on l slots in a line. Two magnets that are neighbours must be at least max(r_i, r_j) apart. Count the arrangements mod 1e9+7.

**Sketch.** Use connected-component DP, inserting magnets from the largest radius down. Each magnet starts a new component, extends one, or joins two, which forces 2s, s or 0 extra length. The free slots left over are spread by stars and bars (`comb.choose`). The DP is one `@memo(array)` function over a typed state.

```cp
type Z = Mod<1e9 + 7>
let (n, l) = read<(1..=50, 1..=1e4)>()
let r = read<[n]1..=1e4>().sorted(by: desc)
let comb = Comb<Z>(l + n)
domain State = (comps: 0..=n, used: 0..l, ends: 0..=2)

@memo(array)
fn ways(i: 0..=n, st: State) -> Z:
    let (c, u, e) = st
    if i == n:
        return Z(c == 1 and u == l - 1 and e == 0)
    fn go(dc: Int, du: Int, de: Int) -> Z =
        State.at((c + dc, u + du, e + de)).map(t => ways(i + 1, t)) ?? 0
    let s = r[i]
    var total = (c - 1 + e) * go(1, 2 * s, 0) + e * go(1, s, -1)
    if c >= 1:
        total += (2 * c - 2 + e) * go(0, s, 0) + e * go(0, 0, -1) + (c - 1) * go(-1, 0, 0)
    total

print(if n == 1: l else: (0..l).sum(u => ways(0, (0, u, 2)) * comb.choose(u + n, n)))
```

**vs spec**

- **New:** product domains with named fields, e.g. `domain State = (comps: 0..=n, ...)`. `State.at(...)` returns `?State`, which replaces the hand-written bounds check in `next`. Only unnamed product domains exist today, and their `.at` is a checker gap.
- **New:** `Z(bool)` and `.sum(f)`.
- **Open:** `r[i]` with `i: 0..=n` indexes `[n]` with a counter. The ideal is `i: r.cuts` with `r.after(i)`, but I didn't find a spelling I liked.

## noip18p3: NOIP 2018 "Track Construction"

**Problem.** A weighted tree on n vertices. Build m edge-disjoint paths so that the shortest of them is as long as possible.

**Sketch.** Binary search on the length k. In a DFS, each vertex receives one chain from each child. Chains of length ≥ k close as tracks immediately. The rest are paired greedily in a sorted multiset: the shortest chain takes the shortest partner that completes it. The longest unpaired chain is passed up to the parent.

```cp
let (n, m) = read<(2..=5e4, 1..=5e4)>()
domain V = 1..=n
let t = Rooted(Graph.undirected(read<[n - 1](u: V, v: V, w: 1..=1e4)>()), root: V.first!)

fn tracks(k: Int) -> Int:
    var made = 0
    fn up(v: V) -> Int:
        var chains = SortedBag(t.down(v).map(a => up(a.to) + a.w))
        made += chains.drain(k..).count()
        var spare = 0
        while let x = chains.pop_first():
            if chains.take_first(y => x + y >= k) is some(_):
                made += 1
            else:
                spare = x
        spare
    up(t.root)
    made

print((1..=5e8).last_true(k => tracks(k) >= m)!)
```

**vs spec**

- **New:** `t.down(v)` yields the arcs into `v`'s children with their fields (`a.to`, `a.w`). It replaces `children(v)` plus `t.w(c)` or `up_arc(c)!.w`.
- **New:** sorted containers drain a value range (`chains.drain(k..)`), and `take_first(ok)` removes and returns the first element satisfying `ok`.

## apio09p2: APIO 2009 "Convention Center"

**Problem.** Given n intervals, find the maximum number of pairwise-disjoint ones and the lexicographically smallest set of that size.

**Sketch.** Greedy: after interval i, the best next pick is the interval starting after it with the smallest right end. Those links form a forest, so "how many fit inside [a, b]" is a depth difference, found with `jump_while`. Try intervals in index order against a sorted map of gaps; take one exactly when the gap's optimum is preserved.

```cp
let n = read<1..=2e5>()
domain I = 1..=n
domain X = 1..=1e9
let iv = read<[I]Range<X>>()

let by_start = I.sorted(by: i => iv[i].l)
let soonest = by_start.rev.scan(min(by: i => iv[i].r))
fn first_from(c: Cut<X>) -> ?I = soonest[by_start.partition_point(i => iv[i].l < c).r?]
let chain = Lift(i => first_from(iv[i].r))
fn fit(gap: Range<X>) -> Int:
    let u = first_from(gap.l) ?? return 0
    if iv[u].r > gap.r:
        return 0
    1 + chain.climb(u, ok: v => iv[v].r <= gap.r).steps

var gaps = IntervalMap<X, ?Int>(init: none)
gaps.assign(X.range, fit(X.range))
var picked = List<I>()
for i in I:
    let (gap, best) = gaps.run_at(iv[i].first!)
    let best = best ?? continue
    if not gap.contains(iv[i]):
        continue
    let (left, right) = (gap.l..iv[i].l, iv[i].r..gap.r)
    let (a, b) = (fit(left), fit(right))
    if a + 1 + b == best:
        picked.push(i)
        gaps.assign(left, a)
        gaps.assign(iv[i], none)
        gaps.assign(right, b)
print(picked.len)
print(picked)
```

**vs spec**

- **New:** `Lift(f)` builds binary lifting over any partial successor function `V -> ?V`. Today `Lift` needs a parent array plus an op and identity, so the dream built a `Rooted` and did depth arithmetic.
- **Design question:** `domain X = 1..=1e9` is a coordinate line with no arrays over it. Its cuts and ranges work without `compress`. I believe the spec allows this, but no example uses it.
- **Already in the spec:** `IntervalMap` covers "gap containing x, plus its optimum", and `Lift.climb` covers the chain count. `spec.cp`'s `SortedMap` + `last_le` and its `depth` differences are idiom bugs there.

## noi10p2: NOI 2010 "Super Piano"

**Problem.** Given a sequence of n values, a chord is a contiguous run whose length is between lo and hi. Sum the k largest chord sums.

**Sketch.** A chord is a pair of prefix cuts. For a fixed start and a range of end cuts, the best end is the highest prefix in that range, found with a sparse table of (prefix, cut). A max-heap holds (start, range) groups. Popping a group's best splits its range around the chosen end.

```cp
let (n, k) = read<(1..=5e5, 1..=5e5)>()
let len = read<Range<1..=n>>()
domain Note = 1..=n
let pre = read<[Note]-1_000..=1_000>().scan(+)
let peak = SparseTable(max, [Note.cuts] c => (pre[c], c))

fn best(from: Note.cuts, ends: Range<Note.cuts>) -> (Int, Note.cuts):
    let (p, at) = peak.fold(ends)
    (p - pre[from], at)

print(split_best(Note.cuts, range: s => s.shift(len), best).take(k).sum())
```

**vs spec**

- **New stdlib:** `split_best(groups, range:, best:)` yields values in decreasing order. Each group owns a range, and `best` gives a range's top value and its position. After a pop, the library re-queues the two pieces of the range on either side of that position. This is the "k best from a family of ranges" pattern (k largest subarray sums, k smallest pair sums, Super Piano). It replaces the `Group` type, the heap and the split logic, about 12 lines.
- **New:** `cut.shift(range)` gives the cuts at those distances, clipped to the domain and possibly empty. It replaces `from.next(lo)` / `from.next(hi) ?? last`.

## noi05p4: NOI 2005 "Cat and Mouse"

**Problem.** On a graph, each turn the cat takes up to two steps toward the mouse along a shortest path, breaking ties by smallest id. Then the mouse stays put or moves to a uniformly random neighbour. Find the expected number of turns until the cat catches the mouse.

**Sketch.** Distances come from all-pairs BFS. The cat's distance to the mouse strictly decreases each turn, so expected turns form an acyclic DP over (cat, mouse) states. It is written as a memoised function with no explicit topological order.

```cp
let (n, e) = read<(1..=1_000, 1..=1_000)>()
domain V = 1..=n
let (cat, mouse) = read<(V, V)>()
let g = Graph.undirected(read<[e](u: V, v: V)>())
let dist = [V] t => bfs(g, from: [t]).dist
fn toward(c: V, m: V) -> V = if c == m: m else: g.next(c).min(by: v => (dist[m][v]!, v))!

@memo(array)
fn turns(c: V, m: V) -> F64:
    if c == m:
        return 0.0
    let c2 = toward(toward(c, m), m)
    if c2 == m:
        return 1.0
    1.0 + chain([m], g.next(m)).mean(of: m2 => turns(c2, m2))

print(turns(cat, mouse), precision: 3)
```

**vs spec**

- **New:** `.mean(of:)`, replacing the hand-written average with `deg + 1`.
- **Already in the spec:** `g.next(c)` yields neighbours directly, so `spec.cp`'s `g.out(c).map(a => a.to)` is a `spec.cp` idiom bug.
