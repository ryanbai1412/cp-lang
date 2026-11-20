# Friction backlog, batch 26: hard trees

Ten tree problems were re-solved statement-first in
[`proto/ports/scout_batch_26/`](../proto/ports/scout_batch_26/README.md) and
checked against the original submissions on small random trees. This file
is what writing them was like. Items are numbered `TR1`..`TR17`, the same
numbers as the `WISH` comments in the ports. Nothing here has been built.

The short version: `Rooted<V>` already carries the hard parts (heavy-first
preorder, `sub`, `lca`, `chains`), and seven of the ten ports are under 50
lines because of it. The friction is one layer up. Almost every port has
to leave the vertex domain and work in `Pre` positions by hand, the only
ready-made path structure is the non-commutative two-tree `Hld`, and four
standard techniques have no library form at all: rerooting, centroid
decomposition, the auxiliary tree, and merging one child's structure into
another's.

## 1. The `Rooted` API: `sub`, `lca`, `fold_path`, `apply_path`, `reroot`

### What works

`sub(v)` as a `Range<Pre>` and `lca` were used in eight ports and never
needed a second thought. The whole query side of `ioi18p3` is two subtree
ranges and a Fenwick tree keyed by `Pre`:

```
shared[a.trip] += a.sign * seen.fold(wolf_range[a.trip])
```

`chains(a, b, skip_top)` is also the right primitive. `sdoi17r1p2` walks a
root path chain by chain and only touches the pointers that have to move,
which is the link-cut "access" bound without a link-cut tree.

### TR1, TR8: there is no `fold_path` / `apply_path` over a plain array

`stdlib/tree.cp` has `Hld.path`, `Hld.path_edges` and `Hld.sub`. The names
this backlog was asked about, `fold_path`, `apply_path` and `reroot`, do
not exist. `Hld` keeps two segment trees so that folds can be ordered, and
has no lazy variant. In `dmopc20c7p4` the value is a plain counter, the
fold is a sum, and the update is "add one":

```
var near = Hld(t, Sum<Int>, [V] v => 0)
if let p = t.up[u]: near.set(p, near.get(p) + (if safe[u]: 1 else: -1))
...
var ans = near.path(u, v)
```

A commutative sum pays for the second tree and spells an add as
get-then-set. With a range update it gets worse: `sdoi17r1p2` builds its
own lazy tree over `Pre` and every use hops through `tin`, `at` or `sub`:

```
var f = LazySegTree<Rooted<V>.Pre, ?Int, Int>.from(MaxOpt, AddMax, [Rooted<V>.Pre] p => t.depth[t.at[p]] + 1)
if let c = same[v]: f.apply(t.sub(c), 1)
print(f.get(t.tin[x])! + f.get(t.tin[y])! - 2 * f.get(t.tin[top])! + 1)
```

Proposal: one family of tree-keyed structures, built from the tree, that
take vertices and never show `Pre`.

```
var near = t.sums([V] _ => 0)                 // Fenwick; add, fold_path, fold_sub
near.add(p, 1)
ans = near.fold_path(u, v)

var f = t.lazy(MaxOpt, AddMax, v => t.depth[v] + 1)
f.apply_sub(c, 1)                             // also f.apply_path(a, b, x)
f.get(x); f.fold_sub(x); f.fold_path(a, b)
```

`fold_path` on a commutative monoid needs one tree; the ordered fold stays
as today's `Hld`. `path_edges` becomes a flag (`fold_path(a, b, edges:
true)`), matching `chains(.., skip_top)`.

### TR6: `reroot`

Two different things are called rerooting and the library has neither.

The first is "root it somewhere else". `noip19p6` roots at vertex 1 only to
learn sizes, finds the centroid, then builds the whole structure again:

```
let first = Rooted<V>.new(u => g.adj(u), V.first)
var c = V.first
while let h = first.heavy[c]:
    if 2 * first.size[h] <= n: break
    c = h
let t = Rooted<V>.new(u => g.adj(u), c)
```

`t.centroid()` (one or two vertices) and `Rooted<V>.at_centroid(g)` would
make that one line.

The second is the rerooting DP: an answer for every root from one
bottom-up and one top-down pass. None of the ten originals turned out to
need it once solved from the statement (the corpus scan flagged several by
their `dfs2`, all false positives), so there is no port to quote, but the
shape is fixed enough to write down:

```
let ans: [V]T = t.reroot(m, lift: (acc, child, parent) => ..., leaf: v => ...)
```

where `m` is the monoid that merges children and `lift` carries a merged
value across an edge. The prefix/suffix product over `kids` that makes it
O(n) without inverses is the part people get wrong; it belongs in the
library.

### TR5, TR7: the three standard walks

Every port iterates the tree in one of three orders, and each is spelled
through `Pre`:

```
for p in Rooted<V>.Pre.rev:          // mmcc14p3, noi20p2: children first
    let v = t.at[p]
for p in Rooted<V>.Pre:              // noip19p2, ioi09p7: parents first
    let v = t.at[p]
```

`for v in t.bottom_up:` and `for v in t.top_down:` say what is meant. The
third order has no spelling: `noip19p6` needs "the multiset of ancestors of
the current vertex", which is an enter/exit walk, and falls back to a
recursive function that would overflow the stack on a 3e5-vertex path:

```
fn walk(x: V):
    above.add(Size(n - t.size[x]), 1)
    ...
    for k in t.kids[x]: walk(k)
    ...
    above.add(Size(n - t.size[x]), -1)
```

`for (v, entering) in t.tour:` yields each vertex twice from the arrays
`Rooted` already has, with no recursion.

### TR3: trees given as a parent list

Four of the ten inputs give parents, not edges, and each differs: no entry
for the root (`ioi09p7`, `noip19p2`), a `-1` at the root (`ceoi22p3`), or a
parent produced by the program itself (`ioi18p3`). Each port builds child
lists only to hand them back as an adjacency function, and the element
must be a record with a `to` field:

```
var kids: [V]List<(to: V)> = [V] _ => []
for v in V:
    if v == V.first: continue
    let boss: V = read()
    kids[boss].push((to: v))
let t = Rooted<V>.new(u => kids[u], V.first)
```

`Rooted<V>.from_parents(boss: [V]?V)` covers all four, with the read side
written as `let boss: [V]?V(none: -1) = read()` or, for the "root has no
entry" form, a read over `V` without its first element.

### TR13: climbing with a predicate

`ioi18p3` needs the highest ancestor that still satisfies a monotone test.
`ancestor(v, k)` exists, so the port bisects on `k`, O(log^2 n):

```
fn climb(t: Rooted<V>, v: V, ok: V -> Bool) -> V:
    var (lo, hi) = (0, t.depth[v])
    while lo < hi:
        let mid = (lo + hi + 1) / 2
        if ok(t.ancestor(v, mid)!): lo = mid
        else: hi = mid - 1
    t.ancestor(v, lo)!
```

`t.climb(v, ok)` can walk heavy chains and bisect inside one chain, which
is O(log n) and needs no lifting table.

### TR12: a tree built by union-find

The same port builds two reachability (Kruskal reconstruction) trees.
`std.dsu` chooses the representative by size, and here the newly added
vertex must become the top, so the port rewrites `find`. Either
`Dsu<V>.unite_under(child, top)` or a ready `reach_tree(order, adj)`
returning a `Rooted<V>` removes fifteen lines.

## 2. Centroid decomposition

The corpus has no unported centroid-decomposition submission (`ioi11p2` is
already ported), so this section is argued from `noip19p6` and from what
the decomposition would have to look like in this language.

The classical C++ shape is a recursive function that marks a centroid as
removed, loops over its neighbours skipping removed ones, and recurses.
Three things about it do not fit CP Lang as it stands. The `removed` array
is mutated between sibling recursive calls, so every helper takes it `mut`
or it becomes a global. Each level needs its own sizes, and `Rooted` is
immutable and whole-tree, so the natural move (build a `Rooted` per
component) is O(n log n) allocations of seven arrays each. And the
component a centroid owns is not a `Range` of anything, so the "indexed
slice" story that makes `sub(v)` pleasant does not apply.

The proposal is to give the decomposition as data rather than as a
callback protocol:

```
let cd = Centroids<V>.new(g)          // cd.up[v]: ?V, the centroid above; cd.level[v]
for c in cd.order:                    // centroids, top level first
    for (v, d, branch) in cd.part(c): // its component: vertex, distance to c, which neighbour's side
        ...
for a in cd.above(v):                 // v's O(log n) centroid ancestors, with dist(v, a)
    ...
```

`part(c)` covers the offline "count pairs through c" problems (`ioi11p2`
would be a loop over `part` with a map keyed by length), and `above(v)`
covers the online "nearest marked vertex" kind. Distances come from the
stored per-level arrays, O(n log n) memory, and no user code touches a
`removed` flag.

## 3. Small-to-large and mutable borrowing

`noi20p2` is the port where the language loses. The DP row of a vertex is
its heavy child's row, edited, with the light children multiplied in. The
port wants to move the row out of the array:

```
var row: List<Z> = []
if let h = t.heavy[x]:
    row = g[h]
    g[h] = []
```

On the interpreter `row = g[h]` aliases and the port works. Under the
borrow rules as approved this is a copy, or an error, and neither is what
is meant. Two things are missing.

TR11, an owning move out of an indexed place. `take(g[h])` leaves the
default value behind and gives the caller the storage, the same contract
as `(adj[k] ??= [])` in reverse. It is one intrinsic, and it is also what
every "keep the bigger set" loop needs:

```
var mine = take(bag[h])
for c in light:
    for x in take(bag[c]): mine.add(x)      // small into large
bag[x] = mine
```

Without `take`, the usual workaround is `swap(mut bag[x], mut bag[h])`,
which needs the `law disjoint` proof that `x != h`. That is provable (a
vertex is not its own heavy child) but not by the range checker, so the
port would carry an `unchecked` for the most common idiom in this area.

TR10, a structure whose parts can change owner. The accepted solution
keeps each row as a sparse segment tree over depths with a lazy multiplier
and merges two trees node by node, O(n log n) in total. The port keeps a
dense `List<Z>` per vertex and is O(n^2) on a path. No stored borrows and
no pointers means a mergeable tree has to be an arena with indices, owned
by one value shared by all rows:

```
var pool = SegPool<Depth, Z, Mul>.new()         // one arena
var root: [V]pool.Node = ...                    // handles, Copy
root[x] = pool.merge(root[x], root[c], (a, b) => a * b)
```

The handles are a sealed index into the pool, which fits the existing
`Index` design, but it is a new library and the merge callback with lazy
tags (what happens when one side is empty) is the hard part of the
problem, not something the library can hide entirely. This is the largest
gap the batch found.

## 4. Virtual (auxiliary) trees

`ceoi22p3` uses the idea in its simplest form: sort the picked vertices by
preorder position and look only at neighbours.

```
sort(mut picked, by: v => two.tin[v].ord)
for i in Ask: print("?", picked[i.ord], picked[i.ord + 1])
```

That was painless. The full technique (TR14) is not expressible cleanly
today. The auxiliary tree's vertices are a subset of `V` that changes per
query, so its arrays cannot be `[V]T` without paying O(n) per query, and
the language's answer to "a fresh small index set" is a frozen domain:

```
let aux = t.aux(picked)            // domain aux.K; aux.v: [K]V; aux.up: [K]?K; aux.len: [K]Int
var dp: [aux.K]Int = 0
for k in aux.K.rev:                // children before parents, as in Pre
    if let p = aux.up[k]: dp[p] += dp[k] + aux.len[k]
```

This needs a domain whose size is known only at run time, created inside a
loop over queries, with arrays over it that die with the iteration. `let K
= n` covers the size; what has not been exercised is doing it q times with
total size bounded by the sum of the picked sets. If a per-query domain
allocates, the technique loses its point. It should be a named test for
the bounded-index design.

## 5. Constraints the types cannot state

Tree problems carry promises that matter for correctness and that no
domain expresses.

"The edges form a tree" is the first. `let edges: [E](u: V, v: V) = read()`
with `E = 0..(n - 1)` gets the count right and nothing else. `Rooted.new`
stops at run time if a vertex is unreachable, which is the right behaviour;
a stress generator derived from the types will produce a random graph with
n - 1 edges, almost never a tree. `cpl new` and `cpl stress` need a `Tree<V>`
input shape (with path, star, caterpillar and random-parent generators),
because tree bugs live in the shapes: the recursion in `noip19p6` and the
dense rows in `noi20p2` are both fine on random trees and both fail on a
path.

`noi20p2` promises that in every given pair `u` is an ancestor of `v`. The
port relies on it (it reads `t.depth[p.u]` as the constraint) and nothing
checks it. `ioi18p3` promises `l <= s`, `e <= r` and a connected graph.
`ioi09p7` promises `r1 != r2`; the original and the port disagree when
they are equal, which the checker found on its first run. `btoi15p3`
promises n >= 3. These want a `requires` on the input that the generator
honours and the debug build asserts:

```
let paths: [Path](u: V, v: V) = read() requires all p: t.is_anc(p.u, p.v) and p.u != p.v
```

The second kind is output that is not unique. `btoi15p3` accepts any
optimal set of links, so `check.py` carries a hand-written bridge test;
that is the `checker` block already approved, and this is a clean example
for it. `ceoi22p3` is interactive with hidden data, and its judge in
`check.py` is the `interactor` block.

The third is depth. Nothing in a type says "this recursion is as deep as
the tree". The compiler could warn when a recursive function's argument is
a child of its parameter in a `Rooted`, and the library should make the
warning rare by offering the iterative walks of section 1.

## 6. Collisions between features

`Pre` is a struct-owned domain and that collides with everything that
wants to name it. A domain alias is rejected (`domain Pre = Rooted<V>.Pre`),
so `sdoi17r1p2` writes `Rooted<V>.Pre` five times, and with two trees over
the same `V` (`ioi18p3`, `ceoi22p3`) the two `Pre` domains are the same
type although a position in one tree means nothing in the other:

```
seen.add(wolf.tin[human.at[Rooted<V>.Pre.nth(k.ord)]], 1)
```

Swapping `wolf` and `human` in that line typechecks. A domain owned by the
value (`human.Pre`, `wolf.Pre`) would make it an error; that is the same
mechanism section 4 needs for `aux.K`.

`Bounds<D>` against integers. Offline sweeps bucket queries by a boundary
of a range, 0..=n. The port declares `domain Cut = 0..=n` and converts with
`Cut(h.lo.ord)`. `[Bounds<Pre>]T` as an array key would remove the
conversion and the off-by-one it invites.

Optional values against lazy trees. `Max<Int>` has identity `Int.MIN`, and
adding to it traps, so range-add range-max is `LazySegTree<_, ?Int, Int>`
with `MaxOpt` and `AddMax`, and every read ends in `!`. The three `!` in
one line of `sdoi17r1p2` are noise the user cannot remove. A tree built
from a total function (`from`) has no empty leaves; `get` and a non-empty
`fold` could return `T`.

`?V` against open ranges. `for v in V.first + 1..:` is the natural "every
vertex but the root" and fails when n = 1, because `V.first + 1` is `none`.
Both ports that used it were changed to a `continue`. An open range from
`none` should be empty.

Labels against indices. `v.to_idx` is 0-based and `v.label` is the number
in the statement. `noip19p6` sums vertex labels and the first version used
`to_idx`; every answer was wrong by the number of terms. Where a domain
value meets arithmetic, `v.label` is almost always what is meant, and a
bare `v * k` could mean it.

Operations read by tag (TR2). Three ports read "1 x" / "2 x y" style
queries by reading the tag and then branching on how many values follow.
An `enum` read by its tag is on the cheat sheet; the interpreter does not
have it, and these tree problems are where it would be used most.

## 7. Small things the interpreter lacks

These cost a retry each and are listed so the next batch does not rediscover
them: `bsearch_first` (TR4; three ports bisect by hand), reading `[V]Char`
from one token (TR9), printing a list on one line (TR15), `flush()` and
reading stdin incrementally (TR16; the interpreter reads all input before
it starts, so an interactive port can only be checked by running it
twice), `while let x = stack.pop()` (TR17), `if let` as the body of an
expression function, and an inline `[domain Q = 0..t]`.

## Ranking

By how many ports each would have shortened, and by how wrong the
workaround can go:

1. Tree-keyed structures with `fold_path`, `apply_path`, `fold_sub`,
   `apply_sub` (TR1, TR8): four ports.
2. `bottom_up`, `top_down`, `tour` (TR5, TR7): six ports, and removes the
   only recursion in the batch.
3. `from_parents` (TR3): four ports.
4. `take` out of an indexed place (TR11): one port here, every
   small-to-large solution elsewhere.
5. Value-owned domains (`t.Pre`, `aux.K`): fixes the two-tree mix-up and
   is the precondition for auxiliary trees.
6. `Centroids<V>`, `t.centroid()`, `t.reroot(...)` (TR6): no port in this
   batch could use them, because the corpus has run out of such problems.
7. A mergeable segment-tree pool (TR10): one port, and the only item that
   decides whether a 25-point problem can be solved at full size.
