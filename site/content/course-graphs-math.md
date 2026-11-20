This chapter covers the contest algorithms that show up across almost every Division 1/2 round and Olympiad: union-find, graphs and shortest paths, rooted trees with LCA and HLD, network flow, modular arithmetic and combinatorics, number theory, string matching, and exact integer geometry.

## Union-find (`Dsu`) and rollback

`std.dsu.Dsu<V>` is parameterized by your vertex domain `V`, so you never mix 0-based and 1-based vertex IDs or pass an edge index where a vertex belongs. It supports `.unite(a, b) -> Bool`, `.find(v) -> V`, `.same(a, b)`, `.size(v)`, `.components`, and `.groups()`.

When you need offline dynamic connectivity (divide-and-conquer on time), snapshot and restore with `.mark()` and `.undo_to(snap)` — rollback disables path compression automatically when snapshots are active so every undo is `O(1)`.

## Modular arithmetic (`Mod<P>`) and combinatorics (`Comb<P>`)

`std.mod.Mod<P>` stores residues in a compact 4-byte `uint32_t` with zero-cost compile-time modulus reduction (`P` can be a `const` like `998_244_353` or a runtime variable for problems that read the modulus from input). Division `a / b` uses the modular inverse automatically, and `Comb<P>(max_n)` precomputes factorials and inverse factorials in `O(n)` for `O(1)` `.choose(n, k)` and `.perm(n, k)`.

```example dsu_mod
```

```exercise graphs-dsu-mod-output
```

```exercise graphs-mod-ord
```

## Graphs, BFS, Dijkstra, and SCC

`std.graph` constructs compact flat CSR (Compressed Sparse Row) graphs — much faster and more cache-friendly than `vector<vector<int>>` in C++:

```cp CP Lang 0.2.2
use std.graph.{Graph, bfs, dijkstra, topo, scc, condense}

let (n, m): (1..=2e5, 0..=2e5) = read()
domain V = 1..=n
domain E = 0..m
let edges: [E](u: V, v: V, w: Int) = read()

let g = Graph.directed(edges)
let dist = dijkstra(g, from: [(V.first, 0)], w: a => edges[a.e].w)
print(dist[V.last] ?? -1)
```

`from` is a list of `(vertex, starting distance)` pairs, so several sources are one call, and `w` gives the weight of an arc: `a.e` is the index of the edge it came from, `a.to` the vertex it leads to.

Notice two contest conveniences:

- Unreachable vertices have distance `none` (`?Int`), not `1e18`. You never accidentally add an edge weight to `INF` and overflow.
- `bfs` and `dijkstra` also accept an **implicit neighbor function** `v => ...` alongside explicit `Graph` objects, so grid BFS (`p => p.adj4`), state-space graphs `(v, mask)`, and bitmask transitions work with the exact same calls and zero graph construction.

```exercise graphs-bfs-optional
```

## Rooted trees, LCA, and HLD (`Rooted<V>`)

`std.tree.Rooted<V>(g, root)` computes parent pointers, depths, subtree sizes, heavy-light decomposition, and preorder intervals in one linear pass.

Its key innovation is a **value-owned preorder domain** `t.Pre`:

```cp CP Lang 0.2.2
use std.tree.{Rooted}
use std.segtree.{SegTree}

let t = Rooted(g, V.first)
var st = SegTree<t.Pre, Int>(+, [t.Pre] p => weight[t.preorder[p]])

let sub_sum = st.fold(t.sub(u))          // t.sub(u) has type Range<t.Pre>
let path_sum = t.fold_path(u, v, st)     // HLD path fold in O(log^2 n)
let anc = t.lca(u, v)                    // O(log n) LCA
```

Because `st` is indexed by `t.Pre` and `t.sub(u)` returns `Range<t.Pre>`, the type checker guarantees you can never pass a raw vertex `u: V` to `st.set(...)` without mapping it through `t.pos(u)`.

```exercise graphs-rooted-domain
```

## Network flow with payload enums

In C++, Dinic's algorithm is notorious for off-by-one vertex numbering (`source = 0`, `left = 1..n`, `right = n+1..n+m`, `sink = n+m+1`). In CP Lang, pair `FlowGraph<Node>` with a **payload enum**:

```cp CP Lang 0.2.2
use std.flow.{FlowGraph}

enum Node: Src; Sink; Left(L); Right(R)

var fg = FlowGraph<Node>()
for a in L: fg.add_edge(Node.Src, Node.Left(a), 1)
for b in R: fg.add_edge(Node.Right(b), Node.Sink, 1)
for (a, b) in pairs: fg.add_edge(Node.Left(a), Node.Right(b), 1)
print(fg.max_flow(Node.Src, Node.Sink))
```

No manual offset math, no `2 * n + 2` sizing bugs, and the compiler packs `Node` densely into flat indices under the hood.

## Number theory, string algorithms, and integer geometry

The remaining `std` modules cover the classic toolkit with the same domain-typed discipline:

- **Number theory (`std.math`):** `gcd`, `lcm`, `ext_gcd(a, b) -> (g, x, y)`, `pow_mod(a, e, m)`, `Sieve(n)` (`.is_prime(x)`, `.spf(x)`, `.primes`), `factor(n)`, `divisors(n)`.
- **Strings (`std.strings`):** `z_function(s)`, `prefix_function(s)`, `SuffixArray(s)` (`.sa`, `.lcp`), and `RollingHash(seq)` (dual-base salted hashing over `Str` or any `[D]T`, with `.get(l..r) -> HashVal`).
- **Exact geometry (`std.geometry`):** `Pt(x, y)` with `dot`, `cross`, `norm2`, `orient(a, b, c)` (`-1 | 0 | 1`), and `ByAngle` for integer angular sorting without calling `atan2`.

```exercise graphs-geometry-exact
```
