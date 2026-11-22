This chapter covers the contest algorithms that show up across almost every Division 1/2 round and Olympiad: union-find, graphs and shortest paths, rooted trees with LCA and HLD, network flow, modular arithmetic and combinatorics, number theory, string matching, and exact integer geometry.

## Union-find (`Dsu`) and rollback

`std.dsu.Dsu<V>` is parameterized by your vertex domain `V`, so you never mix 0-based and 1-based vertex IDs or pass an edge index where a vertex belongs. It supports `.unite(a, b) -> Bool`, `.find(v) -> V`, `.same(a, b)`, `.size(v)`, `.components`, and `.groups()`.

When you need offline dynamic connectivity (divide-and-conquer on time), snapshot and restore with `.mark()` and `.undo_to(snap)` — rollback disables path compression automatically when snapshots are active so every undo is `O(1)`.

## Modular arithmetic (`Mod<P>`) and combinatorics (`Comb<M>`)

`std.mod.Mod<P>` stores residues in a compact 4-byte `uint32_t` with zero-cost compile-time modulus reduction (`P` can be a `const` like `998_244_353` or a runtime variable for problems that read the modulus from input). Division `a / b` uses the modular inverse automatically, and `Comb<Z>(max_n)` (for `type Z = Mod<P>`) precomputes factorials and inverse factorials in `O(n)` for `O(1)` `.choose(n, k)` and `.perm(n, k)`.

```example dsu_mod
```

```exercise graphs-dsu-mod-output
```

```exercise graphs-mod-ord
```

Integer literals use the expected `Mod<P>` type, so initialization and compound updates need no `Z(0)` or `Z(1)` wrappers. For an `Int` variable, use an explicit `Z(x)` conversion. Numeric type arguments are available as members of the specialized type: `Z.P` reads its modulus.

```cp CP Lang 0.2.2
type Z = Mod<7>
var ways: Z = 0
let unit: Z = 1
ways += 1
ways -= 2
ways *= 3
print(ways, unit, Z.P, Mod<11>.P) // 4 1 7 11
```

## Graphs, BFS, Dijkstra, and SCC

`std.graph` constructs compact flat CSR (Compressed Sparse Row) graphs — much faster and more cache-friendly than `vector<vector<int>>` in C++:

```cp CP Lang 0.2.2
let (n, m): (1..=2e5, 0..=2e5) = read()
domain V = 1..=n
domain E = 0..m
let edges: [E](u: V, v: V, w: Int) = read()

let g = Graph.directed(edges)
let dist = dijkstra(g, from: [(V.first, 0)], w: a => edges[a.e].w)
print(dist[V.last] ?? -1)
```

`from` is a list of `(vertex, starting distance)` pairs, so several sources are one call, and `w` gives the weight of an arc: `a.e` is the index of the edge it came from, `a.to` the vertex it leads to. `Graph.directed` and `Graph.undirected` read the edge fields `u` and `v` by name (other fields, like `w`, ride along), and arcs carry `.e` when the edges are an array over a declared domain such as `[E]`; over a plain `[0..m]` an arc is just its target.

Notice two contest conveniences:

- Unreachable vertices have distance `none` (`?Int`), not `1e18`. You never accidentally add an edge weight to `INF` and overflow.
- `bfs` and `dijkstra` also accept an **implicit neighbor function** `v => ...` alongside explicit `Graph` objects, so grid BFS (`p => p.adj4`), state-space graphs `(v, mask)`, and bitmask transitions work with the exact same calls and zero graph construction.

`Graph`, `bfs`, `dijkstra`, `topo`, and `peel` are auto-imported. `peel` runs Kahn's algorithm and returns the vertices it removes; `topo` requires a complete order and returns `none` if a cycle remains. The remaining vertices can be on a cycle **or reachable from one**, so `peel` is not a list of precisely the non-cycle vertices.

`peel(g)` and `topo(g)` take the vertex domain from `g`, and a neighbour lambda's domain is the one it indexes, so `peel(v => adj[v])` needs no annotation either.

```cp CP Lang 0.2.2
domain V = 0..4
let edges = [(V(0), V(1)), (V(1), V(2)), (V(2), V(1)), (V(2), V(3))]
let g = Graph.directed(edges)
print(bfs(g, from: [V.first])) // 0 1 2 3
print(peel(g))                // only 0 is removed; 3 is downstream of the cycle
print(topo(g), none: "cycle")
```

```exercise graphs-bfs-optional
```

## Rooted trees, LCA, and HLD (`Rooted<V>`)

`std.tree.Rooted<V>(g, root)` computes parent pointers, depths, subtree sizes, heavy-light decomposition, and preorder intervals in one linear pass.

Its key innovation is a **value-owned preorder domain** `t.Pre`:

```cp CP Lang 0.2.2
use std.tree.{Rooted}
use std.segtree.{SegTree}

domain V = 1..=4
let g = Graph.undirected([(V(1), V(2)), (V(2), V(3)), (V(2), V(4))])
let weight: [V]Int = [V] v => v.to_idx + 1
let (u, v) = (V(3), V(4))
let t = Rooted(g, V.first)
var st = SegTree<t.Pre, Int>(+)
for v in V: st.set(t.pos(v), weight[v])

let sub_sum = st.fold(t.sub(u))          // t.sub(u) has type Range<t.Pre>
let path_sum = t.fold_path(u, v, st)     // HLD path fold in O(log^2 n)
let anc = t.lca(u, v)                    // O(log n) LCA
print(sub_sum, path_sum, anc)
for (c, rest) in t.excl_fold(V(2), +, c => weight[c]):
    print(c, rest)                      // 3 4, then 4 3: sum of the other children
```

Because `st` is indexed by `t.Pre` and `t.sub(u)` returns `Range<t.Pre>`, the type checker guarantees you can never pass a raw vertex `u: V` to `st.set(...)` without mapping it through `t.pos(u)`.

For rerooting DP, `t.excl_fold(v, m, f)` yields `(child, rest)` for every child of `v`, where `rest` folds `f` over all its siblings. It evaluates `f` once per child and takes O(degree(v)) total work using prefix/suffix folds, without requiring division or inverses. Child order is preserved, so a custom `Monoid(id:, op:)` need not be commutative.

```exercise graphs-rooted-domain
```

## Network flow with typed vertices

`FlowGraph<Node>` provides Dinic's algorithm over a vertex domain. Keep the left/right-to-vertex conversions in two helpers so offset arithmetic has one home. Payload enums are intended to replace these conversions, but payload-enum flow indexing is not yet supported by the execution backend:

```cp CP Lang 0.2.2
use std.flow.{FlowGraph}

let (n, m) = (2, 2)
domain L = 0..n
domain R = 0..m
domain Node = 0..(n + m + 2)
let (src, sink) = (Node(0), Node(1))
fn left(v: L) -> Node: Node(2 + v.to_idx)
fn right(v: R) -> Node: Node(2 + n + v.to_idx)

let pairs: List<(L, R)> = [(L(0), R(0)), (L(1), R(1))]

var fg = FlowGraph<Node>()
for a in L: fg.add(src, left(a), 1)
for b in R: fg.add(right(b), sink, 1)
for (a, b) in pairs: fg.add(left(a), right(b), 1)
print(fg.max_flow(src, sink))
```

This example matches both pairs and prints `2`. The domain gives the network exactly `n + m + 2` vertices; `left` and `right` return checked `Node` values rather than untyped integers.

## Number theory, string algorithms, and integer geometry

The remaining `std` modules cover the classic toolkit with the same domain-typed discipline:

- **Number theory (`std.math`):** `gcd`, `lcm`, `ext_gcd(a, b) -> (g, x, y)`, `pow_mod(a, e, m)`, `Sieve(n)` (`.is_prime(x)`, `.spf[x]`, `.primes`), `factor(n)`, `divisors(n)`.
- **Strings (`std.strings`):** `z_function(s)`, `prefix_function(s)`, `SuffixArray(s)` (`.sa`, `.lcp`), and `RollingHash(seq)` (dual-base salted hashing over `Str` or any `[D]T`, with `.get(l..r) -> HashVal`).
- **Exact geometry (`std.geometry`):** `Pt(x, y)` with `dot`, `cross`, `norm2`, `turn(a, b, c)` (`-1 | 0 | 1`), and `ByAngle` for integer angular sorting without calling `atan2`.

```exercise graphs-geometry-exact
```

## Write a program

```exercise graphs-write-components
```
