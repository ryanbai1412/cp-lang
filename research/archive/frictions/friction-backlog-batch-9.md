# Friction backlog, batch 9: tree decompositions and graph structure

A Stage-1 scout pass over ten submissions: heavy-light paths, a centroid,
rerooting, Euler labels, LCA, the virtual-tree problem, 2-SAT and bridges.
The ports are in
[`proto/ports/scout_batch_9/`](../proto/ports/scout_batch_9/): noi21p1,
utso15p5, btoi19p3, ioi19p2, coci25c3p4, ioi10p6, ioi20p6, sdoi17r2p4,
cmt21r2ztr and noip22p3. This file follows the format of
[`friction-backlog-batch-4.md`](friction-backlog-batch-4.md), and its
items are numbered `T1`... so they do not collide with earlier batches.

What was checked: the ports were written against commit `722c0ea` and
then rebased onto `4299a09`, which brought in the fixes for scout batch 1
(`decisions.md` section 13). Four of this batch's complaints were already
answered there, so the ports were changed to use the answers and those
items now say what is left. All ten ports build with `proto/cpc` at
`4299a09` and agree with their originals on 200 generated cases each
(`proto/ports/scout_batch_9/stress.sh`; ioi20p6 and ioi19p2 lose 117 and
41 of the 200 to their `where` clauses). So every "today" snippet below is
the line a port really contains, or one it contained before it was
rewritten. Every error message quoted was produced by `proto/cp.py build`
at `4299a09`. The "wanted" snippets do not build. A count is the number
of the ten ports that contain the awkward line; other ports are named
separately, and the wider corpus was not counted. Largest-size inputs were
not timed.

## Suggested order for a Stage-2 sprint

1. T12: `first_where` gave a wrong answer without a message. It is the
   only item here that is a trap and not a missing convenience.
2. T1 and T3: 3 of 10. `Hld` with `path` and `path_edges` is already
   written in `stdlib/tree.cp`; the prototype compiler only offers the raw
   ranges, and every port that touches a path pays for it.
3. T16 and T17: 9 of 10 and 5 of 10. They are test scaffolding, not
   language, but they are most of the lines that are not the solution.
4. T6, T7, T10, T11: one library piece each (bridge blocks, `TwoSat`, a
   tour, a centroid); 1 of 10 each, and each replaces 3 to 12 lines.
5. T8 and T9: 2 of 10 and 1 of 10. A domain made of named parts; this is
   the only item here that asks for something in the language.
6. T18: small typing and parser gaps, 7 of 10 between them.
7. T4, T13, T15: what is left after main's fixes; 1 of 10 each.
8. T2, T5, T14: 1 of 10 each.

## T1. A path is three ranges, a guard and a fold (3 of 10)

btoi19p3, utso15p5 and noi21p1. `stdlib/tree.cp` has `Hld<V, T>` with
`path(a, b)`, `path_edges(a, b)`, `sub(v)` and `set(v, x)`, and a
`chains(a, b, skip_top)`. The prototype compiler has none of them: a tree
has `tree.chains(a, b)`, a list of `(l, r)` positions, and the port brings
its own `SegTree<tree.Pre, T>`. *Checked:* `tree.chains(a, b, true)` gives
`nodes of this tree are V, got Bool`.

That is fine for a value on every vertex (btoi19p3, one line). For a value
on every edge, stored at the edge's lower end, the highest vertex of the
path has to be left out by hand. utso15p5:

```cp
let peak = tree.sub(tree.lca(u, v)).l
var path: Top = (0, 0)
for (l, r) in tree.chains(u, v):
    if l != peak: path = both(path, above.query(l, r))
    elif let below = l + 1 and l != r: path = both(path, above.query(below, r))
```

Wanted, with the library that already exists:

```cp
let above = Hld(tree, top_two, [Town] v => (cost_above[v], 0))
let path = above.path_edges(u, v)
```

and btoi19p3's `min(turn.query(l, r) for (l, r) in tree.chains(bottom, x))`
becomes `turn.path(bottom, x)`. This removes 5 lines from utso15p5 and
the `SegTree` setup loop from btoi19p3. noi21p1's `peak` line is the same
workaround; what it needs is T2.

## T2. Assign along a path, fold in order (1 of 10)

noi21p1. The original is about 100 lines of lazy segment tree whose node is
"left colour, right colour, equal neighbours inside", and a reversed copy
for the climbing half of a path. The prototype has `AssignSum` and
`AddMax` and no lazy tree over a monoid of the port's own, so the port was
restructured to avoid needing one: two `AssignSum`s, one of stamps and one
of "the edge to the cell before is heavy" bits, and a comparison of stamps
where two ranges of a path meet.

```cp
for (l, r) in tree.chains(a, b):
    stamp.assign(l, r, now)
    tied.assign(l, r, 1)
    tied.assign(l, l, 0)
    if let after = r + 1: tied.assign(after, after, 0)
```

It is about 15 lines and correct, but it took the most thought of the ten, and
the two off-by-one cells (`l` and `r + 1`) are exactly where it is easy to
be wrong. Wanted: a lazy `Hld` whose update is "assign" and whose fold is
the port's own monoid, so the problem statement can be said directly:

```cp
type Run = (first: Int, last: Int, same: Int)       // equal neighbours inside
var colour = Hld(tree, runs, [V] v => fresh(v), assign: true)
colour.assign_path(a, b, now)
print(colour.path(a, b).same)
```

Simplifies 1 of 10 here. Path assignment with an ordered fold is a
standard heavy-light exercise, so the wider corpus has more.

## T3. Positions in the layout (3 of 10)

btoi19p3, utso15p5 and noi21p1. Four small gaps
around `tree.Pre`:

*A vertex's position.* `tree.pos(v)` is an `Int`, so it cannot index a
structure over `tree.Pre`. *Checked:* `seg.query(tree.pos(v), tree.pos(v))`
gives `a SegTree<Pre_V, Int> is indexed by Pre_V, got Int[0..=10]`. All
three ports write `tree.sub(v).l` instead (6 uses):

```cp
for v in Village:
    if let d = shop_below[v]: turn.set(tree.sub(v).l, d - far[v])
```

`SegTree.of(min, f)` would state this once, but `f` is given a position
and has no way to ask which vertex is there.

*The vertex at a position.* noi21p1 keeps its own inverse:

```cp
var who: [tree.Pre]V = V.first
for v in V: who[tree.sub(v).l] = v
```

*Checked:* `tree.at(l)` gives `nodes of this tree are V, got Pre_V`, which
reads as if `at` existed.

*A range that may be empty.* "Everything in this range after its first
cell" needs a guard, because `l + 1` is optional and `(l + 1, r)` is not a
range when `l == r`. *Checked:* `seg.query(l + 1, r)` gives `indexed by
Pre_V, got ?Pre_V`. utso15p5 and noi21p1 (3 uses):

```cp
if let second = l + 1 and l != r: heavy += tied.sum(second, r)
```

*One cell.* `AssignSum` has `assign(l, r, x)` and `sum(l, r)` only, so
noi21p1 reads a cell with `stamp.sum(l, l)` and writes one with
`tied.assign(l, l, 0)`. *Checked:* `st.get(l)` gives `no method get on
AssignSum<Pre_V> (assign(l, r, x), sum(l, r))`.

Wanted: `tree.pos(v)` typed as `tree.Pre` (with `.idx` for the number, as
noip22p3's low-link wants), `tree.at(p)`, structures that take a range
value so that `tied.sum(l.next..=r)` is 0 when empty, and `get`/`set` on
every range structure. T1 makes most of these unnecessary in ports, but
the library itself is written with them.

## T4. A rooted tree keeps lengths, not edges (1 of 10; batch 4 B14)

Mostly answered on main. At `722c0ea`, btoi19p3 had to strip the lengths
to root the tree and then scan for each child to get them back, six lines.
Now a tree is rooted straight from the edge records, and `dist` adds the
lengths:

```cp
let tree = Rooted(roads, out, weight: .len)
let far: [Village]Int = [Village] v => tree.dist(out, v)       // the way from v to the exit
```

What is left:

- The distance from the root is a field of the library's `Rooted` (`far`),
  and the prototype does not offer it. *Checked:* `tree.far(v)` gives `no
  method far on Rooted<V> (lca, dist, is_anc, parent, ancestor, step,
  children, size, pos, depth, .kids, .root, .preorder)`. So the port asks
  for `tree.dist(out, v)`, an LCA query per vertex, for a number the tree
  already has.
- Anything else on an edge is still lost. utso15p5 stores each kept road's
  cost at its lower town and has to work out which end that is:

  ```cp
  above.set(tree.sub(if tree.parent(u) == v: u else: v).l, (cost, 0))
  ```

- `weight:` is only read from a list of edges. *Checked:* rooting the
  adjacency lists of a graph with lengths gives `Rooted needs adjacency
  lists ([V]List<V>, or arcs with a `to: V`) or a list of edges (records
  that start with two V), got [V]List<(f0: V, f1: Int[1..=9])>`.

Wanted: `tree.far(v)`, and `tree.above(v)` for the record of the edge to
the parent (`none` at the root), so that utso15p5's line is
`[Town] v => tree.above(v)?.cost`.

## T5. Rerooting is written out again (1 of 10 writes it, 1 avoids it)

coci25c3p4 writes the two passes by hand: `down[v]` from the children,
then `around[v]` from the children and "everything around the parent
except v":

```cp
for v in tree.preorder:
    for c in tree.children(v): around[v] = times(around[v], down[c])
    if let p = tree.parent(v): around[v] = times(around[v], hung(without(around[p], down[v])))
```

Here the product has an inverse (`without`, polynomial division), so the
second pass is three lines. Without one it is the prefix and suffix
products that `ergonomics-audit.md` already lists. ioi10p6's original
reroots too; the port did not need to, because "everyone outside the
subtree" is `below[City.first] - below[c]`.

Wanted, as in the audit, one userland combinator; this port adds the
request that it accept an inverse when there is one:

```cp
let around = tree.reroot(one: [Z(1)], merge: times, lift: hung, undo: without)
```

## T6. Bridges and the blocks between them (1 of 10; coi06p2 earlier)

noip22p3. This is the second port to write low-link by hand
(`hillclimb_dp_graph/coi06p2.cp` is the first), and then to name each
block by its topmost city because there is no domain for blocks:

```cp
var low: [City]Int = [City] v => tree.pos(v)
for v in tree.preorder.rev:
    for w in land[v]:
        if tree.parent(v) != w: low[v] min= tree.pos(w)
    if let p = tree.parent(v): low[p] min= low[v]

var top: [City]City = [City] v => v
var cities: [City]Int = 0
for v in tree.preorder:
    if let p = tree.parent(v) and low[v] < tree.pos(v): top[v] = top[p]
    cities[top[v]] += 1
...
for b in tree.preorder.rev:
    if top[b] != b: continue
```

Two things to note. The low-link pass is right because `Rooted` over a
graph that is not a tree is the depth-first spanning tree, so every other
edge joins a vertex to an ancestor. `decisions.md` section 12 promises
that for the prototype; it also records that `stdlib/tree.cp`'s
`Rooted.new` still searches breadth-first, where these five lines would be
wrong without a message. And `tree.parent(v) != w` is wrong when the input
has two roads between the same pair; the generator can produce them, and
the port agrees with the original only because the original makes the
same assumption.

Wanted: the same shape as `scc`, a fresh domain and a map into it, and a
tree over that domain:

```cp
let (Block, of) = bridge_blocks(land)       // also: land.bridges
let blocks = Rooted(condense(land, of), of[City.first])
let cities = count_by(of)                   // [Block]Int
for b in blocks.preorder.rev: ...
```

This removes about ten lines from noip22p3 and the `if top[b] != b:
continue` filter, and gives the DP a real tree to run `@memo` over. Articulation
points want the same treatment (`cut_blocks`, the block-cut tree); no port
in this batch needed them.

## T7. 2-SAT (1 of 10)

cmt21r2ztr. There is `scc`, and nothing above it, so the port numbers its
own literals and reads the answer off the component order:

```cp
domain Claim = 0..4 * Mark.len
fn below(q: Mark) -> Claim: Claim(4 * q.idx)
fn above(q: Mark) -> Claim: Claim(4 * q.idx + 2)
fn no(c: Claim) -> Claim: Claim(c.idx ^ 1)
var arcs: List<(u: Claim, v: Claim)> = []
fn either(a: Claim, b: Claim):
    arcs.push((no(a), b))
    arcs.push((no(b), a))
...
let (Knot, of) = scc(Graph.directed(arcs))
fn holds(c: Claim) -> Bool: of[c] > of[no(c)]
if any(of[below(q)] == of[no(below(q))] or of[above(q)] == of[no(above(q))] for q in Mark):
```

`of[c] > of[no(c)]` depends on the order in which `scc` numbers its
components. `stdlib/graph.cp` documents it, but it is one more thing to
get the right way round.

Wanted: a userland `TwoSat<D>` over any domain of variables, with literals
`x` and `not x`:

```cp
var sat = TwoSat<Fact>()            // Fact: (mark, Below | Above), see T8
sat.either(not below(a), not below(b))
if let holds = sat.solve():
    ...holds[below(q)]...
else: print("NO")
```

This removes about ten lines and three of the four `.idx` and `.label`
uses in cmt21r2ztr.

## T8. A graph whose nodes are several kinds of thing (2 of 10)

cmt21r2ztr and sdoi17r2p4. Both build a helper graph whose nodes are a
mix: "below x" and "above x" claims and their denials; or roads, four
chains of links per town, and a start. A domain is one range, so both pack
the kinds into it with arithmetic. sdoi17r2p4:

```cp
domain Node = 0..9 * m + 1
fn taken(r: Road) -> Node: Node(r.idx)
fn link(slot: Int, chain: 0..=3) -> Node: Node(m + 4 * slot + chain)
let start = Node(9 * m)
```

and a running `slot` counter, because a town's links are a slice of one
flat numbering. Of the 30 `.idx` and `.label` uses in the batch, 25 are in
test generators (T16); the other 5 are these packings and T9's. A wrong
multiplier here is an out-of-range node at best and a wrong answer at
worst.

Wanted: a domain declared as a sum of named parts, each over its own
domains, laid out by the compiler:

```cp
domain Node = Taken(Road) | Link(Slot, 0..=3) | Start
arcs.push((Link(s, 2), Taken(r), roads[r].cost))
```

`[Node]T`, `Graph` and `dijkstra` work unchanged because it is still a
dense domain. This is the one item here that is a language change, and it
is the general form of `Claim` in T7. It fits the unified bounded-index
mechanism in `decisions.md`: a product of domains already exists for
tables; this is the sum.

## T9. Compressing pairs (1 of 10)

cmt21r2ztr needs a dense name for every (slot, value) pair that occurs.
`compress` takes numbers, so the pair is packed into one and unpacked
again:

```cp
const SPAN = 1073741824          // 2^30, above every value
fn key(s: Slot, x: Int) -> Int: s.label * SPAN + x
let (Mark, mark) = compress(keys)
fn at(s: Slot, x: Int) -> Mark: mark.first_ge(key(s, x))!
fn slot(q: Mark) -> Slot: Slot(mark[q] / SPAN)
fn value(q: Mark) -> Int: mark[q] % SPAN
```

Wanted: `compress` over any ordered value, and a lookup that says what it
is:

```cp
let (Mark, mark) = compress((s, f.x) for f in facts for s in [f.i, f.j, f.k])
let q = Mark.of((f.i, f.x))         // the mark of a value that is known to be there
let (s, x) = mark[q]
```

This removes about 5 lines. `Mark.of` would also replace `first_ge(...)!`
wherever a port compresses and then looks a value up.

## T10. Entering and leaving (1 of 10)

ioi20p6 numbers a vertex on the way in at even depth and on the way out at
odd depth. `tree.preorder` only has the way in. *Checked:* `tree.tour`
gives ``Rooted<V> has no `.tour` ``. So the port recurses, which the
library otherwise avoids:

```cp
fn walk(s: Station):
    if tree.depth(s) % 2 == 0: number(s)
    for c in tree.children(s): walk(c)
    if tree.depth(s) % 2 == 1: number(s)
walk(Station.first)
```

Wanted: `for (v, entering) in tree.tour`, built from the same arrays as
`preorder` (leaving `v` happens at position `sub(v).r`), so nothing
recurses:

```cp
for (s, entering) in tree.tour:
    if entering == (tree.depth(s) % 2 == 0): number(s)
```

1 of 10 here. Euler-tour problems with an event on both sides (bracket
sequences, "active on the path to the root") are common in the corpus.

## T11. The centroid (1 of 10)

ioi19p2. *Checked:* `tree.centroid()` gives `no method centroid on
Rooted<V> (lca, dist, is_anc, parent, ancestor, step, children, size, pos,
depth, .kids, .root, .preorder)`. (That list leaves out `sub` and `chains`, which do
exist.) The port:

```cp
var hub = Spot.first
for v in Spot:
    if 2 * (n - tree.size(v)) <= n and all(2 * tree.size(c) <= n for c in tree.children(v)): hub = v
```

Wanted: `tree.centroid`, and for the decomposition problems this batch did
not reach, `tree.centroids` as a `Rooted` over the same vertices (the
centroid tree), so that "every centroid ancestor of v" is
`climb(cd, v)`. No accepted centroid-decomposition submission that was not
already ported fitted the batch, so that half is not measured here.

## T12. `first_where` is a binary search (wrong answer; 2 of 10)

ioi19p2 and ioi10p6. ioi19p2 wants any spot whose group is big enough:

```cp
var seed: ?Spot = none
for v in Spot:
    if v != hub and group.size(v) >= small: seed = v
```

Main added `first_where` on any range, so the port became

```cp
let seed = Spot.first_where(v => v != hub and group.size(v) >= small)
```

It built, and printed "no split" for a map that has one (7 spots, links
0-1 0-2 0-3 1-4 0-5 0-6, sizes 3 and 2). `first_where` is a binary search:
it assumes the test is false and then true along the range, and nothing
says so at the call or when the assumption does not hold. The name reads
as "the first one where", which is the linear search this port needs.
The port went back to the loop. *Checked:* `first(v for v in V if ...)`
gives `unknown name first`.

ioi10p6 wants the city with the smallest value and writes
`min((busiest(c), c) for c in City).1`, which works, but says "the
smallest pair" where it means "the city where `busiest` is smallest".

ioi19p2 also grows a connected patch of a given size inside one side of a
split, which is a breadth-first search with a filter and a quota, written
with an index into its own queue:

```cp
var patch = [from]
part[from] = mark
var i = 0
while i < patch.len:
    for w in land[patch[i]]:
        if patch.len < count and part[w] == 0 and group.same(w, side) == group.same(from, side):
            part[w] = mark
            patch.push(w)
    i += 1
```

`std.graph`'s `bfs` returns distances for the whole graph, so it does not
replace this.

Wanted: a linear `first(v for v in Spot if ...)` returning an optional;
the binary search under a name that says it (`partition_point` is already
there for the other direction), and in debug builds a check that the test
really is monotone on the values it tried; `argmin(City, busiest)`; and a
lazy `land.bfs(from, through: w => ...)` so that the patch is
`land.bfs(from, through: ok).take(count)`.

## T13. No way to stop early at the top level (1 of 10)

utso15p5. Kruskal itself is fine since main: with `.cost` and a tree
rooted from the kept edges it is four lines, down from eight at `722c0ea`.

```cp
var joined = Dsu<Town>()
var kept: List<(u: Town, v: Town, cost: Int)> = []
for (u, v, cost) in roads.sorted_by(.cost):
    if joined.unite(u, v): kept.push((u, v, cost))
```

But the rest of the program is nested under `if kept.len < n - 1:
print(-1) else:`, because `Rooted` stops on a disconnected graph and there
is no early exit at the top level of a program. *Checked:* `return` there
gives `return outside a function`, and `exit()` gives `unknown name exit`.

Wanted: `return` (or `stop`) at the top level, so "print -1 and finish" is
two lines and the main path is not indented. A
`spanning_forest(roads, by: .cost)` in the library would make the four
lines one; it is a smaller gain.

## T14. Polynomials are lists (1 of 10)

coci25c3p4 multiplies and divides small polynomials (a knapsack over
subtree sizes), 10 lines:

```cp
fn times(a: List<Z>, b: List<Z>) -> List<Z>:
    var c = [Z(0) for _ in 0..a.len + b.len - 1]
    for i in 0..a.len:
        for j in 0..b.len: c[i + j] += a[i] * b[j]
    c
```

and "add one to the coefficient of x" is three lines because the list may
be too short: `if f.len == 1: f.push(Z(0))` then `f[1] += 1`.

Wanted: a userland `Poly<T>` with `*`, exact `/`, `+` and a coefficient
access that grows, so that `hung(inside)` is `inside + x`. The naive
O(n*m) product is what a tree knapsack needs, so it can come before any
FFT does.

## T15. "No value" in a fold (1 of 10)

Mostly answered on main. At `722c0ea` btoi19p3 had `const INF =
1000000000000000000`, a table and a `SegTree` that started from it, and
`elif shop_below[bottom] == INF`. With `min` skipping `none` the constant
is gone:

```cp
var shop_below: [Village]?Int = none
...
var turn = SegTree<tree.Pre, ?Int>(none, (a, b) => min(a, b))
...
elif let best = min(turn.query(l, r) for (l, r) in tree.chains(bottom, x)):
    print(far[x] + best)
else:
    print("oo")
```

What is left: sdoi17r2p4 gets `[Node]?Int` back from `dijkstra`, folds it
under `if let`, and ends with `print(best[v]!)` for towns the input
promises are reachable. A `read` precondition cannot say "every town is
reachable", so the `!` is the only place that promise is written.

## T16. Every port writes its own random tree (9 of 10)

All but cmt21r2ztr. Six ports have a `random_tree` that
differs only in the names of the domains and whether labels start at 0:

```cp
fn random_tree() -> [E](u: V, v: V):
    [E] e => (V(rand(1, e.label)), V(e.label + 1))
```

Four (noip22p3, ioi19p2, sdoi17r2p4, utso15p5) have "a random tree, then
extra edges" so the graph is connected. Where an edge carries data the generator
restates the bounds that the type already gives:
`rand(1, 1000000000)` next to `len: 1..=1e9`. These are 25 of the 30
`.idx` and `.label` uses in the batch. `stdlib/tree.cp` has a random tree as
adjacency lists; it does not fit `read(gen:)` of an edge list.

Wanted: generators that are chosen by shape and fill the rest from the
types:

```cp
let edges: [E](u: V, v: V, len: 1..=1e9) = read(gen: tree)          // or connected, dag, any
```

Three ports needed more than a tree. ioi19p2 only produced "no split
exists" after the tree was made bushy and most extra links were sent to
one spot, and then in 1 or 2 cases of 250. utso15p5 with no generator
printed -1 in 188 of 200 cases, because `m` is drawn without regard to
`n` and the map is rarely connected; with one, and at size 40, 80 of 200
cases have an answer. ioi20p6's `where t != s` rejects 117 of 200 cases
at small sizes. A `read(gen: distinct)` for "two different vertices", and
a stress report that counts how often each output branch was reached,
would have shown all three at once.

## T17. Graders and checkers (5 of 10)

*Grader-style originals (3 of 10).* ioi10p6, ioi20p6 and ioi19p2 define a
function and have no `main`, so `cp.py stress` cannot run them. Each got a
`harness/<problem>.cpp` that includes the original and reads the port's
input. ioi20p6 is worse than that: the real problem is two programs that
may not share memory, and the port can only say so in a comment.

*More than one right answer (2 of 10).* cmt21r2ztr and ioi19p2.
`cp.py stress` compares outputs word for word, so both needed a checker in
C++ (27 and 34 lines) and a second runner, `harness/judge.py`.

Wanted: the co-located `checker` block that is already approved, so the
check is written in the language next to the solution and sees the typed
input:

```cp
checker (out: [Spot]0..=3, ref):
    if all(x == 0 for x in out): return all(x == 0 for x in ref)
    ...
```

and for graders, `cpl stress --entry find_split` or an `export fn`, so a
port can be the function the judge calls.

## T18. Smaller typing and parser gaps (7 of 10)

Each of these cost one rewrite. All messages are from `cp.py build`.

- utso15p5 and btoi19p3: `SegTree<tree.Pre, Top>((0, 0), both)` and
  `SegTree<tree.Pre, ?Int>(none, min)` give `SegTree<D, T>(identity, (a, b)
  => combined)`. `SegTree.of(min, f)` does take an operator or a named
  function; the constructor only takes a lambda, so the ports write
  `(a, b) => both(a, b)` and `(a, b) => min(a, b)`.
- cmt21r2ztr: `const SPAN = 1 << 30` and then `mark[q] / SPAN` gives
  `` `SPAN` may be 0 in `mark[q] / SPAN` ``; a shift of constants is not
  folded. The port
  writes `1073741824`.
- cmt21r2ztr: `[key(s, f.x) for f in facts for s in [f.i, f.j, f.k]]`
  gives `unknown name f`: the second `for` cannot see the first.
- cmt21r2ztr and ioi19p2: `let [p, q, r] = ...` gives `expected NAME (at
  '[')`, and `let (a, b, c) = sorted([3, 1, 2])` gives `cannot destructure
  List<Int> into 3 names`. ioi19p2 takes `by_size[0]`, `[1]`, `[2]` apart
  on three lines.
- ioi20p6: a `List` has no `first_ge`, `last_le` or `.last` (`List<Int>
  has no method `first_ge``, ``List<Int> has no `.last` ``); arrays have
  the first two. The port loops and writes `near[near.len - 1]`.
- noip22p3: `two.pow(k)` gives ``Mod<1000000007> has no method `pow` ``.
  The operator is `**`; the message could say so.
- sdoi17r2p4: `for v in Town if v != Town.first:` gives `expected ':' (at
  'if')`. A comprehension takes a filter and a loop does not.

## Already good

Things these ports needed that built on the first try:

- A whole test case inside `for _ in 0..t:`, with its own `domain`s,
  generator functions, `Rooted` and structures over `tree.Pre` (noi21p1,
  sdoi17r2p4).
- `tree.preorder.rev` with `if let p = tree.parent(v)` for every
  bottom-up pass: 4 of 10 use it and none recurses for it.
- `tree.is_anc`, `tree.lca`, `tree.depth`, `tree.size`, `tree.children`.
  btoi19p3's "is x cut off by this road" is `tree.is_anc(bottom, x)`.
- `let (Knot, of) = scc(Graph.directed(arcs))` and
  `dijkstra(Graph.directed(arcs), start)` over a domain the port made up.
- `Dsu<Spot>` with `unite` returning whether it joined, and `size`.
- New on main and used here: `Rooted(roads, root, weight: .len)` straight
  from the edge records (6 of 10 no longer build a `Graph` first),
  `roads.sorted_by(.cost)`, and a `SegTree` over `?Int` with `none` as its
  identity.
- `Mod<P>` with `/` and `**`; noip22p3's counting has no modular
  arithmetic written out.
- `x min= y` and `x max= y`, including on `?Int`.
- `read() where ...` for input promises, and printing a whole array.

Two solutions changed shape for the better because the tree vocabulary
was there. sdoi17r2p4's original builds a virtual tree per town, over an
Euler tour and a sparse table; sorting each town's roads by `words.pos` and
taking `words.depth(words.lca(...))` of neighbours does the same job, so
the port has no virtual tree at all. ioi10p6's original reroots; the port
is two passes over `below`.
