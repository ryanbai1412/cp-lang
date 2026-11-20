Canonical one-page contest reference for the locked **CP Lang v2** specification ([architecture](https://github.com/ryanbai1412/cp-lang/blob/main/research/language-architecture-v2.md), [decisions §16–§17](https://github.com/ryanbai1412/cp-lang/blob/main/decisions.md)). Every construct has one spelling.

## 1. Bindings, numbers & operators

```cp CP Lang v2
let n = 10                      // immutable binding
var ans = 0                     // mutable binding
const MOD = 1_000_000_007       // compile-time constant (inlined)
```

- **Four numeric types:** `Int` (64-bit signed symmetric `-(2^63-1)..=(2^63-1)`), `U64` (wrapping 64-bit unsigned), `I128` (128-bit signed), `F64` (64-bit IEEE double). Range types like `0..=1_000_000_000` pack tightly (1/2/4 bytes) and check bounds on entry.
- **`F64` rules:** `1e18` is `Int`; `.` marks `F64` (`0.0`). Integer literals unify with `F64`; `Int` variables need `F64(x)`. `Int(x)` on `F64` is an error — write `floor(x)`, `ceil(x)`, `round(x)`, or `trunc(x)`. NaN and infinity trap in local runs, making `<` total. Format floats with `"{x:.6f}"` (never `-0.00`); compare with `close(a, b, eps)` and `sign(x, eps)`. Intrinsics: `sqrt`, `sin`, `cos`, `tan`, `atan2`, `exp`, `ln`, `hypot`, `PI`.
- **Scalars:** `Bool` (`true`, `false`), `Char` (1 ASCII byte, `'a'`), `Str` (value-semantic `0..len`-indexed ASCII bytes, zero UTF-8).
- **Blocks & comments:** `// line comment` only. Indentation defines blocks; no semicolons. Lines continue after binary ops, `=`, or inside open brackets.
- **Arithmetic & bitwise:** `+`, `-`, `*`, `/` (Euclidean floor division), `%` (Euclidean, always in `0..|b|`), `**` (power); `&`, `|`, `^`, `~`, `<<`, `>>` and compound assignments; `popcount(x)`, `ctz(x)`, `clz(x)`, `bit_len(x)`.
- **Logic & fold updates:** `and`, `or`, `not`, `==`, `!=`, `<`, `<=`, `>`, `>=`, `x += y`, `x min= y`, `x max= y` (any 2-argument function works as `x f= y`).

## 2. Optionals (`?T`)

No `INF` or `-1` sentinels. `?Int` (`LLONG_MIN`), `?F64` (quiet NaN), `?V`, `?Bool`, `?Char`, and `?Enum` are niche-packed with zero extra memory.

```cp CP Lang v2
var dist: [V]?Int = none        // none or bare value `dist[src] = 0`
let d = dist[v] ?? -1           // fallback on none (parenthesize inside `+`: `a + (x ?? 0)`)
let best = min(a, b)            // min / max skip none on either side; both none => none
dist[v] min= d + w              // updates if dist[v] is none or d + w < dist[v]!
ans min= dist[v]                // skips when dist[v] is none even if `ans` is plain Int
let x = opt!                    // unwrap (traps on none; elided when flow-proven)
let y = maybe()?                // early-return none from fn returning ?T
(adj[k] ??= []).push(x)         // initialize place lazily if none, then lend mutably
let below = if let p = h[i] - 1: st.fold(..=p) else: 0
```

- **Uniform optionality:** `v + k`, `v - k`, `xs.last`, and `min(gen)`/`max(gen)` always return `?T`.
- **No implicit `?T` or `Mod<P>` ordering:** `?T` and `Mod<P>` do not implement `Ord`. Place `none` explicitly when sorting: `sort(mut xs, by: x => (x.d == none, x.d ?? 0))`.

## 3. Domains, indices, boundaries, ranges & grids

```cp CP Lang v2
domain V = 1..=n                // nominal bounded index: 1..=n (0-cost in C++)
let a: [V]Int = read()          // dense array indexed by V (zero checks on a[v])
var dp: [Row, Col]Int = 0       // 2D array indexed by (Row, Col); g[p] works for p: (Row, Col)
var cnt: [0..=k]Int = 0         // structural plain array indexed by Int (same bounds = same type)
cnt.fill(0)                     // whole-array reset (also `seen.fill(none)` for [D]?T)
let b: [V]Int = [V] v => a[v] * 2
```

- **Structural range vs. `domain`:** `[0..n]T` / `[0..=n]T` is a plain `Int`-indexed array whose type is structural (`for i in 0..n` gives `i: Int`). Name a `domain V = 1..=n` when you want zero-check nominal index safety (`for v in V` gives `v: V`, so `v + 1` is `?V`).
- **`Index` & `Seq`:** `V.K` (count), `V.first`, `V.last`, `v.to_idx` (`0..V.K`), `V.from_idx(i) -> ?V`, `v + k -> ?V`, `v - k -> ?V`, `b - a -> Int`, `v.cyc(k) -> V`.
- **Boundaries (`V.bounds`, `V.K + 1` gaps):** `v.l` / `v.r` (gap left/right of `v`), `v.l(k)` / `v.r(k)` (`k`-element window edges counting `v`, `v.l == v.l(1)`), `b.left -> ?V`, `b.right -> ?V`.
- **One `Range<I>` type:** `l..r` (half-open), `l..=r` (inclusive), `..i`, `..=i`, `i..`, `i.r..`, `l.r..r.l`. Members: `.first -> ?I`, `.last -> ?I`, `.mid -> ?I`, `.len`, `.is_empty`, `.rev`, `.step(k)`, `(lo..=hi).clamp(D)`. Range structures accept open and empty ranges (`st.fold(..h[i])`, `pre.sum(..i)`).
- **Slices & grids (`std.grid`):** `a[l..=r]` (mutable slice; `sort(mut a[l..=r])`, `reverse(mut a[l..=r])`), `g.row(r)`, `g.col(c)`, `g.transposed`, `g.flipped`, `swap(mut g.row(r1), mut g.row(r2))`. Product-domain `type Cell = (Row, Col)` indexes `[Row, Col]T` directly (`g[p]`) and works with `[Cell]T`, `bfs`, `Dsu<Cell>`, and `@memo`, with `Dir4`/`Dir8` (`.cw`, `.ccw`, `.back`, `Dir4.from("UDLR", ch)`), `p.step(d)`, `p.ray(d)`, `p.adj4`, and `p.adj8`.
- **Algebraic `Index` (5 O(1) `Copy` shapes):** `Bool`, unit/payload `enum`, `?T`, `lo..=hi`, and tuples/records `(A, B)`, plus `Subset<D>` (`D.K <= 25`).
- **Escape hatches:** `a.raw[i]` (0-based `Int` indexing, checked in debug) and `unchecked:` blocks (elide bounds/overflow traps in judge builds).

## 4. Strings (`Str`)

Default `Str` is a flat, value-semantic, `0..len`-indexed ASCII byte sequence (zero UTF-8, O(1) indexing and slicing).

```cp CP Lang v2
let s: Str = read()
let ch = s[i]                   // Char (0..s.len); digit: ch - '0', letter: ch - 'a'
let sub = s[l..r]               // O(1) slice; also s[..k], s[k..], s.len, s.is_empty
var out = ""
out.push('a')
out += "bc" * 3                 // mutable byte buffer when `var`; Str(chars) from List<Char>
if s.starts_with(x, at: i): ... // s.split(' '), s.split_once(' '), s.trim_start(' '), s.parse_int()
let joined = " ".join("{x}" for x in ans)
let a: [domain A]Char = read()  // opt-in nominal character domain for multi-string DP
```

## 5. Control flow, parallel assignment & `defer`

```cp CP Lang v2
if x < 0: neg()
elif x == 0: zero()
else: pos()

let sign = if x >= 0: 1 else: -1          // one inline-if syntax
var (r0, r1) = (a, b)                     // declares mutable vars
(r0, r1) = (r1, r0 - q * r1)              // parallel tuple assignment (evaluates RHS first)

for v in V if ok(v): ...                  // filtered loop
for (i, x) in a.items: ...                // (index/key, value) everywhere
for i in (0..n).step(2).rev: ...
while let x = q.pop() and x > 0: ...
'outer: for r in Row:
    for c in Col:
        if bad(r, c): break 'outer        // labeled break / continue

let res = match op:                       // exhaustive expression; bare variants, `=>`
    Add(l, r, v) => st.apply(l..=r, v)
    Query(l, r) => st.fold(l..=r)

let snap = dsu.mark()
defer dsu.undo_to(snap)                   // one cleanup primitive
```

## 6. Functions, generics, closures & ownership

```cp CP Lang v2
fn swap_adj(mut a: [V]Int, i: V):         // mut on param AND at call site
    if let j = i + 1: swap(mut a[i], mut a[j])

swap_adj(mut a, v)

var vis: [V]Bool = false
fn dfs(u: V):                             // plain nested fn mutates outer var directly
    vis[u] = true
    for v in g[u]: if not vis[v]: dfs(v)

sort(mut es, by: e => (e.w, -e.u))        // one lambda syntax: `x => expr`
```

- **Generics & inference:** Write `<D>` explicitly on generic definitions (`fn f<D>(a: [D]Int)`). Call sites unify non-closure arguments (`from:`, `root:`, `g`) before closures; use `f::<T>()` only for return-polymorphic calls like `parse::<Int>(tok)`.
- **Ownership:** Small values (`<= 16 B`, no heap) are `Copy`. Heap values (`Array`, `List`, `Map`, `Str`) have strict value semantics — move on last use, copy explicitly with `.copy()`. Destructuring `let (r, c) = q[head]` copies/owns values. No stored or returned references; `mut` borrows and `lends` projections stay lexical.

## 7. Records, structs & enums

`Eq`, `Ord` (when every field is `Ord`, never `?T` or `Mod<P>`), `Hash`, `Show`, `Read`, `Gen`, `Copy`, and `Index` derive automatically.

```cp CP Lang v2
type Edge = (u: V, v: V, w: Int)          // structural named record; e.u or let (u, v, w) = e

struct SegNode:
    best: Int = 0
    cnt: Int = 1
    fn merge(self, o: SegNode) -> SegNode: ...
    fn reset(mut self): self.best = 0

enum Dir = N | E | S | W                  // unit enum (auto Index, K = 4)

enum Node:                                // payload enum (auto sum Index on demand)
    Source
    Sink
    Build(s: Station)
    Serve(u: User)
```

## 8. I/O, `@multitest`, `@memo` & verification

```cp CP Lang v2
@multitest                                // also @multitest(until_eof), @multitest(count: t)
let (n, m): (1..=2e5, 1..=n) = read()     // dependent bounds on scalars, tuples, record fields
let g: [Row, Col]?Int = read(none: "X")   // symmetric `none:` on read / print / read_output
print(ans, none: "IMPOSSIBLE")
for (r, c) in read_until((0, 0)): ...     // sentinel-terminated stream
assume n >= 1 and m <= 200_000            // local debug + stress filter (stripped on judge)
assert ok                                 // always checked (local + judge)
debug(n, dp)                              // stderr locally (replayed on stress fail); stripped on judge

@tags('.', '*', '1'..='9')
enum Cell: Empty | Wall | Coin(val: 1..=9)

@memo                                     // flat table for algebraic Index <= 2^26, else salted HashMap
fn dp(u: V, used: Bool) -> Int: ...       // @memo(flat) or @memo(hash) to override

@gen fn make_case(size: Int): ...         // whole-input generator (choice-stream shrinking)
@brute(main) fn slow(): ...               // default oracle for `cpl stress` (stripped on judge)
@checker fn check(): ...                  // reads solution stdout via read_output(), checks with assert
@interactor fn judge(): ...               // local stdio judge paired with `ask("?", x, limit: 2)`
@grader fn restore(n: Int) -> List<Int>:  // IOI/DMOJ grader entry
```

## 9. Collections

Every container uses `.len`, `.is_empty`, and `.clear()`. Construct empty `List`/`Deque` with `= []`; construct `MinHeap<T>()`, `MaxHeap<T>()`, `Set<T>()`, `SortedBag<T>()`, `HashSet<T>()`, `Map<K, V>()`, `HashMap<K, V>()`, `Bitset<D>()`, `Dsu<V>()` with `Type()`.

- **`List<T>`:** `var xs: List<T> = []`, `.push(x)`, `.push_id(x) -> Int`, `.pop() -> ?T`, `.first`, `.last`, `xs[l..r]`, `xs + ys`, `xs.freeze() -> (domain D, [D]T)`.
- **`Deque<T>`:** `var q: Deque<T> = []`, `.push(x)`, `.pop() -> ?T`, `.push_front(x)`, `.pop_front() -> ?T`, `.first`, `.last`, `q[i]`.
- **`MonoDeque<T>(min | max)`:** imperative monotone queue — `.push(at: Int, val: T)`, `.drop_before(min_at: Int)`, `.clear()`, `.best -> ?T`.
- **`MinHeap<T>()` / `MaxHeap<T>()`:** `.push(x)`, `.pop() -> ?T`, `.peek -> ?T`. Use `desc(x)` inside tuples for mixed order, e.g. `MinHeap<(Int, Desc<V>)>`.
- **`Set<T>()` / `SortedBag<T>()` / `HashSet<T>()`:** `.add(x)`, `.remove(x)`, `x in s`, `|`, `&`, `-`, `^`. Ordered sets also expose `.first`, `.last`, `.pop_first()`, `.pop_last()`, `.first_ge(k)`, `.first_gt(k)`, `.last_le(k)`, `.last_lt(k)`, `.rank(k)`, `.nth(i)`.
- **`Map<K, V>()` / `HashMap<K, V>()`:** `m[k]`, `m[k] f= x`, `m[k] ??= init`, `m.get(k) -> ?V`, `m.remove(k)`, `k in m`, `for (k, v) in m.items`.
- **`Bitset<D>()` & `Subset<D>` (`Copy` `Index`, `D.K <= 25`):** `.add(v)`, `.remove(v)`, `.toggle(v)`, `v in s`, `|`, `&`, `-`, `^`, `.len`, `s.shifted(k)`. `Subset<D>` also supports `for g in s.subsets`, `s.with(v)`, `s.without(v)`.

## 10. Search, sequences & math prelude

- **Binary search (free functions on ranges, methods on domains):** `bsearch_first(r, ok) -> ?I`, `bsearch_last(r, ok) -> ?I`, `bsearch_first_some(r, f) -> ?(I, W)`. (`F64` ranges take `steps: 80`.)
- **Sorted & linear lookup:** `xs.first_ge(x) -> ?I`, `xs.last_le(x) -> ?I`, `first(it) -> ?T`, `a.find(x) -> ?I`, `min_by(it, key)`, `max_by(it, key)`.
- **Sliding & sequences:** `D.windows(k)`, `sliding_min(a, k)`, `sliding_max(a, k)`, stateless `slide_until(r, (l, r) => ok)`, `sort(mut a, by:)`, `sorted(a, by:)`, `reverse(mut a)`, `dedup(mut a)`, `rotate_left(mut a, k)`, `next_permutation(mut a)`, `let xs = compress(all)` (domain `xs.D`, value `xs[v]`, lookup `xs.at(val) -> xs.D` and `xs.first_ge(val)`), `prefix(a)` (`pre.sum(l..=r)` in 1D/2D), `scan(a, +)` / `scan(a, max)`, `runs(a)`, `xs.adjacent`, `a.items`, `sum(it)`, `any(it)`, `all(it)`, `count(it)`, `Perm<V>` (`.inv()`, `.cycles()`).
- **Math & geometry:** `abs`, `min`, `max`, `gcd`, `lcm`, `isqrt`, `pow_mod(a, e, m)`, `sieve(n)`, `factor(x)`.
  - **`Mod<P>`:** unified for compile-time `const` (`type Z = Mod<MOD>`) and runtime-frozen `let` (`let p: 1..=1e9 = read(); type Z = Mod<p>`).
  - **`Comb<P>(n)`:** `.fact(k)`, `.inv_fact(k)`, `.choose(n, k)`, `.arrange(n, k)`.
  - **`Matrix<R, C, T>`:** `a * b`, `a.pow(e)`, `a.mul_with(b, add:, mul:, zero:)`.
  - **`Pt<T = Int>`:** `+`, `-`, `a.dot(b)`, `a.cross(b)`, `a.norm2`, `turn(a, b, c)`, `turn_eps(a, b, c, eps)`, `unit(angle)`, `ByAngle`, `convex_hull(pts)`.

## 11. Core data structures & graphs (`std`)

- **Range structures** (accept bare operators `+`, `*`, `^`, `&`, `|`, `min`, `max`, `gcd` or `Monoid(id, op)`):
  - `Fenwick<D, Int>(a)` / `Fenwick2D<R, C, Int>()` — sum-only: `.add(i, d)`, `.sum(l..=r)`.
  - `SparseTable<D, Int>(min, a)` — `.fold(l..=r)` in O(1).
  - `SegTree<D, T>(+, a)` or `SegTree<D, T>(Monoid(id, op), a)` — `.set(i, v)`, `.fold(l..=r)`, `.max_right(from, ok)`, `.min_left(to, ok)`.
  - `LazySegTree<D, T>(AddSum, a)` (stock tags: `AddSum`, `AddMin`, `AddMax`, `AssignSum`, `AssignMin`, `AssignMax`, or custom `Action(id, compose, apply)`) — `.apply(l..=r, tag)`, `.fold(l..=r)`, `.max_right(from, ok)`, `.min_left(to, ok)`.
  - `Steps<K, V>(init)` — piecewise-constant intervals (`Steps<K, Bool>` for interval sets).
  - `Dsu<V>()` — `.unite(a, b) -> Bool`, `.find(v) -> V`, `.same(a, b)`, `.size(v)`, `.components`, `.mark()` / `.undo_to(snap)`.
- **Graphs & trees** (`from:`/`root:`/`g` first for inference; unweighted algorithms accept `Graph<V, E>` or `V -> Iter<V>`):
  - `Graph<V>` / `Graph<V, W>` — `Graph.undirected(es)`, `Graph.directed(es)`, `g[u]`, `bfs`, `dijkstra`, `shortest`, `topo`, `topo_prefix`, `scc`, `condense`, `two_edge_components`, `biconnected_components`, `articulation_points`, `two_color`, `mst`.
  - `Rooted<V>(g, root)` — DFS tree/forest, lazy LCA, `.parent(v)`, `.children(v)`, `.depth(v)`, `.size(v)`, `.preorder`, `.postorder`, `.lca(a, b)`, `.dist(a, b)`, `.is_anc(a, b)`, `.ancestor(v, k)`, `.lift_while(v, pred)`, `.sub(v) -> Range<Pre>`, `.chains(a, b, edges?: Bool)`, `.fold_path(a, b, ds)`, `.apply_path(a, b, mut ds, tag)`, `.virtual_tree(keys)`.
  - `FlowGraph<Node>()` / `MinCostFlow<Node>()` / `max_weight_matching<L, R>` — pair with payload-enum `Node`.
  - `TwoSat<V>()` — `.add_or(a, b)`, `.implies(a, b)`, `.solve() -> ?[V]Bool`.
- **Strings:** `z_function(s)`, `prefix_function(s)`, `palindromes(s)`, `SuffixArray(s)` (`.sa`, `.lcp`), `RollingHash(s)` (`.get(l..r)`).
- *(Advanced structures — `Rope`, `RangeKth`, `LiChao`, `PotentialDsu`, `Cover`, `Line`/`Hom`, `TwoStackFold`, `mo`, `Poly`/NTT, `XorBasis`, `Frac`, `Fixed`, `Tri`, `Counter` — live in `examples/local-lib/` with identical zero-cost speed.)*

## 12. Reserved keywords (40 total)

- **Contestant (32, including 3 literals):** `let`, `var`, `const`, `fn`, `struct`, `enum`, `domain`, `type`, `use`, `pub`, `if`, `elif`, `else`, `for`, `while`, `in`, `match`, `return`, `break`, `continue`, `defer`, `and`, `or`, `not`, `assert`, `assume`, `mut`, `self`, `unchecked`, `true`, `false`, `none`.
- **Library-author only (8):** `trait`, `impl`, `where`, `lends`, `lend`, `law`, `inline`, `yield`.
