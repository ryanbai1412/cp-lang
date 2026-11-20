The standard library is written in CP Lang, not wired into the compiler. Every container follows one uniform verb vocabulary—`.len`, `.is_empty`, `.clear()`, `.first`, `.last`—and compiles down to flat C++17 with zero extra indirection.

## Collections

Construct empty `List<T>` and `Deque<T>` with `= []`; construct all other containers with `Type()`.

- **`List<T>`:** `.push(x)`, `.push_id(x) -> Int`, `.pop() -> ?T`, `.first`, `.last`, `xs[l..r]`, `xs + ys`, and `xs.freeze() -> (domain D, [D]T)` (replaces dedicated pool types).
- **`Deque<T>`:** `.push(x)`, `.pop() -> ?T`, `.push_front(x)`, `.pop_front() -> ?T`, `.first`, `.last`, `q[i]`.
- **`MonoDeque<T>(min | max)`:** imperative monotone queue for sliding windows and CHT/DP transitions: `.push(at: Int, val: T)`, `.drop_before(min_at: Int)`, `.clear()`, `.best -> ?T`.
- **`MinHeap<T>()` / `MaxHeap<T>()`:** `.push(x)`, `.pop() -> ?T`, `.peek -> ?T`. For mixed-order keys inside tuples or `sort(..., by:)`, wrap descending fields with `desc(x)` (type `Desc<T>`):

```cp CP Lang v2
var pq = MinHeap<(Int, Desc<V>)>()
pq.push((dist, desc(u)))
sort(mut items, by: x => (x.cost, desc(x.value)))
```

- **`Set<T>()` / `SortedBag<T>()` / `HashSet<T>()`:** `.add(x)`, `.remove(x)`, `x in s`, `|`, `&`, `-`, `^`. Ordered sets add `.first`, `.last`, `.pop_first()`, `.pop_last()`, `.first_ge(k)`, `.first_gt(k)`, `.last_le(k)`, `.last_lt(k)`, `.rank(k)`, `.nth(i)`.
- **`Map<K, V>()` / `HashMap<K, V>()`:** `m[k]` (traps if missing), `m[k] f= x`, `m[k] ??= init`, `m.get(k) -> ?V`, `m.remove(k)`, `k in m`, `for (k, v) in m.items`. All hash tables use per-process salted open addressing.
- **`Bitset<D>()` & `Subset<D>`:** `.add(v)`, `.remove(v)`, `.toggle(v)`, `v in s`, `|`, `&`, `-`, `^`, `.len` (popcount), `s.shifted(k)`. `Subset<D>` (`D.K <= 25`) is `Copy` + `Index` and supports `for g in s.subsets`, `s.with(v)`, `s.without(v)`.

## Search, sliding windows, and sequences

Binary search uses three free functions on ranges (also available as methods on domains; `F64` ranges take `steps: 80`):

```cp CP Lang v2
let ans = bsearch_first(1..=1e9, x => feasible(x))
let hi = bsearch_last(0..=n, k => cost(k) <= budget)
let hit = bsearch_first_some(lo..=hi, x => try_build(x))
```

Sorted lookup is uniform across `List`, `[D]T`, `Set`, `Map`, and `SortedBag`: `xs.first_ge(x) -> ?I` and `xs.last_le(x) -> ?I`. Linear helpers are `first(it) -> ?T`, `a.find(x) -> ?I`, `min_by(it, key)`, and `max_by(it, key)`.

For sliding windows and sequences:

```cp CP Lang v2
var q = MonoDeque<Int>(max)
for (i, x) in a.items:
    q.push(i, x)
    q.drop_before(i - k + 1)

for (l, r) in slide_until(0..n, (l, r) => pre.sum(l..=r) <= limit):
    best max= r - l + 1
```

Also in the prelude: `D.windows(k)`, `sliding_min(a, k)`, `sliding_max(a, k)`, `compress(all)` (`xs.D`, `xs[v]`, `xs.at(val)`, `xs.first_ge(val)`), `prefix(a)` (`pre.sum(l..=r)` in 1D/2D), `scan(a, +)` / `scan(a, max)`, `runs(a)`, `xs.adjacent`, `sort(mut a, by:)`, `sorted(a, by:)`, `reverse(mut a)`, `dedup(mut a)`, `rotate_left(mut a, k)`, `next_permutation(mut a)`, `sum`, `any`, `all`, `count`, and `Perm<V>` (`.inv()`, `.cycles()`).

## Range structures: bare ops, `Monoid(id, op)`, and `Action`

Range structures accept bare operators (`+`, `*`, `^`, `&`, `|`, `min`, `max`, `gcd`) directly, or a custom `Monoid(id, op)`. Lazy segment trees accept stock tags (`AddSum`, `AddMin`, `AddMax`, `AssignSum`, `AssignMin`, `AssignMax`) or a custom `Action(id, compose, apply)`:

```cp CP Lang v2
var bit = Fenwick<V, Int>(a)                        // sum-only: .add(i, d), .sum(l..=r)
var rmq = SparseTable<V, Int>(min, a)               // O(1) .fold(l..=r)
var st = SegTree<V, Int>(+, a)                      // bare operator
var mx = SegTree<V, Node>(Monoid(Node.ID, merge), a)// custom monoid
var lz = LazySegTree<V, Int>(AddSum, a)             // range add + range sum

let aff = Action(
    id: (1, 0),
    compose: (outer, inner) => (outer.0 * inner.0, outer.0 * inner.1 + outer.1),
    apply: (tag, val, len) => tag.0 * val + tag.1 * len,
)
var affine = LazySegTree<V, Int>(aff, a)
```

`Fenwick` is intentionally sum-only; use `SegTree` for custom associative operations. `SegTree` and `LazySegTree` both support `.max_right(from, ok)` and `.min_left(to, ok)` for O(log n) segment-tree binary search. `Steps<K, V>(init)` handles piecewise-constant intervals (`Steps<K, Bool>` for interval sets), and `Dsu<V>()` includes `.mark()` / `.undo_to(snap)` for rollback DSU out of the box.

## Graphs, `Rooted<V>`, and flows

`Graph<V>` and `Graph<V, W>` pack into flat CSR storage (`Graph.undirected(es)`, `Graph.directed(es)`, `g[u]`). Unweighted algorithms accept either a `Graph` or a plain neighbor function `V -> Iter<V>`: `bfs`, `dijkstra`, `shortest` (auto 0-1 BFS for `0..=1` weights), `topo`, `topo_prefix`, `scc`, `condense`, `two_edge_components`, `biconnected_components`, `articulation_points`, `two_color`, and `mst`.

`Rooted<V>(g, root)` unifies rooted tree queries, lazy LCA, Euler tours, virtual trees, and heavy-light path operations into one structure:

```cp CP Lang v2
let tree = Rooted(g, root)
var seg = LazySegTree<tree.Pre, Int>(AddSum, 0)

tree.apply_path(a, b, mut seg, delta)
let ans = tree.fold_path(a, b, seg)
let sub_sum = seg.fold(tree.sub(u))
```

`Rooted<V>` also provides `.parent(v)`, `.children(v)`, `.depth(v)`, `.size(v)`, `.preorder`, `.postorder`, `.lca(a, b)`, `.dist(a, b)`, `.is_anc(a, b)`, `.ancestor(v, k)`, `.lift_while(v, pred)`, `.chains(a, b, edges?: Bool)`, and `.virtual_tree(keys)`.

Core `std` also ships `FlowGraph<Node>()`, `MinCostFlow<Node>()`, `max_weight_matching<L, R>`, and `TwoSat<V>()` (`.add_or(a, b)`, `.implies(a, b)`, `.solve() -> ?[V]Bool`).

## Modular arithmetic, math, and geometry

One `Mod<P>` type works for both compile-time `const` moduli and runtime-frozen `let` moduli:

```cp CP Lang v2
const MOD = 998_244_353
type Z = Mod<MOD>

let p: 1..=1e9 = read()
type DynamicZ = Mod<p>
```

`Mod<P>` packs into 4 bytes when `P <= 2^31 - 1`, supports `+`, `-`, `*`, `/`, `**`, `.pow(e)`, `.inv`, and intentionally does not implement `Ord`. Core math also includes `Comb<P>(n)` (`.fact`, `.inv_fact`, `.choose`, `.arrange`), `Matrix<R, C, T>` (`.pow(e)`, `.mul_with`), `Pt<T = Int>` (`.dot`, `.cross`, `.norm2`, `turn`, `turn_eps`, `ByAngle`, `convex_hull`), `gcd`, `lcm`, `isqrt`, `pow_mod`, `sieve`, and `factor`.

## Core `std` vs. local libraries (`examples/local-lib/`)

Specialized contest structures—`Rope`, `RangeKth`, `LiChao`, `PotentialDsu`, `Cover`, `Line`/`Hom`, `TwoStackFold`, `mo`, `Poly`/NTT, `XorBasis`, `Frac`, `Fixed`, `Tri`, and `Counter`—live in `examples/local-lib/` rather than core `std`. Because CP Lang keeps the language core general, a local library compiles to the exact same zero-overhead C++17 as `std` and is pinned by lockfile for reproducible submissions.
