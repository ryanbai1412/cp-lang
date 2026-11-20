Canonical one-page contest reference for the locked **CP Lang v2.1** specification ([architecture](https://github.com/ryanbai1412/cp-lang/blob/main/research/language-architecture-v2.1.md), [decisions §16–§18](https://github.com/ryanbai1412/cp-lang/blob/main/decisions.md)). Every construct has one spelling.

## 1. Bindings, numbers & operators

```cp CP Lang v2.1
let n = 10                      // immutable binding
var ans = 0                     // mutable binding
const MOD = 1_000_000_007       // compile-time constant (inlined)
```

- **4 numeric types:** `Int` (64-bit signed symmetric `-(2^63-1)..=(2^63-1)`), `U64` (wrapping 64-bit unsigned), `I128` (128-bit signed), `F64` (64-bit IEEE double). Range types like `0..=1_000_000_000` pack tightly (1/2/4 bytes) and check bounds on entry.
- **`F64` rules:** `1e18` is `Int`; `.` marks `F64` (`0.0`). Integer literals unify with `F64`; `Int` variables need `F64(x)`. `Int(x)` on `F64` is an error — write `floor(x)`, `ceil(x)`, `round(x)`, or `trunc(x)`. NaN and infinity trap in local runs, making `<` total. Bare `print(x)` of `F64` is an error — format with `"{x:.6f}"` (never `-0.00`). `==`/`!=` lint toward `close(a, b, eps)` and `sign(x, eps)` (exempting `== 0.0`). Intrinsics: `sqrt`, `sin`, `cos`, `tan`, `atan2`, `exp`, `ln`, `hypot`, `PI`.
- **Scalars & casts:** `Bool` (`true`, `false`), `Char` (1 ASCII byte, `'a'`), `Str` (`0..len`-indexed ASCII bytes, zero UTF-8). Lossless casts: `Int(ch)` (`0..=127`), `Char(i)`, `Int(b)` (`0 | 1`). Hex `0xFF` & binary `0b1010` literals.
- **Comments & blocks:** `// line comment` only. Indentation defines blocks; no semicolons. Lines continue after binary ops, `=`, or inside open brackets.
- **Arithmetic & ranges:** `+`, `-`, `*`, `/` (Euclidean floor division), `%` (Euclidean, always in `0..|b|`), `**` (power). `..` / `..=` bind looser than arithmetic and tighter than comparisons/`in` (`i + 1..=n` is `(i + 1)..=n`).
- **Bitwise:** `&`, `|`, `^` (XOR), `~` (NOT), `<<`, `>>`, plus `&=`, `|=`, `^=`, `<<=`, `>>=`. Helpers: `popcount(x)`, `ctz(x)`, `clz(x)`, `bit_len(x)`.
- **Logic & updates:** `and`, `or`, `not`, `==`, `!=`, `<`, `<=`, `>`, `>=`, `x += y`, `x min= y`, `x max= y` (any 2-arg fn works as `x f= y`).

## 2. Optionals (`?T`)

No `INF` or `-1` sentinels. `?Int` (`LLONG_MIN`), `?F64` (quiet NaN), `?V`, `?Bool`, `?Char`, and `?Enum` are niche-packed with zero extra memory.

```cp CP Lang v2.1
var dist: [V]?Int = none        // none or bare value `dist[src] = 0`
let d = dist[v] ?? -1           // fallback on none (mixing `??` with `+ - * / %` without parens is an error)
let best = min(a, b)            // min / max skip none on either side (incl. min(gen)/max(gen) over ?T)
dist[v] min= dist[u] + w        // tropical: `+ - += -=` & `sum` on ?Int/?F64 absorb none; min= skips none
ans min= dist[v]                // skips when dist[v] is none even if `ans` is plain Int
if dq.last != x: ...            // `==` / `!=` auto-lift T to ?T (never `<` / `<=` / `>` / `>=`)
let x = opt!                    // unwrap (traps on none; elided when flow-proven)
let y = maybe()?                // early-return none from fn returning ?T
(adj[k] ??= []).push(x)         // initialize place lazily if none, then lend mutably
if let a = x and let b = y: ... // chained optional bindings (also `if let` as expression)
```

- **Uniform optionality:** `v + k`, `v - k`, `xs.last`, `min(gen)`/`max(gen)`, `min_by`/`max_by`, and `SparseTable.fold` always return `?T`.
- **No implicit `?T` or `Mod<P>` ordering:** `?T` and `Mod<P>` do NOT implement `Ord` (`a < b` is an error, and structs/tuples with `?T` or `Mod<P>` fields compile cleanly without deriving `Ord`). State where `none` belongs explicitly when sorting: `sort(mut xs, by: x => (x.d == none, x.d ?? 0))`. Use `match (a, b):` to combine custom `?Struct`s.

## 3. Domains, indices, boundaries, ranges & grids

```cp CP Lang v2.1
domain V = 1..=n                // nominal bounded index: 1..=n (0-cost in C++)
let a: [V]Int = read()          // dense array indexed by V (zero checks on a[v])
var dp: [Row, Col]Int = 0       // 2D array indexed by (Row, Col); g[p] works for p: (Row, Col)
var cnt: [0..=k]Int = 0         // structural Int/Char-range array (`['a'..='z']Int`, independent fill per slot)
cnt.fill(0)                     // whole-array reset (also `seen.fill(none)`, `a.swap(i, j)` safe when i == j)
let b: [V]Int = [V] v => a[v] * 2   // comprehension over domain V (list comprehension: `[x * 2 for x in a if x > 0]`)
```

- **Anonymous range vs. `domain`:** `[0..n]T`, `[0..=n]T`, and `['a'..='z']T` are structural arrays (same written bounds unify across calls; `for i in 0..n` gives `i: Int`). `var a: [D]T = expr` clones `expr` independently per slot (`[D]List<T> = []` never aliases). Name a `domain V = 1..=n` for zero-check nominal safety (`for v in V` gives `v: V`, `v + 1` is `?V`).
- **Index (`V`) & Seq (`1..=n`):** `V.K` (count), `V.first`, `V.last`, `V[..k]` (first `min(k, V.K)` range), `v.to_idx` (`0..V.K`), `V.from_idx(i) -> ?V` (row-major on tuples `(R, C)`), `v + k -> ?V`, `v - k -> ?V`, `b - a -> Int`, `v.cyc(k) -> V`, `v.mirror -> V`.
- **Boundaries (`V.bounds`, `V.K + 1` gaps):** `v.l` / `v.r` (gap left/right of `v`), `v.l(k)` / `v.r(k)` (`v.l == v.l(1)`), `b.left -> ?V`, `b.right -> ?V`.
- **One `Range<I>` type:** `l..r`, `l..=r`, `..i`, `..=i`, `i..`, `i.r..`, `l.r..r.l`. Properties/methods: `.l`/`.r` (boundary cuts even if empty), `.first -> ?I`, `.last -> ?I`, `.mid -> ?I`, `.len`, `.is_empty`, `.rev`, `.mirror`, `.step(k)`, `r1 & r2` (intersection), `(lo..=hi).clamp(D)`. All range structures accept open/empty ranges (`st.fold(..h[i])`, `pre.sum(..i)`).
- **Slices & grids (`std.grid`):** `a[l..=r]` (mutable slice; `.sorted()`, `.first_ge`, `.last_le`, `sort(mut a[l..=r])`, `reverse(mut a[l..=r])`), `g.row(r)`, `g.col(c)`, `g.transposed` (`[C, R]T`), `g.flipped_rows`, `g.flipped_cols`, `swap(mut g.row(r1), mut g.row(r2))`. `[R, C]Char = read()` reads packed rows. `type Cell = (Row, Col)` indexes `[Row, Col]T` (`g[p]`) and works with `[Cell]T`, `bfs`, `Dsu<Cell>`, `@memo`, `Dir4`/`Dir8` (`.cw`, `.ccw`, `.back`, `Dir4.from("UDLR", ch)`), `p.step(d)`, `p.ray(d)`, **`p.adj4`**, **`p.adj8`**.
- **Algebraic `Index` (5 strictly O(1), `Copy` shapes):** `Bool` (`for b in Bool`, `[V, Bool]T`), unit/payload `enum` (on demand), `?T`, `lo..=hi` (`Int`/`Char`), and row-major tuples/records `(A, B)`, plus `Subset<D>` (`D.K <= 25`, `for s in Subset<D>`). (No `[D]E: Index`; no `Pool<T>` — use `type Id = 0..=MAX` + `List<Node>` + `.push_id(x)` + `.freeze()`.)
- **Escape hatches:** `a.raw[i]` (0-based `Int` indexing, checked in debug) and `unchecked:` block (elides bounds/overflow traps in judge builds).

## 4. Strings (`Str`)

Default `Str` is an `Int`-indexed (`0..s.len`, zero `.raw`) ASCII byte sequence (zero UTF-8, O(1) indexing and slicing).

```cp CP Lang v2.1
let s: Str = read()
let ch = s[i]                   // Char (0..s.len); digit: ch - '0', letter: ch - 'a', s.items
let sub = s[l..r]               // O(1) slice; also s[..k], s[k..], s.len, s.is_empty, match s:
var out = ""
out.push('a'); out += "bc" * 3  // mutable when `var`; s + t, s + ch, ch + s, Str(chars)
if s.starts_with(x, at: i): ... // also s.split(' '), s.split_once(' '), s.trim_start(' '), s.parse_int()
let joined = " ".join("{x}" for x in ans)
let a: [domain A]Char = read()  // opt-in nominal character domain for multi-string DP
```

## 5. Control flow, parallel assignment & `defer`

```cp CP Lang v2.1
if x < 0: neg()
elif x == 0: zero()
else: pos()

let sign = if x >= 0: 1 else: -1          // ONE inline-if syntax (no `then`)
var (r0, r1) = (a, b)                     // declares mutable vars
(r0, r1) = (r1, r0 - q * r1)              // parallel tuple assignment (evaluates RHS first)

for v in V if ok(v): ...                  // filtered loop
for mut b in blocks: b.query()            // mutable element iteration (for self-adjusting structs)
for (i, x) in a.items: ...                // (index/key, value) everywhere
for i in (0..n).step(2).rev: ...
while let a = x and let b = y and a > 0:  // chained optional bindings + guard
'outer: for r in Row:
    for c in Col:
        if bad(r, c): break 'outer        // labeled break / continue; bare `return` exits top level / @multitest

let res = match op:                       // exhaustive expression; bare variants, `=>`
    Add(l, r, v) => st.apply(l..=r, v)
    Query(l, r) => st.fold(l..=r)

let snap = dsu.mark()
defer dsu.undo_to(snap)                   // block-scoped LIFO (runs at end of enclosing loop iteration/block)
```

## 6. Functions, generics, closures & ownership

```cp CP Lang v2.1
fn swap_adj(mut a: [V]Int, i: V):         // mut on param AND at call site
    if let j = i + 1: swap(mut a[i], mut a[j])

swap_adj(mut a, v)

var vis: [V]Bool = false
fn dfs(u: V):                             // plain nested `fn` mutates outer `var` directly
    vis[u] = true
    for v in g[u]: if not vis[v]: dfs(v)

sort(mut es, by: e => (e.w, -e.u))        // ONE lambda syntax: `x => expr`
```

- **Generics & inference:** Always write `<D>` explicitly on generic definitions (`fn f<D>(a: [D]Int)`). Array parameters may size off sibling `Int` parameters left or right (`fn f(a: [0..r]Bool, r: Int)`). Call sites unify non-closure arguments (`from:`, `root:`, `g`) before closures; use `f::<T>()` for return-polymorphic or vertexless closure calls (`parse::<Int>(tok)`, `topo::<V>(next)`).
- **Ownership & evaluation:** Small values (`<= 16 B`, no heap) are `Copy`; reading a `Copy` element releases its borrow immediately before calls (`merge(hrc[u], v)`). Indexed assignment `a[i].f = rhs` evaluates indices and `rhs` before borrowing `a[i]` (safe for recursive arenas). Heap values (`Array`, `List`, `Map`, `Str`) move on last use, copy via `.copy()`, swap in O(1) via `swap(mut a, mut b)` / `a.swap(i, j)`, and move out via `.take()` / `.absorb(mut other)` for small-to-large. Destructuring `let (r, c) = q[head]` always owns/copies values. No stored or returned references.

## 7. Records, structs & enums

`Eq`, `Ord` (only when every field is `Ord`, never `?T`), `Hash`, `Show`, `Read`, `Gen`, `Copy`, and `Index` derive automatically — no `@derive`.

```cp CP Lang v2.1
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

```cp CP Lang v2.1
@multitest                                // whole-file, or `@multitest fn solve():` (code above runs once!)
let (n, m): (1..=2e5, 1..=n) = read()     // dependent bounds on scalars, tuples, record fields
let g: [Row, Col]?Int = read(none: "X")   // symmetric `none:` on read / print / read_output
print(ans, none: "IMPOSSIBLE")            // lists print space-separated; domain v prints label; grids by row
for (r, c) in read_until((0, 0)): ...     // sentinel-terminated stream; read_line() skips trailing ws after read()
assume n >= 1 and m <= 200_000            // local debug + stress filter/validator (pure; stripped on judge)
assert ok                                 // ALWAYS checked (local + judge)
debug(n, dp)                              // stderr locally (structured `[1, 2]`, replayed on stress fail); stripped on judge

@tags('.', '*', '1'..='9')
enum Cell: Empty | Wall | Coin(val: 1..=9) // single-char packed tags + char-range payload

@memo                                     // top-level or nested in loops (block-scoped cache; runtime flat <= 2^24
fn dp(u: V, used: Bool) -> Int: ...       // else salted hash; uniform range-param narrowing; @memo(flat|hash))

let (k, walls) = read(gen: () => gen_walls(pts)) // typed gen: with optional `witness val`
@gen fn make_case(size: Int): ...         // whole-input generator (auto choice-stream shrinking)
@brute(main) fn slow(): ...               // default oracle for `cpl stress` (stripped on judge)
@checker fn check(): ...                  // reads solution stdout via read_output(), checks with assert
@interactor fn judge(): ...               // local stdio judge paired with `ask("?", x, limit: 2)`
@grader const N: 0..=2000                 // extern const; bodyless `@grader fn` = extern; `@grader struct` = stateful
@grader fn restore(n: Int) -> List<Int>:  // file-scope `domain V` binds from grader args for shared callbacks
```

- **Honest `cpl stress`:** Defaults to in-file `@brute`/`@checker`/`@interactor`, escalates sizes (`1..=20`; `--max-vals` for overflow hunting), diffs line-by-line unless `@checker` (`--close 1e-9` for floats), always builds with `-DLOCAL`, shrinks via choice streams, replays `debug(...)` side-by-side, and prints max size, rejection rate (>50% warning), diff mode, and output diversity.

## 9. Collections (unified verb vocabulary)

Every container uses `.len`, `.is_empty`, `.clear()`, `.take()` (O(1) move-out leaving empty), and `.absorb(mut other)` (O(min) small-to-large drain). Discarding `.pop*()`/`.add()`/`.unite()` results never warns. Construct empty `List`/`Deque` with `= []`, or pass any iterable to `Set<T>(it)` / `SortedBag<T>(it)` / `MinHeap<T>(it)` / etc.

- **`List<T>`:** `var xs: List<T> = []`, `[expr for x in it if c]`, `List.filled(n, v)`, `.push(x)`, `.push_id(x) -> Int`, `.pop() -> ?T`, `.first`, `.last` (by value; mutate top via `xs[xs.len - 1]`), `.swap(i, j)`, `xs[l..r]`, `xs + ys`, `xs.freeze() -> (domain D, [D]T)`.
- **`Deque<T>`:** `var q: Deque<T> = []`, `.push(x)`, `.pop() -> ?T`, `.push_front(x)`, `.pop_front() -> ?T`, `.first`, `.last`, `q[i]`.
- **`MonoDeque<T>(min | max)`:** imperative monotone queue — `.push(at: Int, val: T)`, `.drop_before(min_at: Int)`, `.clear()`, `.best -> ?T`.
- **`MinHeap<T>()` / `MaxHeap<T>()`:** `.push(x)`, `.pop() -> ?T`, `.peek -> ?T`, `.len`, `.is_empty`. Use `desc(x)` inside tuples for mixed order, e.g. `MinHeap<(Int, Desc<V>)>` or `Set<(Int, Desc<V>)>`.
- **`Set<T>()` / `SortedBag<T>()` / `HashSet<T>()` / `Marks<D>()`:** `.add(x) -> Bool`, `.remove(x)`, `x in s`, `|`, `&`, `-`, `^`. Ordered (`Set`, `SortedBag`) also have `.first`, `.last`, `.pop_first()`, `.pop_last()`, `.first_ge(k)`, `.first_gt(k)`, `.last_le(k)`, `.last_lt(k)` (all `-> ?K`), `.rank(k)`, `.nth(i)`. `Marks<D>()` is a generation-stamped visited set with O(1) `.clear()`.
- **`Map<K, V>()` / `HashMap<K, V>()`:** `m[k]` (traps if missing), `m[k] f= x`, `m[k] ??= init`, `m.get(k) -> ?V`, `m.remove(k)`, `k in m`, `for (k, v) in m.items`. Ordered `Map` also has `.first`, `.last`, `.first_ge(k)`, `.first_gt(k)`, `.last_le(k)`, `.last_lt(k)` (all `-> ?(K, V)`). (All hashes are per-process salted open-addressed flat tables.)
- **`Bitset<D>()` (large `D`) & `Subset<D>` (`Copy` `Index`, `D.K <= 25`):** `.add(v)`, `.remove(v)`, `.toggle(v)`, `v in s`, `|`, `&`, `-`, `^`, `.len` (popcount), `s.shifted(k)` (`k > 0` up, `k < 0` down). `Subset<D>` also has `Subset<D>.empty`, `.full`, `for s in Subset<D>`, `for g in s.subsets`, `s.first -> ?D`, `s.with(v)` / `s.without(v)`.

## 10. Search, sequences & math prelude

- **8 canonical search names (plus `r.mid`):**
  - Binary search (monotone, debug-checked; free fns on ranges, methods on domains; `F64` ranges take `steps: 80`): **`bsearch_first(r, ok) -> ?I`**, **`bsearch_last(r, ok) -> ?I`**, **`bsearch_first_some(r, f) -> ?(I, W)`**.
  - Sorted lookup (uniform on `List`, `[D]T`, slices, `Set`, `Map`, `SortedBag`; `by:` on sequences): **`xs.first_ge(x)`**, **`xs.last_le(x)`**.
  - Linear search: **`first(it) -> ?T`**, **`a.find(x) -> ?I`**, **`min_by(it, key) -> ?T`** / **`max_by(it, key) -> ?T`** (first extremum wins ties).
- **Sliding & sequences:** `D.windows(k)`, `D.chunks(k)` (clamped non-overlapping `Range<D>`), `sliding_min(a, k)`, `sliding_max(a, k)`, stateless `slide_until(r, (l, r) => ok)` yielding `(lo: I, hi: I)`, stable `sort(mut a, by:)` / `sorted(it, by:)` (any iterable), `reverse(mut a)`, `dedup(mut a)`, `rotate_left`/`rotate_right(mut a, k)`, `next_permutation(mut a)`, `let xs = compress(all)` (domain `xs.D`, `xs[v]`, slice `xs[range]`, `xs.at(val) -> xs.D`, `xs.first_ge(val)`), `prefix(a)` (`pre.sum(l..=r)` in 1D/2D), `scan(a, +)` / `scan(a, max)`, `runs(a)`, `xs.adjacent`, `a.items`, `sum(it)` / `prod(it)` (type-zero/one seeded), `any(it)`, `all(it)`, `count(it)`, `Perm<V>` (`.inv()`, `.cycles()`).
- **Math & geometry:** `abs`, `min`, `max`, `gcd`, `lcm`, `isqrt`, `pow_mod(a, e, m)`, `Sieve(n)` (`.is_prime`, `.spf`, `.primes`, `.factor`), `factor(x) -> List<(Int, Int)>`, `divisors(x) -> List<Int>`.
  - **`Mod<P>`:** unified for `const` (`type Z = Mod<MOD>`) and runtime `let` (`let M: 1..=1e9 = read(); type Z = Mod<M>`). Works for any `P >= 1` (incl. composite); 4-byte when `P <= 2^31 - 1`, `I128` widen above; one-way `Int` reduction; `+`, `-`, `*`, `/`, `**`, `.pow(e)`, `.inv`, `.try_inv() -> ?Z`.
  - **`Comb<P>()`:** lazily auto-grows across test cases — `.fact(k)`, `.inv_fact(k)`, `.choose(n, k)`, `.arrange(n, k)`.
  - **`Matrix<R, C, T>`:** `a * b`, `a.pow(e)`, `a.mul_with(b, add:, mul:, zero:)` (tropical `(max, +)`).
  - **`Pt<T = Int>`:** `+`, `-`, `a.dot(b)`, `a.cross(b)`, `a.norm2`, `turn(a, b, c)` (`Int` only), `turn_eps(a, b, c, eps)`, `unit(angle)`, `ByAngle`, `convex_hull(pts)`.

## 11. Core contest data structures & graphs (`std`)

- **Range structures** (accept bare operators `+`, `*`, `^`, `&`, `|`, `min`, `max`, `gcd` OR `(id, op)`):
  - `Fenwick<D, Int>(a)` / `Fenwick2D<R, C, Int>()` — `.add(i, d)`, `.sum(l..=r)`.
  - `SparseTable<D, Int>(min, a)` — `.fold(l..=r) -> ?T` in O(1) (`none` on empty range; no identity needed).
  - `SegTree<D, T>(+, a)` or `SegTree<D, T>(Monoid(id, op), a)` — `.set(i, v)`, `.fold(l..=r)`, `.max_right(from, ok)`, `.min_left(to, ok)`.
  - `LazySegTree<D, T>(AddSum, a)` (stock tags: `AddSum`, `AddMin`, `AddMax`, `AssignSum`, `AssignMin`, `AssignMax`, or custom `Action(id, compose, apply, over:)`) — `.apply(l..=r, tag)`, `.set(i, v)` (pushes root-to-leaf), `.fold(l..=r)`, `.max_right(from, ok)`, `.min_left(to, ok)`.
  - `Steps<K, V>(init)` — lazy-sorted piecewise-constant intervals (covers interval sets via `Steps<K, Bool>`).
  - `Dsu<V>()` — `.unite(a, b) -> Bool`, `.find(v) -> V`, `.same(a, b)`, `.size(v)`, `.components`, `.groups() -> List<List<V>>`, `.mark()` / `.undo_to(snap)`.
- **Graphs & trees** (`from:`/`root:`/`g` first for inference; unweighted algorithms accept `Graph<V, E>` or plain `V -> Iter<V>`, using `topo::<V>(next)` when vertexless):
  - `Graph<V>` / `Graph<V, W>` — `Graph.undirected(es)`, `Graph.directed(es)`, `g[u]`, `bfs`, `dijkstra`, `shortest` (auto 0-1 BFS for `0..=1` weights), `topo`, `topo_prefix`, `scc`, `condense`, `two_edge_components`, `biconnected_components`, `articulation_points`, `two_color`, `mst`.
  - `Rooted<V>(g, root)` / `Rooted.from_parents(par)` — value-owned preorder domain `t.Pre`, `t.pos(v) -> t.Pre`, `t.preorder: [t.Pre]V`, `t.postorder`, `t.tour` (`(v, entering)`), `.parent(v)`, `.children(v)`, `.depth(v)`, `.size(v)`, `.lca(a, b)`, `.dist(a, b)`, `.is_anc(a, b)`, `.ancestor(v, k)`, `.lift_while(v, pred) -> ?V` (highest ancestor where `pred` holds from `v` up), `.sub(v) -> Range<t.Pre>`, `.chains(a, b, edges?: Bool)`, **`.fold_path(a, b, ds)`**, **`.apply_path(a, b, mut ds, tag)`** (1×-memory via `Reversible`), `.virtual_tree(keys)`.
  - `FlowGraph<Node>()` / `MinCostFlow<Node>()` / `max_weight_matching<L, R>` — pair with payload-enum `Node`.
  - `TwoSat<V>()` — `.add_or(a, b)`, `.implies(a, b)`, `.solve() -> ?[V]Bool`.
- **Strings:** `z_function(s)`, `prefix_function(s)`, `palindromes(s)`, `SuffixArray(s)` (`.sa`, `.lcp`), `RollingHash(seq)` (accepts `Str` or `[D]T`, independent per-instance double salted hash, `.get(l..r) -> HashVal`).
- *(Advanced structures — `Rope`, `RangeKth`, `LiChao`, `PotentialDsu`, `Cover`, `Line`/`Hom`, `TwoStackFold`, `mo`, `Poly`/NTT, `XorBasis`, `Frac`, `Fixed`, `Tri`, `Counter` — live in `examples/local-lib/` with identical zero-cost speed.)*

## 12. Reserved keywords (40 total)

- **Contestant (32, including 3 literals):** `let`, `var`, `const`, `fn`, `struct`, `enum`, `domain`, `type`, `use`, `pub`, `if`, `elif`, `else`, `for`, `while`, `in`, `match`, `return`, `break`, `continue`, `defer`, `and`, `or`, `not`, `assert`, `assume`, `mut`, `self`, `unchecked`, `true`, `false`, `none`.
- **Library-author only (8):** `trait`, `impl`, `where`, `lends`, `lend`, `law`, `inline`, `yield`.
