The standard library is written in ordinary CP Lang rather than hardwired into the compiler. Every container shares one uniform vocabulary — `.len`, `.is_empty`, `.clear()`, `.take()` (an `O(1)` move-out leaving the container empty) — and compiles down to flat, tree-shaken C++17.

The prelude auto-imports common types and helpers, including `bsearch_first`, `bsearch_last`, `bfs`, `topo`, `peel`, and `resized`; the examples below need no imports for those names.

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
sort(mut items, by: x => (x.cost, desc(x.value)))
```

- **`Set<T>()` / `SortedBag<T>()`:** `.add(x)`, `.remove(x)`, `x in s`, `.first`, `.last`, `.first_ge(k)`, `.first_gt(k)`, `.last_le(k)`, `.last_lt(k)`. A `Set` stores unique keys; a `SortedBag` keeps duplicates and provides `.count(x)` and `.remove_one(x)` to delete one copy. Neighbor lookups return `?K`, so an absent key is `none` rather than a sentinel.
- **`Map<K, V>()` / `HashMap<K, V>()`:** `m[k]` (returns `?V` or traps on direct index depending on lookup; `m.get(k) -> ?V` never inserts), `m[k] = v`, `m[k] f= x` (auto-inserts from the operator identity: `0` for `+=`/`-=`, `none` for `min=`/`max=`), `(m[k] ??= init)` (lazy initialize and lend mutably), `m.remove(k) -> ?V`, `k in m`, `for (k, v) in m.items`. Ordered `Map` lookups (`.first_ge(k)`, `.last_le(k)`) return `?(K, V)`. All hash tables use per-process splitmix64 salting to defeat anti-hash tests. `m.get_or_insert(k, v)` stores `v` when `k` is absent and returns the stored value.
- **`Bitset<D>()` & `Subset<D>`:** `Bitset<D>` provides `.add(v)`, `.remove(v)`, `.toggle(v)`, `v in s`, `|`, `&`, `-`, `^`, `.len` (popcount), `.first_set -> ?D`, `s.shifted(k)`. `Subset<D>` (`D.K <= 25`) is a 4-byte `Index` value for bitmask DP: `Subset<D>.empty`, `Subset<D>.full`, `Subset<D>.single(v)`, `s.first -> ?D`, `s.with(v)`, `s.without(v)`, `s.complement`, `for mask in Subset<D>`, `for v in mask`, and `for sub in mask.subsets`. `s = s.set(v, on)` returns a copy with `v` put in or taken out by a `Bool`; on its own, `s.set(v, on)` leaves `s` unchanged.

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
- `prefix(a)` — 1D or 2D prefix sums (`pre.sum(l..=r)`).
- `scan(a, +)` / `scan(a, max)` — inclusive running folds; `fold(+, a)` — the whole fold; `a.excl_fold(+)` — yields `(x, rest)`, the fold of every other element, without division.
- `runs(a)` — run-length `(value, count)` pairs; `runs_by(a, key)` — groups of equal `key`, each `(key, items)`; `xs.adjacent` — consecutive pairs; `D.windows(k)` — sliding windows.
- `sort(mut a, by:)`, `sorted(a, by:)` (descending via `by: desc`), `reverse(mut a)`, `dedup(mut a)`, `distinct(a)`, `rotate_left(mut a, k)` (negative rotates right), `next_permutation(mut a)`, `prev_permutation(mut a)`.
- `sum`, `any`, `all`, `count`, `first(it) -> ?T`, `a.find(x) -> ?I`, `min_by(it, key)`, `max_by(it, key)`.
- `Perm<V>` (`.inv()`, `.cycles()`), plus deterministic RNG: `Rng(seed)`, `rand(lo..hi)`, `shuffle(mut xs)`.

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
- `Dsu<V>()` provides `.unite(a, b) -> Bool`, `.find(v) -> V`, `.same(a, b)`, `.size(v)`, `.components`, `.groups() -> List<List<V>>`, plus `.mark()` and `.undo_to(snap)` for `O(1)` rollback DSU.
- `std.marks.Marks<D>` is an opt-in 32-bit generation-stamp visited table (`O(1)` reset); for most problems, `[D]Bool` with a touched-list reset is simpler and equally fast.

## Graphs, trees, flows, and strings

- **`Graph<V>` / `Graph<V, W>` (`std.graph`):** flat CSR storage via `Graph.undirected(es)`, `Graph.directed(es)`, and neighbor iteration `g[u]`. Algorithms accept either a `Graph` or an implicit neighbor function `v => ...`: `bfs`, `dijkstra`, `topo`, `scc`, `condense`, `two_edge_components`, `biconnected_components`, `articulation_points`, and `two_color`.
- **`Rooted<V>(g, root)` / `Rooted.from_parents(par)` (`std.tree`):** value-owned Euler preorder domain `t.Pre`, `t.pos(v) -> t.Pre`, `t.preorder: [t.Pre]V`, `t.postorder`, `t.tour` (`(v, entering)`), `.parent(v)`, `.children(v)`, `.depth(v)`, `.size(v)`, `.lca(a, b)`, `.dist(a, b)`, `.is_anc(a, b)`, `.ancestor(v, k)`, `.lift_while(v, pred) -> ?V`, `.sub(v) -> Range<t.Pre>`, `.chains(a, b, edges?: Bool)`, `.fold_path(a, b, ds)`, `.apply_path(a, b, mut ds, tag)`, and `.virtual_tree(keys)`.
- **Network flow (`std.flow`):** `FlowGraph<Node>()` (Dinic) and `max_weight_matching<L, R>` (Hungarian), designed to pair directly with payload enums (`enum Node: Src; Sink; Left(L); Right(R)`).
- **String algorithms (`std.strings`):** `z_function(s)`, `prefix_function(s)`, `SuffixArray(s)` (`.sa`, `.lcp`), and `RollingHash(seq)` (works on `Str` or any `[D]T`, `.get(l..r) -> HashVal`).

## Modular arithmetic, math, and geometry

One `Mod<P>` type handles both compile-time `const` moduli and runtime-frozen `let` moduli:

```cp CP Lang 0.2.2
const MOD = 998_244_353
type Z = Mod<MOD>

let p: 1..=1e9 = read()
type DynamicZ = Mod<p>
```

- **`Mod<P>` (`std.mod`):** 4-byte storage when `P <= 2^31 - 1`, `+`, `-`, `*`, `/`, `**`, `.pow(e)`, `.inv`, `.try_inv() -> ?Z` (for composite moduli), `.val -> Int`. Intentionally omits `Ord`.
- **`Comb<P>()` (`std.mod`):** `.fact(n)`, `.inv_fact(n)`, `.choose(n, k)`, `.arrange(n, k)` (an immutable `let comb = Comb<MOD>()` extends its tables lazily).
- **Number theory (`std.math`):** `gcd`, `lcm`, `ext_gcd`, `isqrt`, `ceil_div`, `pow_mod`, `Sieve(n)` (`.is_prime`, `.spf`, `.primes`), `factor(n)`, `divisors(n)`.
- **Exact geometry (`std.geometry`):** `Pt<T = Int>` with `.dot`, `.cross`, `.norm2`, `orient(a, b, c)` (`-1 | 0 | 1`), `turn`, `turn_eps`, and `ByAngle` for exact integer angular sorting.

## Local libraries & advanced contest structures

Core `std` is intentionally small enough to memorize. Specialized algorithms live in **local libraries** — regular folders of `.cp` files with a `lib.toml` manifest that compile and tree-shake identically to `std`.

The repository includes a full sample library (`examples/local-lib/`) covering advanced contest topics:

- **Data structures (`ds/`):** `MonoDeque`, `Steps`, `Rope` (implicit treap), `RangeKth` (persistent segtree), `LiChao` & `LiChaoArena`, `Tri<D>` (triangular interval index), `PotentialDsu`, `BipartiteDsu`, `Cover` (union-of-intervals), `TwoStackFold`, `mo_order`, `XorBasis`, `Interner`, and `RollbackArray`.
- **Math & polynomials (`math/`):** `Frac`, `Fixed<D>`, `Congruence` (general CRT), `Hom` / `BothWays`, `Mat<P>` (Gaussian elimination / `det` / `rank`), `Poly` / NTT / 3-prime MTT / Lagrange interpolation, `Powers`, SOS DP (`sos_sub`/`sos_sup`), Miller-Rabin + Pollard's rho (`primes`), and `floor_sum`.
- **Graphs & DP (`graph/`, `dp/`, `geom/`, `str/`, `seq/`):** `TwoSat`, `MinCostFlow`, `Functional` graphs, `Centroids` & `reroot`, `KruskalTree`, Hopcroft-Karp `max_matching`, `bounded_flow`, `mst`, `shortest`/`walk_back`, `convex_hull`, `Dir8`, divide-and-conquer DP (`dnc_opt`), Knuth DP (`knuth`), `palindromes` (Manacher), `Trie`, `SuffixAutomaton`, `AhoCorasick`, `Hash2D`, `lis`, and `fold_others`.

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
