# CP Lang — One-Page Contest Cheat Sheet

*Surviving minimal language reference after Stage 3 Subtraction Audit (`research/subtraction-audit-1.md`). Every construct has one spelling.*

---

## 1. Bindings, Numbers, Operators

```cp
let n = 10                      // immutable binding
var ans = 0                     // mutable binding
const MOD = 1_000_000_007       // compile-time constant (inlined)
```

- **4 numeric types:** `Int` (64-bit signed, symmetric), `U64` (wrapping 64-bit unsigned), `I128` (128-bit signed), `F64` (64-bit IEEE float). Range types like `0..=1_000_000_000` pack tightly (4 bytes) and check bounds on entry.
- **Scalars:** `Bool` (`true`, `false`), `Char` (1 ASCII byte, `'a'`), `Str` (flat ASCII bytes, no UTF-8).
- **Comments:** `// line comment` only. Indentation defines blocks; no semicolons.
- **Arithmetic:** `+`, `-`, `*`, `/` (Euclidean floor division), `%` (Euclidean, always in `0..|b|`), `**` (power).
- **Bitwise:** `&`, `|`, `^` (XOR), `~` (NOT), `<<`, `>>`, plus `&=`, `|=`, `^=`, `<<=`, `>>=`. Helpers: `popcount(x)`, `ctz(x)`, `clz(x)`, `bit_len(x)`.
- **Logic & updates:** `and`, `or`, `not`, `==`, `!=`, `<`, `<=`, `>`, `>=`, `x += y`, `x min= y`, `x max= y` (any 2-arg fn works as `x f= y`).

---

## 2. Optionals (`?T`)

No `INF` or `-1` sentinels. `?Int`, `?F64`, `?V`, `?Bool`, `?Char`, and `?Enum` are niche-packed with zero extra memory.

```cp
var dist: [V]?Int = none        // none or bare value `dist[src] = 0`
let d = dist[v] ?? -1           // fallback on none
let best = min(a, b)            // min / max skip none; both none => none
dist[v] min= d + w              // updates if dist[v] is none or d + w < dist[v]!
let x = opt!                    // unwrap (traps on none)
let y = maybe()?                // early-return none from fn returning ?T
(adj[k] ??= []).push(x)         // initialize place if none, then lend mutably
if let d = dist[v]: print(d)    // bind non-none value
```

---

## 3. Domains, Indices, Boundaries, Ranges & Arrays

```cp
domain V = 1..=n                // nominal bounded index: 1..=n (0-cost in C++)
let a: [V]Int = read()          // dense array indexed by V (zero checks on a[v])
var dp: [Row, Col]Int = 0       // 2D array indexed by (Row, Col)
var cnt: [0..=k]Int = 0         // bounded Int range as Index (checked on entry)
let b: [V]Int = [V] v => a[v] * 2  // comprehension over domain V
```

- **Index (`V`) & Seq (`1..=n`):** `V.K` (count), `V.first`, `V.last`, `v.to_idx` (`0..V.K`), `V.from_idx(i) -> ?V`, `v + k -> ?V`, `v - k -> ?V`, `v.cyc(k) -> V` (cyclic wrap).
- **Boundaries (`V.bounds`, `V.K + 1` gaps):** `v.l` / `v.r` (gap left/right of `v`), `v.l(k)` / `v.r(k)` (`k` steps from `v`), `b.left -> ?V`, `b.right -> ?V`.
- **One `Range<I>` type:** `l..r` (half-open), `l..=r` (inclusive), `..i` (strictly before `i`), `..=i`, `i..` (from `i` to end), `i.r..` (strictly after `i`). Properties/methods: `r.first -> ?I`, `r.last -> ?I`, `r.mid -> ?I`, `r.len`, `r.is_empty`, `r.rev`, `r.step(k)`.
- **Slices & grids:** `a[l..=r]` (mutable slice; `a[l..=r].sort()`, `a[l..=r].reverse()`), `g.row(r)`, `g.col(c)`, `swap(mut g.row(r1), mut g.row(r2))`, `(r, c).around()` (4-neighbors), `(r, c).around8()`.
- **Algebraic `Index` (automatic, O(1), `Copy`):** `Bool`, unit/payload `enum` (derived on demand), `?T`, `lo..=hi`, tuples `(A, B)`, and `Subset<D>` all work as `[K]T` array keys and flat `@memo` keys.
- **Escape hatches:** `a.raw[i]` (0-based `Int` indexing, checked in debug) and `unchecked:` block (elides bounds/overflow traps in judge builds).

---

## 4. Strings (`Str`)

`Str` is an ASCII byte array `[domain]Char` — O(1) indexing and slicing, zero UTF-8.

```cp
let s: Str = read()
let ch = s[i]                   // Char; digit: ch - '0', letter: ch - 'a'
let sub = s[l..r]               // O(1) slice; s.len, s.rev, s.split(' ')
let out = " ".join("{x}" for x in ans)
print("{val:.6f}")              // formatting: :.6f, :08b, :x
```

---

## 5. Control Flow & `defer`

```cp
if x < 0: neg()
elif x == 0: zero()
else: pos()

let sign = if x >= 0: 1 else: -1          // ONE inline-if syntax (no `then`)

for v in V: ...
for (i, x) in a.items: ...                // (index/key, value) everywhere
for i in (0..n).step(2).rev: ...
'outer: for r in Row:
    for c in Col:
        if bad(r, c): break 'outer        // labeled break / continue

match op:                                 // exhaustive; bare variants, `=>`
    Add(l, r, v) => st.apply(l..=r, v)
    Query(l, r) => print(st.fold(l..=r))

let snap = dsu.mark()
defer dsu.undo_to(snap)                   // runs on every block exit (no `undo`/`with`)
```

---

## 6. Functions, Closures & Ownership

```cp
fn swap_adj(mut a: [V]Int, i: V):         // mut on param AND at call site
    if let j = i + 1: swap(mut a[i], mut a[j])

swap_adj(mut a, v)                        // caller explicitly writes `mut`

var vis: [V]Bool = false
fn dfs(u: V):                             // nested `fn` mutates outer `var` directly
    vis[u] = true
    for v in g[u]: if not vis[v]: dfs(v)

sort(mut es, by: e => (e.w, -e.u))        // ONE lambda syntax: `x => expr`
```

- **Ownership:** Small values (`<= 16 B`, no heap) are `Copy`. Heap values (`Array`, `List`, `Map`, `Str`) move on last use; copy explicitly with `.copy()`.
- **Destructuring copies values:** `let (r, c) = q[head]` copies/moves values out cleanly — mutating `q` afterward is always safe.
- **No stored references:** Functions cannot return references; `mut` borrows stay lexical. Generics are always explicit: `fn f<D>(a: [D]Int)`.

---

## 7. Records, Structs & Enums

`Eq`, `Ord` (only when all fields are `Ord`), `Hash`, `Show`, `Read`, `Gen`, `Copy`, and `Index` derive automatically — no `@derive`.

```cp
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

---

## 8. I/O, `@multitest`, `@memo` & Verification

```cp
@multitest                                // runs file body `t` times; also @multitest(until_eof)
let (n, m): (Int, Int) = read()           // typed token read for scalars, tuples, records, [D]T
assume n >= 1 and m <= 200_000            // local debug check + stress filter; stripped on judge
assert ok                                 // ALWAYS checked (local + judge)
let line = read_line()                    // raw line; read_until_eof() for rest of input
debug(n, dp)                              // prints to stderr locally; stripped on judge

@tags("W", "L")
enum Outcome = Win | Lose                 // read() maps "W" -> Win, "L" -> Lose

@memo                                     // flat table if args are Index, salted HashMap otherwise
fn dp(u: V, used: Bool) -> Int: ...       // use @memo(hash) to force sparse table

@brute(main)                              // oracle for `cpl stress` (choice-stream shrinking built in)
fn slow(): ...

@gen                                      // optional custom input generator for `cpl stress`
fn gen_case(): ...
```

- **Interactive & special judges:** `ask("? {mid}")` flushes stdout and reads response; `@interactor fn judge(...)` runs local interactive tests; `@checker fn check(...)` validates constructive outputs; `@grader` marks IOI/DMOJ function-only submissions.

---

## 9. Collections (Unified Verb Vocabulary)

Every container uses `.len`, `.is_empty`, and `.clear()` (properties, never `.len()`).

- **`List<T>`:** `var xs: List<T> = []`, `.push(x)`, `.push_id(x) -> Int`, `.pop() -> ?T`, `.first`, `.last`, `xs.freeze() -> (domain D, [D]T)`.
- **`Deque<T>`:** `.push(x)`, `.pop() -> ?T`, `.push_front(x)`, `.pop_front() -> ?T`, `.first`, `.last`, `q[i]`.
- **`MinHeap<T>` / `MaxHeap<T>`:** `.push(x)`, `.pop() -> ?T`, `.peek -> ?T`. Use `desc(x)` inside tuples for mixed order, e.g. `MinHeap<(Int, Desc<V>)>`.
- **`Set<T>` / `SortedBag<T>` / `HashSet<T>`:** `.add(x)`, `.remove(x)`, `x in s`, `|`, `&`, `-`, `^`. Ordered (`Set`, `SortedBag`) also have `.first`, `.last`, `.pop_first()`, `.pop_last()`, `.first_ge(k)`, `.first_gt(k)`, `.last_le(k)`, `.last_lt(k)`, `.rank(k)`, `.nth(i)`.
- **`Map<K, V>` / `HashMap<K, V>`:** `m[k]` (traps if missing), `m[k] ??= init`, `m.get(k) -> ?V`, `m.remove(k)`, `k in m`, `for (k, v) in m.items`. Ordered `Map` also has `.first`, `.last`, `.first_ge(k)`, `.last_le(k)`. (All hashes are salted.)
- **`Bitset<D>` (large `D`) & `Subset<D>` (`Copy` `Index`, `D.K <= 25`):** `.add(v)`, `.remove(v)`, `.toggle(v)`, `v in s`, `|`, `&`, `-`, `^`, `.len` (popcount), `s.shifted(k)`. `Subset<D>` also has `for g in s.subsets` (submasks) and `s.with(v)` / `s.without(v)`.

---

## 10. Search, Sequences & Math Prelude

- **Binary search (monotone, checked in debug):** `bsearch_first(r, ok) -> ?I`, `bsearch_last(r, ok) -> ?I` (works on `Range<I>`, `D`, `0..=hi`, and `F64` with `steps: 80`).
- **Sorted lookup & linear search:** `xs.first_ge(x) -> ?I`, `xs.last_le(x) -> ?I`, `first(it) -> ?T`, `a.find(x) -> ?I`, `min_by(it, key)`, `max_by(it, key)`.
- **Sequence helpers:** `sort(mut a, by: ...)`, `sorted(a, by: ...)`, `reverse(mut a)`, `dedup(mut a)`, `compress(a) -> (domain Val, [I]Val, [Val]T)`, `prefix(a)` (`pre.sum(l..=r)` in 1D/2D), `scan(a, +)` / `scan(a, max)`, `runs(a)`, `D.windows(k)`, `sliding_min(a, k)`, `sliding_max(a, k)`, `MonoDeque<T>`, `sliding(D, add:, drop:, ok:)`, `xs.adjacent`, `a.items`, `sum(it)`, `any(it)`, `all(it)`, `count(it)`, `Perm<V>` (`.inv()`, `.cycles()`).
- **Math & geometry:** `abs`, `min`, `max`, `gcd`, `lcm`, `isqrt`, `pow_mod(a, e, m)`, `sieve(n)`, `factor(x)`.
  - **`Mod<P>`:** unified for both `const` (`type Z = Mod<MOD>`) and runtime `let` (`let p: 1..=1e9 = read(); type Z = Mod<p>`). Supports `+`, `-`, `*`, `/`, `.pow(e)`, `.inv`.
  - **`Comb<P>(n)`:** `.fact(k)`, `.ncr(n, k)`, `.npr(n, k)`.
  - **`Matrix<R, C, T>`:** `a * b`, `a.pow(e)`, `a.mul_with(b, add:, mul:, zero:)` (for tropical `(max, +)`).
  - **`Pt`:** 2D point/vector (`+`, `-`, `a.dot(b)`, `a.cross(b)`, `a.norm2`, `convex_hull(pts)`).

---

## 11. Core Contest Data Structures & Graphs (`std`)

- **Range structures** (accept bare operators `+`, `*`, `^`, `&`, `|`, `min`, `max`, `gcd` OR `(id, op)`):
  - `Fenwick<D, Int>(a)` / `Fenwick2D<R, C, Int>()` — `.add(i, d)`, `.sum(l..=r)`.
  - `SparseTable<D, Int>(min, a)` — `.fold(l..=r)` in O(1).
  - `SegTree<D, T>(+, a)` or `SegTree<D, T>(id, op, a)` — `.set(i, v)`, `.fold(l..=r)`, `.bsearch_right(l, ok)`.
  - `LazySegTree<D, T>(AddSum, a)` (stock tags: `AddSum`, `AddMin`, `AddMax`, `AssignSum`, `AssignMin`, `AssignMax`, or custom `Action(id, compose, apply)`) — `.apply(l..=r, tag)`, `.fold(l..=r)`.
  - `Steps<K, V>(init)` — piecewise-constant intervals (covers interval sets via `Steps<K, Bool>`).
  - `Dsu<V>()` — `.unite(a, b) -> Bool`, `.find(v) -> V`, `.same(a, b)`, `.size(v)`, `.components`, `.mark()` / `.undo_to(snap)`.
- **Graphs & trees:**
  - `Graph<V>` / `Graph<V, W>` — `Graph.undirected(es)`, `Graph.directed(es)`, `g[u]`, `bfs`, `dijkstra`, `zero_one_bfs`, `topo_sort`, `scc`, `bridges`, `mst`.
  - `Rooted<V>(edges, root)` — `.parent(v) -> ?V`, `.children(v)`, `.depth(v)`, `.size(v)`, `.lca(a, b)`, `.kth_anc(v, k)`, `.sub(v) -> Range<Pre>`, `.chains(a, b)` (HLD ranges over any `SegTree`/`LazySegTree`/`Fenwick`).
  - `FlowGraph<Node>()` — `.add(u, v, cap)` / `.add_cost(u, v, cap, cost)`, `.max_flow(s, t)`, `.min_cost_flow(s, t)` (pairs cleanly with payload-enum `Node`).
  - `TwoSat<V>()` — `.add_or(a, b)`, `.implies(a, b)`, `.solve() -> ?[V]Bool`.
- **Strings:** `z_func(s)`, `kmp(s)`, `manacher(s)`, `SuffixArray(s)` (`.sa`, `.lcp`), `RollingHash(s)` (double salted hash, `.get(l..r)`).

---

## 12. Reserved Keywords (40 Total)

- **Contestant (32, including 3 literals):** `let`, `var`, `const`, `fn`, `struct`, `enum`, `domain`, `type`, `use`, `pub`, `if`, `elif`, `else`, `for`, `while`, `in`, `match`, `return`, `break`, `continue`, `defer`, `and`, `or`, `not`, `assert`, `assume`, `mut`, `self`, `unchecked`, `true`, `false`, `none`.
- **Library-author only (8):** `trait`, `impl`, `where`, `lends`, `lend`, `law`, `inline`, `yield`.
