# Friction backlog, batch 27: hard graphs

A Stage-1 scout pass over ten graph submissions, solved from the
statements (DMOJ API metadata plus the published contest tasks) before the
originals were opened for their exact input format. The ports are in
[`proto/ports/scout_batch_27/`](../proto/ports/scout_batch_27/):
coci20c6p2, noi10p3, cco19p2, noi07p1 (shortest paths), ioi11p1, btoi18p1
(state spaces and functional graphs), ioi18p3, acc7p3, noi15p1 (Kruskal
and DSU) and noi10p4 (a DAG of deadlines). Items are numbered `HG1`... so
they do not collide with earlier batches.

What was checked: all ten ports run on the reference interpreter
(`proto/cpi`, with the real `stdlib/`) and agree with their originals on
100 random inputs each, half small and half medium
(`proto/ports/scout_batch_27/check.py`, on top of commit `7d1c976`). Every
"today" snippet is a line of a port or of a probe that was run on
`proto/cpi`, and every quoted message was produced by it. The "wanted"
snippets do not run. Nothing was built with `proto/cpc`, nothing was run
at full size, and nothing was timed.

What is missing: the unported corpus has no bridges or biconnected
components problem and no 0-1 BFS problem with an accepted submission, so
HG3 and HG9 argue from the ports that came closest rather than from a port
of that exact kind.

## HG1. A state space wants a product or sum domain, and gets `i * n + j`

Four of the ten ports search over something that is not a plain vertex:
blocks of a grid plus an "outside" (noi10p3), cells plus a floor
(cco19p2), (fountain, arrived by its best trail) (ioi11p1), and pieces
named by a DSU representative (cco19p2 again). The cheat sheet says tuples
and payload enums are indices. The interpreter says:

```
error: stdlib/dsu.cp:12: Tuple<Row, Col> is not an index type
error: p4.cp:1: `enum` statements are not supported by the reference interpreter
```

So every one of them is flattened by hand, today:

```cp
domain Node = 0..n * n + 1          // the blocks, then the north-east outside
let out = Node(n * n)
fn block(i: Int, j: Int) -> Node = Node(i * n + j)
...
let (i, j) = (B(u.to_idx / n), B(u.to_idx % n))
```

```cp
domain State = 0..2 * n                 // 2 * fountain + (arrived by its best trail)
fn state(f: F, by_best: Bool) -> State = State(2 * f.to_idx + (if by_best: 1 else: 0))
```

This is the exact arithmetic the domain system exists to remove, and it is
where a slip is easiest: the decode line of `noi10p3` mixes block and
crossing coordinates (`X(i.to_idx + 1)`) and nothing would catch `/` and
`%` being swapped.
Wanted, with no new feature beyond what the cheat sheet already promises:

```cp
enum Node:
    Outside
    Block(i: B, j: B)
let cost = dijkstra(steps, [(Block(i, B.first), south[i, X.first]) for i in B], s => s.cost)
print(cost[Outside]!)
```

```cp
type State = (F, Bool)
var back: [State]List<(to: State)> = []
```

Recommendation: algebraic `Index` (tuple, `Bool`, payload enum) is the
single most valuable thing for graph problems to graduate to the
interpreter. Until it does, every state-space port is a critique of
`to_idx` arithmetic rather than of the language.

## HG2. `dijkstra` takes an adjacency lambda, and that part is right

The one thing that needed no workaround. The same call served a stored
list, a list keyed by DSU representatives and a function that builds arcs
on the fly:

```cp
let dist = dijkstra(v => out[v], [(id(a), 0)], e => e.time)     // coci20c6p2
let drop = dijkstra(p => rests[p], [(floor, 0)], a => a.gap)    // cco19p2
print(dijkstra(steps, starts, s => s.cost)[out]!)               // noi10p3
```

`V`, the arc type and the weight type were all inferred, a named fn or a
`yields` fn is accepted as the adjacency, and multi-source starts with a
starting distance each removed the virtual source vertex from both noi10p3
and cco19p2. One friction, in a sibling: `topo` does not infer.

```cp
let order = topo(p => after[p])!
// error: stdlib/graph.cp:122: unknown type `V`
let order = topo::<Plane, (to: Plane)>(p => after[p])!          // what noi10p4 has
```

`bfs` and `dijkstra` take a `from:` list that pins `V`; `topo`, `scc` and
`components` take only the lambda, and the parameter type of an untyped
lambda is not enough. The message points into the library, not at the
call.

Recommendation: keep the lambda shape everywhere; infer `V` from the
adjacency's result type (`[V]List<...>` indexed by the parameter) or
accept `topo(Plane, p => after[p])`, and report the failure at the call
site.

## HG3. Priority order: `MinHeap` and `desc(x)` are on the cheat sheet only

No port in this batch needed a hand-written heap, because every shortest
path went through `dijkstra`. A probe shows what a port that did need one
(second-best arrival, Dijkstra with a tie-break, a "largest deadline
first" greedy for noi10p4) would meet today:

```cp
var q = MinHeap<(Int, Desc<Int>)>()      // error: unknown name `MinHeap`
```

What runs is the library heap with an explicit order:

```cp
use std.heap.{Heap}
use std.algebra.{Order, Less}
var h = Heap<(Int, Int)>.new(Less<(Int, Int)>)
h.push((3, 1))
while let (a, b) = h.pop(): print(a, b)
```

So there are three spellings of "pop the smallest distance, largest id on
ties": `MinHeap<(Int, Desc<V>)>` with `desc(v)` at every push and `.0` at
every pop (cheat sheet), `Heap<T>.new(Order<T>((a, b) => ...))` (stdlib,
and what `std.graph.shortest` itself uses), and the old `h.push((-x, x))`
trick that `proto/cpc` still suggests in an error message. `desc` is the
weakest of them for graphs: the wrapper leaks into the element type, so
the popped tuple no longer destructures into plain values, and a
comparison written once in an `Order` is instead repeated at each push.

Recommendation: one heap. `Heap<T>(by: key)` with a key lambda, where the
key may contain `desc(...)`, keeps `desc` out of the stored type:

```cp
var pq = Heap<(d: Int, v: V)>(by: e => (e.d, desc(e.v)))
while let (d, v) = pq.pop(): ...
```

and `MinHeap<T>` / `MaxHeap<T>` become two-line aliases of it or go.

## HG4. Building a weighted graph: three reads to get one edge in

`read()` cannot read a domain value, a tuple array, or run inside a
comprehension:

```
error: ioi18p3.cp:60: read() cannot read a Tuple<Int, Int, Int, Int>     // let trips: [Q]Trip = read()
error: ioi18p3.cp:60: unknown name `read`                                // [Q] _ => read()
```

so every port builds its graph the same long way:

```cp
var out: [V]List<(to: V, time: Int)> = []
for _ in 0..m:
    let (a, b, time): (Int, Int, Int) = read()
    out[V(a - 1)].push((to: V(b - 1), time: time))
    out[V(b - 1)].push((to: V(a - 1), time: time))
```

The `- 1` appears four times and the both-directions push is copy-paste;
`domain V = 1..=n` removes the `- 1` but then printing needs
`p.to_idx + 1` (noi10p4), because nothing obvious gives the label back.
`Graph<V, E>.undirected` did not help: it wants a `[E](u: V, v: V)` array,
which is exactly what cannot be read. Wanted:

```cp
domain V = 1..=n
let roads: [m](u: V, v: V, time: Int) = read()
let g = Graph.undirected(roads)              // arcs carry the whole record
let dist = dijkstra(g.adj, [(s, 0)], a => a.time)
for v in V: print(v, dist[v] ?? -1)          // a V prints as its label
```

Iterating weighted arcs was fine once the list existed
(`for a in after[p]: due[p] = min(due[p], due[a.to] - 1)`); the cost is all
in construction.

Names as vertices (coci20c6p2, btoi18p1) needed the same six lines twice:

```cp
var ids = Map<Str, P>()
fn id(name: Str) -> P:
    if name not in ids: ids[name] = P(ids.len)
    ids[name]
```

Recommendation: a stdlib `Interner<K, D>` (`ids.of(name) -> D`), and
`compress` for the offline case (noi15p1 hand-rolls it with a
`Map<Int, Int>` and then declares `domain Var = 0..ids.len` afterwards).

## HG5. Kruskal and DSU: the representative is not yours

Plain Kruskal reads well (acc7p3):

```cp
for g in sorted(gaps, by: g => g.cost):
    if net.unite(g.u, g.v): total += g.cost
```

The Kruskal reconstruction tree does not. `Dsu` unites by size, so the
caller cannot say which vertex represents the merged part, and ioi18p3
keeps a side array keyed by whatever `find` returns:

```cp
var newest = [City] c => c          // the latest city of each part, by its representative
for c in order:
    seen[c] = true
    for d in roads[c]:
        if not seen[d] or parts.same(c, d): continue
        kids[c].push((to: newest[parts.find(d)]))
        parts.unite(c, d)
        newest[parts.find(c)] = c
```

The last line is the classic bug: write `newest[parts.find(d)]` before the
`unite` instead of after and the tree is silently wrong. Batch 18's
noi18p1 has the same four lines with a different payload. Then the query
"highest ancestor still satisfying a monotone predicate" has no helper, so
the port binary-searches over `ancestor(v, k)`:

```cp
fn climb(t: Rooted<City>, v: City, ok: City -> Bool) -> City:
    var (lo, hi) = (0, t.depth[v])
    while lo < hi:
        let mid = (lo + hi + 1) / 2
        if ok(t.ancestor(v, mid)!): lo = mid
        else: hi = mid - 1
    t.ancestor(v, lo)!
```

Wanted, as userland on top of `Dsu` and `Rooted`:

```cp
let human = merge_tree(City.rev, c => roads[c])          // Rooted<City>; subtree = reach at that time
let top = human.climb(s, c => c >= l)                    // highest ancestor accepted by a monotone test
```

and for the edge-weighted form, `kruskal_tree(edges, by: e => e.w)`
returning a `Rooted` over `V + E` nodes. `Dsu` itself wants one addition:
`unite_into(a, b)` (b's part joins a's, a's representative survives) or a
payload merged on unite, so that "what do I know about this part" is not a
side array indexed by a moving key.

Two more from the same port. `Rooted<City>.Pre` is one domain shared by
every tree over `City`, so a human-tree position indexes a wolf-tree
Fenwick without complaint; the port had to write `Rooted<City>.Pre` where
it meant "the wolf tree's positions", and `wolf.Pre` is rejected
(`Rooted<City> has no field or method Pre`). And there is no way to make a
`Pre` from a number, so "the city at Euler position i" became
`sorted(City, by: c => human.tin[c].to_idx)`. The offline rectangle count
that finishes the problem (thirteen lines of events and a Fenwick) is the
third time a batch has written it; a `count_in_rects(points, rects)`
helper is due.

## HG6. Stress inputs: connected, DAG, feasible, planar

`check.py` has a hand-written generator per problem because none of these
promises fits in a range type:

- **Connected** (noi07p1, ioi18p3): a random tree plus extra edges. A
  typed `m` random pairs is disconnected almost always, and noi07p1
  unwraps the distance of every pair.
- **DAG with a feasible schedule** (noi10p4): the generator draws a hidden
  schedule first, then deadlines at or after each slot and rules that
  agree with it. "The rules are acyclic" alone is not enough; the promise
  is "a valid schedule exists", which is the answer to the problem.
- **Every name appears** (coci20c6p2), **distinct values** (acc7p3; the
  original keeps values in a map and silently breaks on a repeat).
- **Planar** (noi10p3): free here, because the planar graph is a grid and
  the input is four arrays. The dual graph is built by the solution, not
  read. A general planar input would have no generator at all.
- **Interesting, not just valid**: the first ioi18p3 generator answered 1
  for 133 of 140 queries, and half of btoi18p1 was `-1` for odd `n`. Each
  needed a nudge (narrow `[l, r]` on sparse graphs, mostly even `n`),
  found by counting outputs; noi15p1 keeps inequalities rare for the same
  reason.

Wanted: graph shapes as generator values, composable with the typed read:

```cp
let roads: [m](u: V, v: V, w: 1..=1000) = read(gen: gen.connected(V, m))
let rules: [m](a: Plane, b: Plane) = read(gen: gen.dag(Plane, m))
assume feasible(k, rules)          // rejection is fine when the shape is already right
```

with `gen.tree`, `gen.connected`, `gen.dag`, `gen.functional` and
`gen.grid_graph` in a userland `std.gen`, and `cpl stress` printing the
distribution of the brute's answers so a generator that only ever says
YES is visible at once.

## HG7. Grader-style problems

Both IOI originals are functions (`count_routes`, `check_validity`), not
programs. Each port invents an input format and `check.py` appends a
`main` to the original (ioi18p3) or builds it with `-DLOCAL` and feeds it
`input.txt` (ioi11p1). The cheat sheet names `@grader`; a port could then
state the judge's signature next to the sample-grader format, and
`cpl build` could emit the function the judge wants.

## HG8. Small things that cost a line each

- `while let p = stack.pop():` traps on a `List`
  (`pop from an empty list`): `List.pop` is not `?T` in the interpreter
  though `Heap.pop` is. Two ports wrote
  `while not stack.is_empty: let p = stack.pop()`.
- No `" ".join(...)`, so noi10p4 carries a seven-line `line(xs)` that
  pushes characters into a `Str`. No `{x:.3f}`, so noi07p1 carries
  `fixed3`; batch 21's `fixed` lives in that batch's private library.
- `all(...)` / `any(...)` over a generator are unknown names.
- `let x = if let d = opt: d + 1 else: 0` does not parse; ioi11p1 wrote
  `(dist[next(goal)] ?? -1) + 1`, which is worse to read.
- `[1..=n]Int` is rejected (`Int is not an index type`); a second
  `domain Slot = 1..=n` was declared just to count per slot.
- A grid of characters has to be `[Row]Str` indexed `board[r][c.to_idx]`;
  `[Row, Col]Char = read()` reads space-separated tokens and runs off the
  end of a packed grid.

## HG9. Where features collide

1. **Dynamic domain x DSU x Dijkstra (cco19p2).** The vertices of the
   shortest-path problem are pieces, known only after the DSU has run.
   There is no way to say "a domain of the representatives", so the port
   runs Dijkstra over all `n * m + 1` cells with empty lists for the
   non-representatives. `components(adj) -> (domain C, comp: [V]C)` does
   this for explicit graphs; `Dsu` needs the same: `let (domain Piece,
   of) = piece.freeze()`.
2. **Domain values x `read()` x 1-based labels (every port).** Typed
   vertices cannot be read, so they enter as `Int` and are converted at
   each use; the query record of ioi18p3 is `(s: Int, e: Int, l: Int, r:
   Int)` and the comparison `c >= l` is `c.to_idx >= l`. `s` and `e` are
   cities and `l`, `r` are thresholds on city numbers: the type system
   should tell them apart and cannot while both are `Int`.
3. **Per-tree `Pre` x two trees (ioi18p3).** See HG5: one nominal domain
   for two different orders removes the protection `Pre` was made for,
   exactly in the problem that has two Euler tours.
4. **Functional graph x reverse BFS (ioi11p1, btoi18p1).** A graph with
   one arc per vertex is a `[V]V` array, but `bfs` needs arcs as records
   with `to`, so ioi11p1 builds `[State]List<(to: State)>` only to walk
   backwards. `std.graph` has no functional-graph helper (cycle of each
   vertex, tail length, reverse lists); btoi18p1 writes cycle walking
   twice with an in-degree array reused as a "still on a cycle" mark.
5. **Optional distances x arithmetic (noi07p1).** Floyd-Warshall on
   `[P, P]?Int` is honest about unreachable pairs but every relaxation is
   `dist[s, t] == none or a + b < dist[s, t]!`. `min=` on an optional
   place exists on the cheat sheet; the port also has to update a second
   array when the minimum changes, and `min=` does not say whether it
   did. `if dist[s, t] min= a + b:` (true when it improved) would cover
   this and every hand-written relaxation.
6. **Topological order x DP (noi10p4).** `topo` returns a `?List<V>`
   and the deadline push is a loop over `order.rev` with an inner loop
   over arcs. That is fine once, but it is the shape of every DAG DP; a
   `dag_dp(adj, init, combine)` over reverse topological order would
   cover this and the usual longest-path DP, and would hide the `topo`
   turbofish of HG2.

## Ranked

1. HG1: algebraic indices in the interpreter (tuple, `Bool`, payload
   enum). Removes the flattening from four ports.
2. HG4: read typed edge records, `Graph.undirected(records)`, labels that
   print.
3. HG5: `merge_tree` / `kruskal_tree`, `Rooted.climb`, and a per-tree
   `Pre`.
4. HG6: `std.gen` graph shapes and an answer histogram in `cpl stress`.
5. HG3: one heap with a key, `desc` in the key only.
6. HG9.5, HG9.6 and HG2: `min=` that reports improvement; `dag_dp`;
   `topo` that infers its vertex type.
7. HG8: `List.pop -> ?T`, `join`, format specs, `all`/`any`.
