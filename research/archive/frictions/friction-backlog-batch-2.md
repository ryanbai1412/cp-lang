# Friction backlog, batch 2: graphs, trees and range queries

A Stage-1 scout pass over ten hard graph, tree and range-query
submissions: apio09p3, ioi11p4, cco19p5, cco20p2, ccc17s4, noip18p3, dpv,
ioi08p2, noi15p2 and ioi14p2. The ports are in
[`proto/ports/scout_batch_2/`](../proto/ports/scout_batch_2/). Nothing in
`crates/` or `proto/cpc/` was changed. This file continues
[`friction-backlog.md`](friction-backlog.md) (batch 1); an item that batch
1 or [`friction.md`](friction.md) already has is listed at the end as a
recurrence, with the new count, and not written up again.

What is different from batch 1: these ports use `stdlib/` and run. They
are written for the reference interpreter (`proto/cpi/`), and
`proto/ports/scout_batch_2/check.py` compares each with its original on
random inputs. So this batch is mostly about the standard library: what a
port reaches for and does not find.

How to read an item. "Today" is copied from a port, so it runs on the
interpreter and gives the right answers. "Wanted" does not run anywhere
yet. The first count is exact: the ports of this batch that have the
line. The corpus count is a regular-expression search over the 710
problems with an accepted C++ submission (the latest one of each), named
in the item, so it is an estimate of how often the pattern is written and
not a list of problems that were read. One thing the interpreter cannot
tell us: it checks no types and no borrows, so where an item says the
real checker would reject a line, that is from reading
`plan/19-index-and-lends.md`, not from a compiler message.

## Suggested order for a Stage-2 sprint

1. B1 (ordered bag) and B2 (tree walks): the two that cost the most
   lines, 24 of the 51 in noip18p3 and three ports out of ten.
2. B3, B4, B5: three small library pieces for range updates and heaps,
   two ports each.
3. B6, B7: the two lines in this batch that run today only because the
   interpreter does not check them.
4. B8 to B12: one port each, one library function each.
5. The interpreter gaps, which decide whether later scout batches can
   run their ports at all.

## B1. No ordered bag (noip18p3, cco19p5; batch 1 F13)

noip18p3 pairs chains at every junction: take the shortest, give it the
shortest partner that completes a track, and remove both. The original
does it with `multiset::lower_bound` and `erase` in eleven lines. `stdlib/`
has no ordered set or map (its README says so), so the port sorts a list
and keeps a "next unused" pointer with path compression, and a binary
search written out by hand. Today:

```cp
fn pair_up(chains: List<Int>, k: Int) -> (tracks: Int, spare: Int):
    let a = sorted(chains)
    var skip = [i for i in 0..=a.len]       // skip[i]: the first unused chain at or after i
    fn unused(i: Int) -> Int:
        var j = i
        while skip[j] != j:
            skip[j] = skip[skip[j]]
            j = skip[j]
        j
    var (tracks, spare) = (0, 0)
    for i in 0..a.len:
        if unused(i) != i: continue
        skip[i] = i + 1
        var (lo, hi) = (i + 1, a.len)       // the first chain after i that is at least k - a[i]
        while lo < hi:
            let mid = (lo + hi) / 2
            if a[mid] >= k - a[i]: hi = mid else: lo = mid + 1
        let j = unused(lo)
        if j < a.len:
            skip[j] = j + 1
            tracks += 1
        else:
            spare = a[i]
    (tracks: tracks, spare: spare)
```

That is 24 of the port's 51 lines, and the reason it is only 16% shorter
than the library-stripped original. cco19p5 needs the smallest element of
a set and writes `min(above[v])` after an emptiness test, which is a scan
unless `Set` promises an order. Wanted:

```cp
fn pair_up(mut chains: SortedBag<Int>, k: Int) -> (tracks: Int, spare: Int):
    var (tracks, spare) = (0, 0)
    while let short = chains.pop_first():
        if chains.take_first_ge(k - short) != none: tracks += 1 else: spare = short
    (tracks: tracks, spare: spare)

if let f = above[v].first: ...              // cco19p5
```

Proposal: `SortedBag<T>` and `SortedSet<T>` in `stdlib/`, ordinary library
code over a balanced tree or a sorted-blocks list, taking an `Order<T>`
like `Heap` does. The operations the corpus uses: `insert`, `remove`,
`first`, `last`, `pop_first`, `pop_last`, `first_ge(x)`, `last_le(x)`,
the `take_` forms that also remove, and `rank(x)` (batch 1 F13 asked for
`rank`). Simplifies 2 ports here. In the corpus, 48 problems use
`multiset`, a member `lower_bound`/`upper_bound`, or an order-statistics
tree.

## B2. A rooted tree gives no way to walk it (dpv, noip18p3, noi15p2; batch 1 F16)

`Rooted<V>` has `lca`, `dist`, `ancestor`, `sub` and `chains`, and nothing
for the commonest use of a rooted tree: a DP that goes over the vertices
children first, or parents first. It has no public parent, children or
depth either, while the `Rooted` builtin of `proto/cpc` has `parent` and
`preorder`, so the two disagree. The ports reach into the library
struct's fields, which the interpreter allows because it does not enforce
`pub`. Today:

```cp
let top_down = [tree.at[p] for p in Rooted<V>.Pre]              // dpv
for v in top_down.rev:
    for c in tree.kids[v]: below[v] = below[v] * (1 + below[c]) % m

let bottom_up = [tree.at[p] for p in Rooted<Junction>.Pre].rev  // noip18p3
if let p = tree.up[v]: ...
```

The weight of the road to the parent is lost when the tree is built,
because `Rooted.new` keeps the arc's `to` and drops the rest. noip18p3
finds it again by testing both ends of every road:

```cp
var up_len: [Junction]Int = 0               // the road from v to its parent
for (u, v, len) in roads:
    if tree.up[u] == v: up_len[u] = len else: up_len[v] = len
```

And a tree given as a parent per vertex, as in noi15p2, has to be turned
into arc lists first, only so that `Rooted.new` can turn it back:

```cp
var needed_by: [Pkg]List<(to: Pkg)> = []
for d in Dep: needed_by[needs[d]].push((to: Pkg(d.label)))
let tree = Rooted<Pkg>(u => needed_by[u], Pkg.first)
```

Wanted:

```cp
for v in tree.bottom_up:
    for c in tree.children(v): below[v] = below[v] * (1 + below[c]) % m

let up_len = [Junction] v => if let a = tree.up_arc(v): roads[a.e].len else: 0
let tree = Rooted<Pkg>.from_parents(needs)
```

Proposal, all library code in `stdlib/tree.cp`: public `parent(v)`,
`children(v)`, `depth(v)`, `size(v)`; the orders `top_down` and
`bottom_up` as generators over `at` (no list is built); `Rooted<V, A>`
keeping the arc that reached each vertex, with `up_arc(v) -> ?A`, which
is also what batch 1 F16 (distances from the root) needs; and
`from_parents`. Simplifies 3 ports here, 4 with ioi11p2 from batch 1. In
the corpus, 52 problems have a DFS that takes its parent as an argument
or erases the parent from the adjacency list first.

## B3. A lazy tree needs a monoid even when only the leaves are read (ioi14p2)

ioi14p2 applies clamps to ranges and reads every column once at the end.
There is no range query, so there is nothing to fold, but `LazySegTree`
wants a `Monoid<T>` and an `Action<T, F>` whose `apply` works on folded
values. The port hands it `Max<Int>`, which is never used for an answer
and is only safe because a clamp happens to commute with `max`. Today:

```cp
let clamps = Action<Int, Clamp>((lo: 0, hi: Int.MAX), (f, g) => (lo: clamp(f, g.lo), hi: clamp(f, g.hi)), (f, h) => clamp(f, h))
var wall = LazySegTree<Col, Int, Clamp>.from(Max<Int>, clamps, [Col] c => 0)
```

Wanted:

```cp
var wall = RangeApply<Col, Clamp>(then)     // then: a Monoid<Clamp>, "g first, then f"
wall.apply(l..=r, raise(h))
for c in Col: print(clamp(wall.at(c), 0))
```

Proposal: `RangeApply<D, F>(m: Monoid<F>)` in `stdlib/lazy.cp`: range
`apply`, point `at`, one array where `LazySegTree` has two, and no
`Action` at all, because composing updates is a monoid on its own. Simplifies 1 port
here. In the corpus, 33 problems have a lazy tag; how many of them only
read points was not counted.

## B4. The common lazy updates are spelled out every time (noi15p2, ioi14p2)

`stdlib/lazy.cp` ships `AddMax`, `AddSum` and `SumLen`. "Set a whole
range" is not there, and writing it means an optional update, two nested
`if`s and a `!`. The three type arguments repeat what the two values
already say. Today, noi15p2:

```cp
let set_all = Action<(Int, Int), ?Bool>(none, (f, g) => f ?? g, (f, x) => if f == none: x else: (if f!: x.1 else: 0, x.1))
var installed = LazySegTree<Pre, (Int, Int), ?Bool>.from(SumLen, set_all, [Pre] p => (0, 1))
```

`Action` is also built by position, and `compose(f, g)` means "g first,
then f", which I had to look up in `algebra.cp` both times. Wanted:

```cp
var installed = LazySegTree<Pre>.from(SumLen, AssignSum, [Pre] p => (0, 1))
installed.apply(r, 1)                       // every position of r becomes 1
```

Proposal: (1) `AssignSum`, `AssignMax`, `AssignMin` and `Clamp` next to
`AddSum`, and a helper `assign(fill)` that builds the optional-update
action from "what a node of this size looks like when every element is
x"; (2) let the element and update types be inferred from the arguments,
so only the index type is written; (3) name the composition by what it
does, for example `Action(id, after: (f, g) => ..., apply: ...)`.
Simplifies 2 ports here; 33 problems in the corpus have a lazy tag.

## B5. A heap is built from an `Order` value (ioi11p4, cco20p2)

Today:

```cp
var heap = Heap<(Int, Room)>(Order<(Int, Room)>((a, b) => a.0 < b.0))   // ioi11p4
var free = Heap<Ex>(Greater<Ex>)                                        // cco20p2
```

The first line is the standard Dijkstra heap and it is the longest line
in its port. `stdlib/graph.cp` writes the same line. `proto/cpc` has a
builtin `MinHeap<(Int, V)>()` that the earlier port cco24p1 uses; the
library has no such name. Wanted:

```cp
var heap = MinHeap<(Int, Room)>()
var free = MaxHeap<Ex>()
var jobs = Heap.by::<Job>(j => j.due)
```

Proposal: `MinHeap<T>` and `MaxHeap<T>` as constructors in
`stdlib/heap.cp` for any `T: Ord` (tuples compare field by field), and
`Heap.by(key)`. Simplifies 2 ports here; 77 problems in the corpus use a
priority queue.

## B6. Swapping two slots of one array (cco19p5; batch 1 F17)

Small-to-large merging swaps two sets held in the same array. Today:

```cp
let f = min(above[v])
above[v].remove(f)
if above[f].len < above[v].len: swap(mut above[f], mut above[v])
for x in above[v]: above[f].add(x)
```

This runs on the interpreter. Under plan 19 §3.6 the `swap` line holds
two `mut` lends of `at` on one array, so `law disjoint(i, j) = i != j` has
to be proven, and nothing in the flow says `f != v`: it follows from "the
set of `v` only holds higher vertices", which is a fact about the data.
The real compiler would stop here and print the law, and again on the
last line, where the loop lends `above[v]` while its body writes
`above[f]`. The same four lines are in batch 1's ioi11p2 for maps. Wanted:

```cp
above[f].absorb(take(mut above[v]))         // the smaller one is poured into the larger
```

Proposal: `absorb(other)` on `Set`, `Map` (with a function for keys in
both), `List` and `SortedBag`: it takes `other` by value, swaps the
buffers when `other` is the larger, and inserts. With `take` (decision
5.5) there is one lend at a time, so no law is needed and there is no
`if`. And `a.swap(i, j)` on arrays, which is a no-op when `i == j`, as
the suggestion in the error for a `swap` whose law cannot be proven.
Simplifies 1 port here, 2 with ioi11p2. In the corpus, 6 problems have
the `if (sz(a) < sz(b)) swap(a, b)` line and 19 swap two elements of one
array by index.

## B7. Edge records with a weight (ioi11p4, noip18p3)

`Graph<V, E>.undirected` is declared over `[E](u: V, v: V)`. Both ports
pass edges that also carry a length, and get the length back through the
edge id. Today:

```cp
let halls: [Hall](u: Room, v: Room, len: Int) = read()
let g = Graph<Room, Hall>.undirected(halls)
for a in g.adj(u):
    if offers[a.to] < 2: heap.push((t + halls[a.e].len, a.to))
```

The interpreter accepts the second line because it does not check types.
Whether a record with an extra field may be passed where `(u: V, v: V)`
is expected has no row in `decisions.md` that I could find, and without
it a weighted graph cannot use `Graph` at all. The lookup `halls[a.e].len` is
correct, but it is three names for "the length of this arc". Wanted:

```cp
let g = Graph.undirected(halls)             // V and E come from the array's type
for (to, len) in g.arcs(u):                 // the other end, then the edge's own fields
    if offers[to] < 2: heap.push((t + len, to))
```

Proposal: decide the rule first. The smallest one that covers this:
a record type with more fields is accepted for a read-only parameter of
a record type with fewer (width subtyping, read-only), which also makes
"a function that only needs `u` and `v`" writable in user code. Then
`Graph<V, E, X>` keeps the edge array and `arcs(u)` yields `to` together
with the edge's other fields. Simplifies 2 ports here, 3 with ioi11p2.
In the corpus, 36 problems read edges as `u v w`.

## B8. Folding over everything but one (dpv)

Rerooting gives each child the product over its siblings. With no
inverse (the modulus is not prime), that is a prefix and a suffix
product, and it is the only raw-index loop in the port. Today:

```cp
var after = List.filled(kids.len + 1, 1 % m)
for i in (0..kids.len).rev: after[i] = after[i + 1] * (1 + below[kids[i]]) % m
var before = beyond[v]
for i in 0..kids.len:
    beyond[kids[i]] = (1 + before * after[i + 1]) % m
    before = before * (1 + below[kids[i]]) % m
```

Wanted:

```cp
let others = fold_others(times, [1 + below[c] for c in kids])   // others[i]: all but the i-th
for (c, rest) in zip(kids, others): beyond[c] = 1 + beyond[v] * rest
```

Proposal: `fold_others(m: Monoid<T>, xs) -> List<T>` in
`stdlib/algebra.cp`, in order, so it works for a monoid that does not
commute. On top of it, `reroot(tree, m, lift, finish)` in
`stdlib/tree.cp`, which would make dpv five lines. Simplifies 1 port
here. In the corpus, 5 problems keep a prefix and a suffix array side by
side or name a function `reroot`.

## B9. A modulus read from the input (dpv)

`Mod<P>` needs `P` at compile time. For a modulus from the input the
library has `ModR`, an operation value: `r.mul(a, b)`. That is longer
than `a * b % m`, so the port uses plain `Int`s and writes `% m` six
times, plus `1 % m` twice for the case `m == 1`. Today:

```cp
var below: [V]Int = 1 % m
for c in tree.kids[v]: below[v] = below[v] * (1 + below[c]) % m
for v in V: print(below[v] * beyond[v] % m)
```

Wanted:

```cp
type M = Mod<m>                             // m was read from the input
var below: [V]M = 1
for c in tree.children(v): below[v] *= 1 + below[c]
for v in V: print(below[v] * beyond[v])
```

Proposal: let the parameter of `Mod` be a frozen run-time `let`, the same
mechanism as `let K = n` for an index type (decision 3.8, plan 19 §3.1):
the value travels as one hidden integer. `Mod<P>` and `ModR` then become
one type, which revisits decision 3.10. Simplifies 1 port here; 3
problems in the corpus read their modulus, so this is about having one
way to write modular code more than about the count.

## B10. A Dijkstra that is not exactly `dijkstra` (ioi11p4)

`dijkstra` and `shortest` settle a vertex the first time it is popped.
ioi11p4 settles it the second time. Nothing of the library function can
be reused, so the port writes the heap loop itself. Today:

```cp
var offers: [Room]Int = 0                   // ways out seen so far, cheapest first
for x in exits:
    offers[x] = 1                           // an exit needs no second way out
    heap.push((0, x))
while let (t, u) = heap.pop():
    offers[u] += 1
    if offers[u] != 2: continue
    ...
    for a in g.adj(u):
        if offers[a.to] < 2: heap.push((t + halls[a.e].len, a.to))
```

Wanted:

```cp
let escape = kth_shortest::<Room>(u => g.arcs(u), [(x, 0) for x in exits], a => a.len, k: 2, sources_at: 1)
print(escape[Room.first]!)
```

Proposal: `kth_shortest(adj, from, w, k)` returning, for every vertex,
its `k`-th smallest arrival, which also covers "second shortest path" and
"k shortest walks". I am less sure of this one than of the others: the
exits settling on their first pop needs an extra argument, and the loop
above is short. The cheaper half is B5 and B7, which remove its two long
lines. Simplifies 1 port here; not counted in the corpus.

## B11. The queue loop (ioi08p2, and three functions in `stdlib/`)

A worklist that grows while it is walked. Today:

```cp
var leaves = [i for i in I if pending[i] == 0]
var next = 0
while next < leaves.len:
    let v = leaves[next]
    next += 1
    ...
    if pending[p] == 0: leaves.push(p)
```

`bfs`, `topo` and `Rooted.new` in `stdlib/` each write the same four
lines. A `for v in leaves` that pushes to `leaves` grows the
list the loop is walking, which the borrow rules reject (plan 19 §11,
"Loops lend what they walk"), so the index is the only spelling. Wanted:

```cp
var leaves = Queue.of(i for i in I if pending[i] == 0)
while let v = leaves.pop():
    ...
    if pending[p] == 0: leaves.push(p)
```

Proposal: `Queue<T>` in `stdlib/` (a list and a head index; `pop` takes
from the front, `push` adds at the back, `seen` gives everything ever
pushed in order, which is the BFS order `Rooted.new` keeps). Simplifies 1
port here and 3 library functions; 29 problems in the corpus use `queue`
or `deque`.

## B12. One outgoing edge per vertex (ioi08p2)

n vertices and n edges, one per vertex: every component is a cycle with
trees on it. The port peels the trees and then walks each cycle, about
fifteen lines before the problem's own logic starts. Today:

```cp
var pending: [I]Int = 0                     // unpeeled islands whose bridge leads here
for i in I: pending[bridge[i].to] += 1
...                                         // the queue loop of B11
for start in I:
    if pending[start] == 0 or seen[start]: continue
    var ring = List<I>()
    var v = start
    while not seen[v]:
        seen[v] = true
        ring.push(v)
        v = bridge[v].to
```

Wanted:

```cp
let shape = Functional<I>(i => bridge[i].to)
for v in shape.peel_order: ...              // tree vertices, leaves first
for ring in shape.rings: ...                // each cycle, in walking order
```

Proposal: `Functional<V>(next: V -> V)` in `stdlib/graph.cp` with
`peel_order`, `rings`, `on_ring(v)` and `ring_of(v)`. Permutation cycles
are the same structure. Simplifies 1 port here; 13 problems in the corpus
name a cycle variable or function.

## Seen again from earlier lists

These are not written up again; the count is the ports of this batch
that hit them.

- **An optional in a running maximum (2: apio09p3, ioi08p2; batch 1 F8
  and F9).** `after = max(after ?? b, b)` and
  `short = max(short ?? Int.MIN, reach[b] - at)`. With "`none` is the
  identity" for `max=` and `min=` on both sides, these are
  `after max= best[a.to]` and `short max= reach[b] - at`.
- **Binary search written out (1: noip18p3, twice; batch 1 F10).** Once
  over a sorted list and once over the answer. 46 problems in the corpus
  have a `while (l < r)` loop of this shape.
- **A sub-domain that is part of another (1: noi15p2; `friction.md` item
  2).** Packages `1..n` each name the package they need, so the port has
  `domain Dep = 1..n` and converts with `Pkg(d.label)`. Third port to hit
  it, after btoi17p6 and aac5p2.
- **Grader-style originals (1: ioi14p2; batch 1 F20, decision 6.8).**
  The port reads the judge's stdin format and `check.py` adds a `main` to
  the original.

## Gaps in the reference interpreter

These are not language friction. The language has the construct and
`proto/cpi/` does not, so a port that wants to run has to avoid it. They
matter for the process: a scout batch can only be checked against its
originals on the interpreter, because `proto/cpc` cannot `use std.x`
(batch 1 F1).

- **`enum` and `match`.** "`enum` statements are not supported by the
  reference interpreter". noi15p2 compares a word
  (`if cmd == "install"`) and ioi14p2 a number (`if op == 1`), where
  `proto/cpc` would read an `enum` with `@tags("install", "uninstall")`.
  34 problems in the corpus dispatch on a command word or code. A small
  wish for the language itself: the tag next to its variant
  (`Install(x: Pkg) = "install"`), not in a list above the `enum`.
- **`min=` and `max=` on an optional.** `best min= 4` on a `?Int` holding
  `none` stops with a Python `TypeError`, where `proto/cpc` treats `none`
  as "no value yet".
- **`[n]T` sized by an `Int`.** "`1` is not an index type". Every sized
  read needs a domain declared for it: `domain P = 0..p` then
  `let pubs: [P]Spot = read()` in apio09p3, and the same in ioi11p4.
- **`mut fn`.** Not parsed. A nested `fn` may write a captured `var`
  without it (`switch` in noi15p2, `unused` in noip18p3).
- **`any`, `all`, `count`.** Unknown names. ccc17s4 writes
  `sum(1 for p in Pipe if ...)` and a loop with `break`.
- **Top-level `return`.** Ends the run with a Python traceback. cco20p2
  wraps its body in `fn moves() -> ?Int`, which reads well anyway.
- **Filling an array with a set.** `var a: [V]Set<V> = Set<V>()` makes
  one set, not an array of them; the ports write `[V] v => Set<V>()`.

## Already good

- A fresh domain from `scc`, numbered in topological order, turned the
  DP of apio09p3 into `for c in C.rev` with no order to compute and no
  memo table. `[C]`-indexed arrays could not be mixed up with `[Spot]`
  ones.
- Boundaries in a Fenwick tree: `placed.prefix(e.l)` in cco20p2 is "the
  placed exercises left of `e`" with no `- 1` to think about.
- `Dsu<House>` with `unite` returning whether it merged, and `count`,
  made ccc17s4's two passes read like the explanation of the algorithm.
- `tree.sub(x)` and `tree.chains(a, b, false)` returning ranges of one
  domain let noi15p2 use the same four-line `switch` for a path and for a
  subtree.
- Sorting a domain by a tuple key:
  `sorted(Pipe, by: p => (pipes[p].cost, not is_old(p)))`.

## Solutions that changed shape

- **ioi14p2**: a sweep over columns with a segment tree over time became
  range clamps on a lazy tree over the columns, 12 lines.
- **ioi08p2**: a recursive cycle search and a memoised tree DP became a
  leaf-first peel and one pass round each cycle; no recursion on a
  million vertices.
- **dpv**, **noip18p3**: recursive DFS became loops over the preorder.
- **cco20p2**: the inversions are counted while the order is built, so
  the order itself is never stored.
