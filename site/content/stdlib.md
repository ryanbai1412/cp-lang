The standard library is written in ordinary CP Lang rather than hardwired into the compiler. Every container shares one uniform vocabulary — `.len`, `.is_empty`, `.clear()`, `.take()` (an `O(1)` move-out leaving the container empty) — and compiles down to flat, tree-shaken C++17.

The prelude auto-imports common types and helpers (capped at 80 names), including `bsearch_first`, `bsearch_last`, `bfs`, `topo`, `peel`, and `resized`; the examples below need no imports for those names.

## Standard library modules

Every module under `stdlib/` is plain CP Lang. Import individual names with `use std.<module>.{Name}`:

| Module | Key exports | Purpose |
| --- | --- | --- |
| `std.algebra` | `Monoid`, `Action`, `Desc`, `desc`, `Reversible`, `MaxSub`, `fold` | Algebraic traits, Kadane monoid, descending key wrapper |
| `std.bag` | `SortedBag`, `SortedSet`, `BagCursor` | Ordered multiset & unique-key set with `.range(r)` views, rank, nth, cursors, and node recycling |
| `std.beats` | `BeatsSegTree` | Ji Driver Segment Tree Beats (`chmin`, `chmax`, `add`, `sum`, `min_val`, `max_val`) |
| `std.bitset` | `Bitset` | Fixed-domain bitset with bitwise ops and `.first_set` |
| `std.derive` | `eq_fields`, `lt_fields`, `show_fields`, `read_fields`, `Hash`, `hash_fields`, `Gen`, `gen`, `gen_fields` | The field-by-field `Eq`, `Ord`, `Show`, `Read`, `Hash` and `Gen` (a seeded random value for stress tests: `let p: P = gen_fields(mut rng)`), written on `fields`; call one from an `impl` (`fn lt(a, b) = lt_fields(b, a)` orders the other way, `fn hash(self) = hash_fields(self)` hashes a struct) |
| `std.deque` | `Deque` | Double-ended queue (`.push_front`, `.pop_front`, `Deque.of`) |
| `std.dp` | `LiChao`, `Line`, `dnc_dp`, `knuth_dp`, `aliens_search`, `monotone_minima` | DP optimizations (Li Chao tree, D&C DP, Knuth DP, Aliens/WQS, SMAWK) |
| `std.slope` | `Concave`, `Convex` | Slope trick: integer concave/convex functions as sorted slopes |
| `std.dsu` | `Dsu`, `DsuOf`, `ParityDsu`, `PotentialDsu`, `RollbackArray` | Disjoint set union (payload, parity, potential) and rollback array sharing `Mark` |
| `std.fenwick` | `Fenwick`, `XorFenwick`, `Fenwick2D`, `DualFenwick`, `RangeFenwick` | 1D/2D Fenwick, XOR, range-add/point-get, and range-add/range-sum |
| `std.flow` | `Arc`, `FlowGraph`, `CostFlow`, `max_matching`, `max_weight_matching`, `max_closure`, `gomory_hu`, `global_min_cut` | Dinic, min-cost flow, matchings, closure, bounded flows, cut trees |
| `std.geometry` | `Pt`, `Vec`, `turn`, `ByAngle`, `convex_hull`, `minkowski_sum`, `closest_pair`, `hull_tangents`, `half_plane_intersect`, `min_enclosing_circle`, `circle_polygon_area` | Exact 2D integer geometry (`I128` predicates, hulls, calipers, closest pair) and careful floating-point geometry |
| `std.graph` | `Graph`, `Tree<V>`, `bfs`, `dijkstra`, `peel`, `topo`, `scc`, `two_color` | CSR graphs, implicit-neighbor algorithms, `Tree<V>` alias |
| `std.grid` | `adj4`, `adj8`, `step`, `ray`, `Dir4`, `Dir8`, `transposed` | No grid type: a cell is a `(Row, Col)` pair over two domains, so `p.adj4` and `p.adj8` are its in-bounds neighbours, `p.step(d)` and `p.ray(d)` walk a `Dir4`/`Dir8`, and `bfs(p => [q for q in p.adj4 if open[q]], [start])` searches the grid |
| `std.hashmap` | `FlatMap` | Open-addressing splitmix64-salted flat hash map for `Int` keys |
| `std.heap` | `MinHeap`, `MaxHeap`, `ErasableMinHeap`, `ErasableMaxHeap`, `IndexedMinHeap`, `IndexedMaxHeap` | Binary, lazy-delete, and domain-keyed priority queues |
| `std.interval` | `IntervalMap` | Maximal-run interval partition over a domain with O(log n) amortized range assign |
| `std.lazy` | `LazySegTree`, `DualSegTree`, `LazyTreap`, `Flip`, `flip_action`, `AddSum`, `AddMin`, `AddMax`, `AssignSum`, `AssignMin`, `AssignMax`, `AffineSum`, `affine_sum` | Lazy/dual segment trees, lazy treap, and stock actions |
| `std.marks` | `Marks`, `MarkMap`, `Interner` | Generation-stamped O(1)-reset visited set/map and online key interner |
| `std.math` | `gcd`, `lcm`, `ext_gcd`, `ceil_div`, `choose2`, `Sieve`, `sieve_multiplicative`, `factor`, `divisors`, `inv_mod`, `crt`, `crt_all`, `mod_log`, `mod_sqrt`, `floor_sum`, `bit_len` | Number theory, primes, CRT, discrete log/sqrt, bit helpers |
| `std.matrix` | `Mat<T>`, `det_real`, `rank_real`, `solve_real`, `inverse_real`, `XorBasis` | Dense matrices, exact and floating-point Gaussian elimination, GF(2) XOR bases |
| `std.mo` | `mo_order`, `mo_steps` | Mo's offline range-query ordering and window transitions |
| `std.mod` | `Mod<P>`, `Comb<M>` | Compile-time and runtime modular arithmetic and combinatorics |
| `std.ntt` | `ntt`, `convolve`, `convolve_mod`, `convolve_int`, `SosOver`, `sos`, `fwht`, `convolve_or`, `convolve_and`, `convolve_xor`, `convolve_subset` | NTT, linear/cyclic/3-prime/exact convolution, SOS, FWHT, bitwise/subset convolutions |
| `std.poly` | `deriv`, `integral`, `inv`, `log`, `exp`, `pow`, `sqrt`, `eval`, `div_mod`, `prod_linear`, `taylor_shift`, `eval_all`, `compose`, `lagrange_at`, `interpolate` | Formal power series, multipoint ops & interpolation over NTT (`List<Mod<P>>`) |
| `std.persistent` | `PersistentSegTree`, `Version`, `RangeKth` | Path-copying persistent segment tree and O(log n) range k-th smallest |
| `std.prefix` | `prefix`, `Prefix`, `prefix_fold`, `suffix_fold`, `Diff` | 1D/2D/list prefix sums (`prefix(a)` dispatches across `[D]T`, `[R, C]T`, `List<T>`), monoid folds, 1D/2D difference arrays |
| `std.rand` | `Rng` | Seeded deterministic RNG (`.int(lo, hi)`, `.next()`) |
| `std.segtree` | `SegTree`, `MergeSortTree`, `SparseSegTree` | Segment tree, orthogonal 2D merge-sort tree, and coordinate-sparse segment tree |
| `std.seq` | `Perm<V>`, `pairs`, `with_prev`, `cyclic_pairs`, `windows`, `scan`, `scan_in_place`, `argmax`, `inversions`, `prev_less`, `next_less`, `sliding_min`, `inverse`, `perm_cycles`, `bsearch_first`, `bsearch_last` | Sequence views, searches, monotone stacks, permutations, folds, `Perm<V>` alias |
| `std.sparse` | `SparseTable`, `DisjointSparseTable`, `idempotent` | O(1) idempotent and non-idempotent associative range folds |
| `std.spec` | `convexity_break`, `monge_break`, `dnc_break`, `knuth_break`, `berlekamp_massey`, `differences`, `newton` | Conjecture checking and sequence/recurrence discovery |
| `std.strings` | `z_function`, `prefix_function`, `manacher`, `SuffixArray`, `SuffixAutomaton`, `PalindromicTree`, `Trie`, `AhoCorasick`, `RollingHash`, `Hash2D` | Pattern matching, palindromes, suffix structures, tries, and rolling hashes |
| `std.timetree` | `TimeTree`, `TimeStep` | Offline segment tree over time for rollback-based dynamic structures |
| `std.tour` | Tour re-exports | Euler tour utilities |
| `std.tree` | `Rooted<V>`, `Centroids<V>`, `LinkCut<V, T>`, `LazyLinkCut<V, T, F>`, `JumpTree<V>`, `PermTree<I, V, Node>` | Rooted tree, Euler tour, LCA, HLD path queries, rerooting DP, virtual trees, centroid decomposition, link-cut trees, trees grown leaf by leaf, permutation trees |
| `std.window` | `MonoDeque`, `WindowFold` | Sliding-window extrema and inverse-free monoid window folds |

## Prelude collections

Construct empty `List<T>` and `Deque<T>` with `= []`; construct all other containers with `Type()`.

- **`List<T>`:** `.push(x)`, `.push_id(x) -> Int`, `.pop() -> ?T`, `.insert(i, x)`, `.remove(i) -> T`, `.swap(i, j)`, `.first -> ?T`, `.last -> ?T`, `xs[l..r]`, `xs + ys`, and `xs.freeze() -> (domain D, [D]T)` for arena construction.
- **`Deque<T>`:** `.push(x)`, `.pop() -> ?T`, `.push_front(x)`, `.pop_front() -> ?T`, `.first -> ?T`, `.last -> ?T`, `q[i]`.
- **`MinHeap<T>()` / `MaxHeap<T>()`:** `.push(x)`, `.pop() -> ?T`, `.peek -> ?T`. For mixed-order keys inside tuples or `sort(..., by:)`, wrap descending fields with `desc(x)` (type `Desc<T>`):

```cp CP Lang 0.2.2
use std.algebra.{desc}

domain V = 0..4
let (dist, u) = (3, V.first)
type Item = (cost: Int, value: Int)
var items: List<Item> = [(2, 10), (1, 5)]
var pq = MinHeap<(Int, Desc<V>)>()
pq.push((dist, desc(u)))
items.sort(by: x => (x.cost, desc(x.value)))
```

- **`Set<T>()` / `SortedSet<T>()` / `SortedBag<T>()`:** `.add(x)`, `.remove(x)`, `x in s`, `.first`, `.last`. `Set` supports `.first_ge(k)`/`.first_gt(k)`/`.last_le(k)`/`.last_lt(k)`. `SortedSet` (unique keys) and `SortedBag` (keeps duplicates, with `.count(x)` and `.remove_one(x)`) support `.range(r)` — an O(1) view over any range (including open-ended `..b`, `..=b`, `a..`, `..`) whose `.first`, `.last`, `.len`, `.is_empty` cost O(log n) and whose iteration (`for x in s.range(r)` / `.rev`) costs O(log n + k). Neighbor lookups return `?K`, so an absent key is `none` rather than a sentinel.
- **`Map<K, V>()` / `HashMap<K, V>()`:** `m[k]` traps if `k` is absent (store `m[k] = v` first, or use `m.get(k) -> ?V` which never traps or inserts), `m[k] f= x` (auto-inserts from the operator identity: `0` for `+=`/`-=`, `none` for `min=`/`max=`), `(m[k] ??= init).push(x)` (always parenthesize `??=` when chaining or using as an expression — never `let v = m[k] ??= init`), `m.remove(k) -> ?V`, `k in m`, `for (k, v) in m.items`. Ordered `Map` lookups (`.first_ge(k)`, `.last_le(k)`) return `?(K, V)`. All hash tables use per-process splitmix64 salting to defeat anti-hash tests. `m.get_or_insert(k, v)` stores `v` when `k` is absent and returns the stored value.
- **`Bitset<D>()` & `Subset<D>`:** `Bitset<D>` provides `Bitset<D>.of(xs)` (from a `List<D>` or generator `k for k in D if ...`), `Bitset<D>.from(a)` (from `[D]Bool`), `.add(v)`, `.remove(v)`, `.toggle(v)`, `v in s`, `|`, `&`, `-`, `^`, `.len` (popcount), `.first_set -> ?D`, `s.shifted(k)`. `Subset<D>` (`D.K <= 25`) is a 4-byte `Index` value for bitmask DP: `Subset<D>.empty`, `Subset<D>.full`, `Subset<D>.single(v)`, `s.first -> ?D`, `s.with(v)`, `s.without(v)`, `s.complement`, `for mask in Subset<D>`, `for v in mask`, and `for sub in mask.subsets`. `s = s.set(v, on)` returns a copy with `v` put in or taken out by a `Bool`; on its own, `s.set(v, on)` leaves `s` unchanged.

## Search, sorting, and sequence helpers

`Deque.of(xs)` starts a queue from a list in front-to-back order. `a.resized(n, fill)` returns a new list, truncated or padded to length `n` in O(n); it does not mutate `a`. The same prelude helper can be called as `resized(a, n, fill)`:

```cp CP Lang 0.2.2
let xs = [1, 2]
var q = Deque.of(xs)
print(q.pop_front(), q.pop_front()) // 1 2
print(xs.resized(4, 0))             // 1 2 0 0
print(resized(xs, 1, 0), xs)        // 1, then the unchanged 1 2
```

Answer-space binary search uses two free functions (`F64` ranges take `steps: 80`), both returning `?I`:

```cp CP Lang 0.2.2
let n = 10
let budget = 25
fn feasible(x: Int) -> Bool: x >= 42
fn cost(k: Int) -> Int: k * k
let ans = bsearch_first(1..=1e9, x => feasible(x))
let hi = bsearch_last(0..=n, k => cost(k) <= budget)
print(ans, hi)
```

Sorted lookup uses `.first_ge(x)`, `.first_gt(x)`, `.last_le(x)`, `.last_lt(x)` everywhere, with the return type matching the container:

- On an array, `List`, or slice: returns the **index** (`?D` or `?Int`).
- On a `Set` or `SortedBag`: returns the **key** (`?K`).
- On a `Map`: returns the **pair** (`?(K, V)`).

Other prelude sequence helpers:

- `compress(all)` — coordinate compression producing a nominal domain `xs.D`, inverse `xs[id]`, `xs.at(val) -> xs.D`, and `xs.first_ge(val) -> ?xs.D`.
- `prefix(a)` — 1D (`List<T>`, `[D]T`) or 2D (`[R, C]T`) prefix sums (`pre.sum(l..=r)`, `pre.prefix(k)`).
- `scan(a, +)` / `scan(a, max)` — inclusive running folds (`a.scan_in_place(+)` in `std.seq` mutates a `List` without allocation); `a.fold(+)` / `a.fold(gcd)` / `a.fold((x, y) => ...)` — whole-sequence left fold across every `Seq` (monoids return their identity when empty; binary functions `(T, T) -> T` follow the same empty-sequence rule as `min`/`max`: trap when empty, or return `?T` with `!` / `??`); `a.excl_fold(+)` — yields `(x, rest)`, the fold of every other element, without division.
- `a.runs()` / `a.runs(by: key)` — run-length `(val, len)` pairs; `a.chunk_by(same)` (in `std.seq`) splits adjacent runs into slices and `a.group(by: key)` groups all elements by key; `xs.adjacent` — consecutive pairs; `D.windows(k)` — sliding windows.
- Sequence methods (prefer method form over free calls on one sequence): `a.sort(by:)`, `a.sorted(by:)` (O(n log n), returns a 0-indexed `List<T>`; descending via `desc: true` or `by: desc`; custom strict-weak-order comparator via `cmp: (a, b) => Bool`, mutually exclusive with `by:`), `D.sort(by: key)` (for domains), `a.reverse()`, `a.deduped()` (adjacent duplicates only, O(n) without hashing; in-place `dedup(mut a)` in `std.seq`), `a.distinct()` (all duplicates in first-seen order across any `Seq`, O(n) expected time / O(n) space via hashing); in `std.seq`: `rotate_left(mut a, k)`, `rotate_right(mut a, k)`, `next_permutation(mut a)`, `prev_permutation(mut a)`, `p.inverse()`, `perm_cycles(p)`.
- Method-form folds & searches: `xs.sum()`, `xs.prod()`, `xs.min(by:)`, `xs.max(by:)`, `xs.argmax(by:)`, `xs.argmin(by:)`, `xs.count(p)`, `xs.all(p)`, `xs.any(p)`, `xs.first(p) -> ?T`, `xs.find(x) -> ?I`, `xs.group(by: k)`. Use free `sum(f(x) for x in xs)` for generator expressions and `min(a, b)` / `max(a, b)` for two or more values.
- `xs.windows(k)` yields every length-`k` window as a list; `zip(xs, ys)` pairs elements up to the shorter list.
- `std.seq` also provides zero-allocation sequence views `pairs(xs)` (consecutive `(a, b)` pairs), `with_prev(xs)` (`(?T, T)` with `none` on the first element), `cyclic_pairs(xs)`, and the standard permutation alias `Perm<V> = [V]V` (paired with `std.graph.Tree<V>` for `cpl stress` generation and shrinking):

```cp CP Lang
use std.seq.{Perm, pairs, with_prev}
use std.graph.{Tree}

let xs = [1, 4, 9]
var diffs = 0
for (a, b) in pairs(xs):
    diffs += b - a
var jumps = 0
for (prev, cur) in with_prev(xs):
    if let p = prev:
        jumps += cur - p
print(diffs, jumps)
```
- Randomness: `rand(lo, hi)` and `shuffle(mut xs)` are in the prelude; `std.rand` adds a seeded `Rng.new(seed)` with `.int(lo, hi)` and `.next()`.

## Range data structures and union-find

Range structures accept bare associative operators (`+`, `*`, `^`, `&`, `|`, `min`, `max`, `gcd`) directly, or a custom `Monoid(id: identity, op: combine)`. Lazy segment trees accept stock tags (`AddSum`, `AddMin`, `AddMax`, `AssignSum`, `AssignMin`, `AssignMax`) or a custom `Action(id:, compose:, apply:, over:)`. Name the fields to make each callback's role clear:

```cp CP Lang 0.2.2
use std.algebra.{Monoid, Action}
use std.segtree.{SegTree}
use std.lazy.{LazySegTree}

domain V = 1..=4
let a: [V]Int = 3
struct Node:
    value: Int
fn merge(a: Node, b: Node) -> Node: Node(a.value + b.value)
let nodes: [V]Node = [V] v => Node(a[v])

var bit = Fenwick<V, Int>(a)                        // sum-only: .add(i, d), .sum(l..=r)
var rmq = SparseTable<V, Int>(min, a)               // O(1) .fold(l..=r) -> Int (non-empty range)
var st = SegTree<V, Int>(+, a)                      // bare operator
var mx = SegTree<V, Node>(Monoid<Node>(id: Node(0), op: merge), nodes)
var lz = LazySegTree<V, Int>(AddSum, a)             // range add + range sum

let aff = Action<Int, (Int, Int)>(
    id: (1, 0),
    compose: (new_f, old_f) => (new_f.0 * old_f.0, new_f.0 * old_f.1 + new_f.1),
    apply: (f, val, span) => f.0 * val + f.1 * span,
    over: Monoid<Int>(id: 0, op: (x, y) => x + y),
)
var affine = LazySegTree<V, Int, (Int, Int)>(aff, a)
affine.apply(V.first..=V.last, (2, 1))
print(bit.sum(V.first..=V.last), rmq.fold(V.first..=V.last), st.all, mx.all.value, lz.all, affine.all)
```

- `SegTree` and `LazySegTree` both provide `.set(pos, val)`, `.fold(l..=r)`, `.all` (`O(1)` root fold), and `O(log n)` binary search on the tree via `.max_right(l, ok)` and `.min_left(r, ok)`. Empty and inverted ranges are valid no-ops (except `SparseTable.fold`, which asserts a non-empty range and returns `T`).
- `Fenwick2D` handles 2D prefix sums; `Fenwick` stays dedicated to sums (`+`/`-`). `bit.kth(k) -> ?V` finds the first index whose prefix sum reaches `k` (non-negative counts).
- `Dsu<V>()` provides `.unite(a, b) -> Bool`, `.find(v) -> V`, `.same(a, b)`, `.size(v)`, `.components`, `.groups() -> List<List<V>>`, plus `.mark() -> Mark` and `.undo_to(snap)` for `O(1)` rollback DSU. `DsuOf<V, T>(init, join)` keeps a value per set (joined on merge, restored on `undo_to`), and `ParityDsu<V>` tracks same/opposite sides with `.unite(a, b, differ:)`, `.diff(a, b) -> ?Bool` and `.consistent`.
- `std.marks.Marks<D>` is an opt-in 32-bit generation-stamp visited table (`O(1)` reset); for most problems, `[D]Bool` with a touched-list reset is simpler and equally fast.

## Graphs, trees, flows, and strings

- **`Graph<V>` / `Graph<V, W>` (`std.graph`):** flat CSR storage via `Graph.undirected(es)`, `Graph.directed(es)`, and neighbor iteration `g[u]`. Algorithms accept either a `Graph` or an implicit neighbor function `v => ...`: `topo`, `peel`, `find_cycle`, `components`, `scc`, `condense`, `bridges`, `two_edge_components`, `biconnected_components`, `articulation_points`, `two_color`, and `euler_walk(edges, directed:) -> ?Walk`. Searches return `Paths` (`p[v]`, `p.reached(v)`, `p.path(t)`, `p.arcs(t)`, `p.order`): `bfs`, `bfs01(next, from:, w:)`, `dijkstra`, `bellman_ford` (`none` on a reachable negative cycle), `floyd_warshall(w: [V, V]?Int)`, and `fixed_cost_paths(from:, moves:, cost:, step:)` for heap-free search over a few fixed-cost move classes; `until: t` stops any of them once `t` is settled. `TwoSat<X>()` solves 2-SAT over a variable domain `X` with `Lit(x, value)` literals, `add_or`, `implies`, `must`, and `at_most_one`. A lambda's vertex domain is the domain it indexes (`peel(v => adj[v])`), and grid cells `(R, C)` are vertices too: `bfs(p => [q for q in p.adj4 if not wall[q]], [(R(0), C(0))])`.
- **`Rooted<V>(g, root)` / `Rooted.weighted(g, root, len)` / `Rooted.forest(g, roots)` / `Rooted.from_parents(par)` (`std.tree`):** a value-owned heavy-first preorder domain `t.Pre` in which every subtree and every heavy chain is one range. Shape: `.parent(v)`, `.children(v)`, `t.heavy[v]`, `.light(v)`, `.depth(v)`, `t.far[v]` (weighted depth), `.size(v)`, `.root_of(v)`. Orders: `t.pos(v) -> t.Pre`, `t.at[p] -> V`, `.sub(v) -> Range<t.Pre>`, `.below(v)` (without `v`), `.subtree(v)`, `.reorder(a)`, `for v in t.at` (parents first; `t.at.rev` children first), `t.tour` (`(v, entering)`). Paths: `.lca(a, b)`, `.dist(a, b)`, `.is_anc(a, b)`, `.ancestor(v, k)`, `.step(a, b, k)`, `.lift_while(v, pred) -> ?V`, `.path(a, b)`, `.diameter()`, `.chains(a, b, edges?: Bool)`, `.fold_path(a, b, ds)`, `.apply_path(a, b, mut ds, tag)`, `.virtual(keys)` (a `VirtualTree` over its own domain `vt.K`: `at`, `up`, `len`, `key`). DP: `.down(init, f)`, `.up(m, lift, finish)`, `.reroot(m, lift, finish)` (every root at once, O(n), no inverse needed), `.excl_fold(v, m, f)`, `.shapes()` (isomorphism classes).
- **`Centroids<V>(g)` (`std.tree`):** centroid decomposition in O(n log n): `.parent(c)`, `.depth(c)`, `.ancestors(v)` yields `(c, dist)` nearest first, `.piece(c)` yields `(v, dist, branch)`.
- **`LinkCut<V, T>.new(op, a)` (`std.tree`):** a forest under `link(u, v)` and `cut(u, v)`, with `connected`, `root_of`, `fold_path(u, v)` in path order, `set(v, x)` and `lca(u, v, root)`, each amortized O(log n). `LazyLinkCut.new(act, a)` adds `apply_path(u, v, f)` under a `LazySegTree` `Action`.
- **`PermTree.of(p)` (`std.tree`):** the binary permutation tree of a bijection `p: [I]V` in O(n): nodes `t.Node` (2n - 1), `t.kind[v]` (`Leaf`, `Incr`, `Decr`, `Partial`, `Full`), `t.left(v)`, `t.right(v)`, `t.span(v)` (positions) and `t.vals(v)` (values), each O(1).
- **`JumpTree<V>()` (`std.tree`):** a forest grown by `add(v, p)` in O(1), with `depth`, `ancestor`, `lca` and `lift_while` in O(log n), as on `Rooted`.
- **Network flow (`std.flow`):** `FlowGraph<Node>()` (Dinic), `CostFlow<Node>()` (min-cost flow), `max_matching<L, R>` (Hopcroft–Karp) with `min_vertex_cover`, `max_weight_matching<L, R>` / `min_weight_matching` (Hungarian), `general_matching` / `max_weight_general_matching` (Edmonds), `max_closure`, `circulation` / `max_bounded_flow` / `min_bounded_flow`, `gomory_hu` and `global_min_cut`, all over the caller's own domains; a network whose nodes come from several domains numbers them in one domain until sum domains land.
- **String algorithms (`std.strings`):** `z_function(s)`, `prefix_function(s)`, `SuffixArray(s)` (`.sa`, `.lcp`), and `RollingHash(seq)` (works on `Str` or any `[D]T`, `.get(l..r) -> HashVal`).

## Modular arithmetic, math, and geometry

One `Mod<P>` type handles both compile-time `const` moduli and runtime-frozen `let` moduli:

```cp CP Lang 0.2.2
const MOD = 998_244_353
type Z = Mod<MOD>

let p: 1..=1e9 = read()
type DynamicZ = Mod<p>
```

- **`Mod<P>` (`std.mod`):** 4-byte storage when `P <= 2^31 - 1`, `+`, `-`, `*`, `/`, `**` (power), `.inv` (getter, no parens), `.try_inv() -> ?Z` (for composite moduli), `.v -> Int` (underlying residue in `0..P`, not `.val` or `.raw`). `print(z)` prints `Mod<P>` directly. Intentionally omits `Ord`.
- **`Comb<M>()` (`std.mod`):** `.fact(n)`, `.inv_fact(n)`, `.choose(n, k)`, `.arrange(n, k)`, `.multinomial(ks)`, `.catalan(n)` (keyed on a `Mod` type `type Z = Mod<P>`; an immutable `let comb = Comb<Z>()` extends its tables lazily).
- **Number theory (`std.math`):** `gcd`, `lcm`, `ext_gcd`, `ceil_div`, `choose2`, `pow_mod`, `inv_mod`, `crt`, `crt_all`, `mod_log`, `mod_sqrt`, `primitive_root`, `floor_sum`, `Sieve(n)` (`sv.is_prime(x)`, `sv.factor(x)`, `sv.phi_table()`, `sv.mobius_table()`, `sv.div_count_table()`, `sv.div_sum_table()`, `sv.spf` array of smallest prime factors, `sv.primes: List<Int>` — note `Int` has no `.to_idx`; use `sv.primes` or build a prime-index table), `sieve_multiplicative`, `factor(n)`, `divisors(n)`, `divisors_from_factors(fs)`. (`isqrt` and `bit_ceil` are in the prelude — call `isqrt(n)` directly without `use std.math.isqrt`.)
- **NTT & convolutions (`std.ntt`):** `ntt<P>(mut a, inverse:, bit_reversed:)`, `convolve<P>(a, b, cyclic:)`, `convolve_mod(a, b, m, cyclic:)`, `convolve_int(a, b, cyclic:)`, in-place `sos(mut f, over: Sub | Sup = Sub, inverse:)` and `fwht(mut a, inverse:)`, and `convolve_or`/`convolve_and`/`convolve_xor`/`convolve_subset`.
- **Formal power series (`std.poly`):** `deriv`, `integral`, `inv(a, n)`, `log(a, n)`, `exp(a, n)`, `pow(a, k, n)`, `sqrt(a, n) -> ?List<Mod<P>>`, `eval(a, x)`, `div_mod(a, b)`, `prod_linear(roots)`, `taylor_shift(a, c)`, `eval_all(a, xs)`, `compose(f, g, n)`, `lagrange_at(ys, x)`, and `interpolate(xs, ys)` over NTT.
- **Matrices & XOR basis (`std.matrix`):** `Mat<T>` (`+`, `-`, `*`, `.pow(e)`, `.mul_vec(v)`, `.transpose()`, `.product(n)`, exact-field `.det()`, `.rank()`, `.solve(b)`, `.solve_space(b)`, `.inverse()`), floating-point `det_real`/`rank_real`/`solve_real`/`inverse_real` with explicit `eps:`, and `XorBasis` (`.add`, `.reduce`, `.contains`, `.max_xor`, `.min_xor`, `.kth`, `.merge`, `.monoid`).
- **Exact geometry (`std.geometry`):** `Pt<T = Int>` with `.dot`, `.cross`, `.norm2`, `orient(a, b, c)` (`-1 | 0 | 1`), `turn`, `turn_eps`, and `ByAngle` for exact integer angular sorting.

## Local libraries & advanced contest structures

Core `std` is intentionally small enough to memorize. Specialized algorithms live in **local libraries** — regular folders of `.cp` files with a `lib.toml` manifest that compile and tree-shake identically to `std`.

The repository includes a full sample library (`examples/local-lib/`) covering advanced contest topics:

- **Data structures (`ds/`):** `MonoDeque`, `Steps`, `Rope` (implicit treap), `RangeKth` (persistent segtree), `LiChao` & `LiChaoArena`, `Tri<D>` (triangular interval index), `PotentialDsu`, `Cover` (union-of-intervals), `TwoStackFold`, `mo_order`, `XorBasis`, `Interner`, and `RollbackArray`.
- **Math & polynomials (`math/`):** `Frac`, `Fixed<D>`, `Congruence` (general CRT), `Hom` / `BothWays`, `Mat<P>` (Gaussian elimination / `det` / `rank`), `Poly` / NTT / 3-prime MTT / Lagrange interpolation, `Powers`, SOS DP (`sos_sub`/`sos_sup`), Miller-Rabin + Pollard's rho (`primes`), and `floor_sum`.
- **Graphs & DP (`graph/`, `dp/`, `geom/`, `str/`, `seq/`):** `TwoSat`, `Functional` graphs, `KruskalTree`, `mst`, `shortest`/`walk_back`, `convex_hull`, `Dir8`, divide-and-conquer DP (`dnc_opt`), Knuth DP (`knuth`), `palindromes` (Manacher), `Trie`, `SuffixAutomaton`, `AhoCorasick`, `Hash2D`, `lis`, and `fold_others`.

### Starting your own local library

Create a directory with a `lib.toml` manifest and register it in your workspace's `cp.toml`:

```toml cp.toml
[libs]
lib = "../my-cp-lib"
```

Then import modules directly from any solution:

```cp CP Lang 0.2.2
use lib.ds.li_chao.{LiChao, Line}
use lib.graph.two_sat.{TwoSat}
```

Only `pub` declarations are exported, `@test` blocks live beside each module for `cpl test`, and `cpl build` inlines and tree-shakes only the functions your solution actually calls into the single submitted `.cpp` file.
