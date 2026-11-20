The standard library is written in CP Lang, not wired into the compiler. Every container follows one uniform verb vocabulary—`.len`, `.is_empty`, `.clear()`, `.take()` (an O(1) move-out that leaves it empty)—and compiles down to flat C++17 with zero extra indirection.

## Collections

Construct empty `List<T>` and `Deque<T>` with `= []`; construct all other containers with `Type()`.

- **`List<T>`:** `.push(x)`, `.push_id(x) -> Int`, `.pop() -> ?T`, `.insert(i, x)`, `.remove(i) -> T`, `.swap(i, j)`, `.first`, `.last`, `xs[l..r]`, `xs + ys`, and `xs.freeze() -> (domain D, [D]T)` (replaces dedicated pool types).
- **`Deque<T>`:** `.push(x)`, `.pop() -> ?T`, `.push_front(x)`, `.pop_front() -> ?T`, `.first`, `.last`, `q[i]`.
- **`MinHeap<T>()` / `MaxHeap<T>()`:** `.push(x)`, `.pop() -> ?T`, `.peek -> ?T`. For mixed-order keys inside tuples or `sort(..., by:)`, wrap descending fields with `desc(x)` (type `Desc<T>`):

```cp CP Lang v2.2
var pq = MinHeap<(Int, Desc<V>)>()
pq.push((dist, desc(u)))
sort(mut items, by: x => (x.cost, desc(x.value)))
```

- **`Set<T>()` / `SortedBag<T>()` / `HashSet<T>()`:** `.add(x)`, `.remove(x)`, `x in s`, `|`, `&`, `-`, `^`. Ordered sets add `.first`, `.last`, `.pop_first()`, `.pop_last()`, `.first_ge(k)`, `.first_gt(k)`, `.last_le(k)`, `.last_lt(k)`, `.rank(k)`, `.nth(i)`. Every ordered lookup on a set returns the key, `?K`.
- **`Map<K, V>()` / `HashMap<K, V>()`:** `m[k]` (traps if missing), `m[k] = v` (inserts or overwrites), `m[k] f= x` (auto-inserts from the operator's identity: `0` for `+=` and `-=`, `none` for `min=` and `max=`), `m[k] ??= init`, `m.get(k) -> ?V`, `m.remove(k) -> ?V`, `k in m`, `for (k, v) in m.items`. Ordered `Map` lookups (`.first_ge(k)`, `.last_le(k)`, ...) return the pair, `?(K, V)`. All hash tables use per-process salted open addressing.
- **`Bitset<D>()` & `Subset<D>`:** `.add(v)`, `.remove(v)`, `.toggle(v)`, `v in s`, `|`, `&`, `-`, `^`, `.len` (popcount), `s.shifted(k)`. `Subset<D>` (`D.K <= 25`) is a 4-byte value that implements `Index`: `Subset<D>.empty`, `Subset<D>.full`, `s.first -> ?D`, `s.with(v)`, `s.without(v)`, `for mask in Subset<D>` (all `2^K` subsets), `for v in mask` (its members), and `for sub in mask.subsets` (its submasks).

## Search and sequences

Binary search uses two free functions on ranges (`F64` ranges take `steps: 80`). Both return `?I`, so “no value satisfies the predicate” is `none`:

```cp CP Lang v2.2
let ans = bsearch_first(1..=1e9, x => feasible(x))
let hi = bsearch_last(0..=n, k => cost(k) <= budget)
```

Sorted lookup has one pair of names, `.first_ge(x)` and `.last_le(x)`, and the return type follows the container: a sequence (`List`, `[D]T`, a slice) returns the **index** (`?Int` or `?D`), a `Set` or `SortedBag` returns the **key** (`?K`), and a `Map` returns the **pair** (`?(K, V)`). On a `List<Int>` both an index and a value are `?Int`, so remember that it is the index. Linear helpers are `first(it) -> ?T`, `a.find(x) -> ?I`, `min_by(it, key)`, and `max_by(it, key)`.

A two-pointer window is an explicit loop, which can carry whatever incremental state the window needs:

```cp CP Lang v2.2
var l = 0
for r in 0..n:
    while pre.sum(l..=r) > limit: l += 1
    best max= r - l + 1
```

Also in the prelude: `D.windows(k)`, `compress(all)` (`xs.D`, `xs[v]`, `xs.at(val)`, `xs.first_ge(val)`), `prefix(a)` (`pre.sum(l..=r)` in 1D/2D), `scan(a, +)` / `scan(a, max)`, `runs(a)`, `xs.adjacent`, `sort(mut a, by:)`, `sorted(a, by:)`, `reverse(mut a)`, `dedup(mut a)`, `rotate_left(mut a, k)` (a negative `k` rotates right), `next_permutation(mut a)`, `prev_permutation(mut a)`, `sum`, `any`, `all`, `count`, `Perm<V>` (`.inv()`, `.cycles()`), and `Rng(seed)`, `rand(lo..hi)`, `shuffle(mut xs)`. Sort descending with `by: desc`.

## Range structures: bare ops, `Monoid(id, op)`, and `Action`

Range structures accept bare operators (`+`, `*`, `^`, `&`, `|`, `min`, `max`, `gcd`) directly, or a custom `Monoid(id, op)`. Lazy segment trees accept stock tags (`AddSum`, `AddMin`, `AddMax`, `AssignSum`, `AssignMin`, `AssignMax`) or a custom `Action(id, compose, apply, over: op)`. `LazySegTree<D, T>` takes two type arguments and infers the tag type from the action; `compose(new_f, old_f)` applies `old_f` first, then `new_f`:

```cp CP Lang v2.2
var bit = Fenwick<V, Int>(a)                        // sum-only: .add(i, d), .sum(l..=r)
var rmq = SparseTable<V, Int>(min, a)               // O(1) .fold(l..=r) -> Int, non-empty range
var st = SegTree<V, Int>(+, a)                      // bare operator
var mx = SegTree<V, Node>(Monoid(Node.ID, merge), a)// custom monoid
var lz = LazySegTree<V, Int>(AddSum, a)             // range add + range sum

let aff = Action(
    (1, 0),
    (new_f, old_f) => (new_f.0 * old_f.0, new_f.0 * old_f.1 + new_f.1),
    (f, val, span) => f.0 * val + f.1 * span,
    over: +,
)
var affine = LazySegTree<V, Int>(aff, a)
```

`Fenwick` is intentionally sum-only; use `SegTree` for custom associative operations. `SegTree` and `LazySegTree` both have `.all` (an O(1) fold of the whole domain) and `.max_right(l, ok)` / `.min_left(r, ok)` for O(log n) segment-tree binary search. Empty and inverted ranges are valid no-ops everywhere except `SparseTable.fold`, which asserts a non-empty range and returns a plain `T`. `Dsu<V>()` includes `.mark()` / `.undo_to(snap)` for rollback DSU out of the box.

## Graphs, `Rooted<V>`, and flows

`Graph<V>` and `Graph<V, W>` pack into flat CSR storage (`Graph.undirected(es)`, `Graph.directed(es)`, `g[u]`). Unweighted algorithms accept either a `Graph` or a plain neighbor function `V -> Iter<V>`: `bfs`, `dijkstra`, `topo`, `scc`, `condense`, `two_edge_components`, `biconnected_components`, `articulation_points`, and `two_color`.

`Rooted<V>(g, root)` unifies rooted tree queries, lazy LCA, Euler tours, virtual trees, and heavy-light path operations into one structure:

```cp CP Lang v2.2
let tree = Rooted(g, root)
var seg = LazySegTree<tree.Pre, Int>(AddSum, 0)

tree.apply_path(a, b, mut seg, delta)
let ans = tree.fold_path(a, b, seg)
let sub_sum = seg.fold(tree.sub(u))
```

`Rooted<V>` also provides `.parent(v)`, `.children(v)`, `.depth(v)`, `.size(v)`, `.preorder`, `.postorder`, `.lca(a, b)`, `.dist(a, b)`, `.is_anc(a, b)`, `.ancestor(v, k)`, `.lift_while(v, pred)`, `.chains(a, b, edges?: Bool)`, and `.virtual_tree(keys)`.

Core `std` also ships `FlowGraph<Node>()` and `max_weight_matching<L, R>`, which pair with a payload-enum `Node`, and the string helpers `z_function`, `prefix_function`, `SuffixArray`, and `RollingHash`.

## Modular arithmetic, math, and geometry

One `Mod<P>` type works for both compile-time `const` moduli and runtime-frozen `let` moduli:

```cp CP Lang v2.2
const MOD = 998_244_353
type Z = Mod<MOD>

let p: 1..=1e9 = read()
type DynamicZ = Mod<p>
```

`Mod<P>` packs into 4 bytes when `P <= 2^31 - 1`, supports `+`, `-`, `*`, `/`, `**`, `.pow(e)`, `.inv`, and intentionally does not implement `Ord`. `.try_inv() -> ?Z` handles a composite modulus. Core math also includes `Comb<P>()` (`.fact`, `.inv_fact`, `.choose`, `.arrange`; an immutable `let comb = Comb<MOD>()` grows its tables lazily), `Pt<T = Int>` (`.dot`, `.cross`, `.norm2`, `turn`, `turn_eps`, `ByAngle`), `gcd`, `lcm`, `isqrt`, `ceil_div`, `pow_mod`, `Sieve(n)`, `factor`, and `divisors`.

## Core `std` vs. local libraries (`examples/local-lib/`)

Specialized contest structures—`MonoDeque`, `Steps`, `TwoSat`, `Matrix`, `MinCostFlow`, `Dir8`, `palindromes`, `shortest`, `walk_back`, `mst`, `convex_hull`, `Rope`, `RangeKth`, `LiChao`, `PotentialDsu`, `Cover`, `Line`/`Hom`, `TwoStackFold`, `mo`, `Poly`/NTT, `XorBasis`, `Frac`, `Fixed`, `Tri`, and `Counter`—live in `examples/local-lib/` rather than core `std`; v2.2 moved the first eleven there because each had at most one use across the corpus. `std.marks.Marks<D>` (32-bit stamps) is opt-in; the default visited table is `[D]Bool` with a touched-list reset. Because CP Lang keeps the language core general, a local library compiles to the exact same zero-overhead C++17 as `std` and is pinned by lockfile for reproducible submissions.
