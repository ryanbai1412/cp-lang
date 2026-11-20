Canonical one-page contest reference for the locked **CP Lang 0.2.2** specification ([architecture](https://github.com/ryanbai1412/cp-lang/blob/main/research/language-architecture-v2.2.md), [decisions §16–§19](https://github.com/ryanbai1412/cp-lang/blob/main/decisions.md)). Every construct has one spelling: 34 reserved keywords, 4 numeric types, 7 core traits, and 8 contestant decorators. This page mirrors [`docs/one-page-cheat-sheet.md`](https://github.com/ryanbai1412/cp-lang/blob/main/docs/one-page-cheat-sheet.md).

## 1. Bindings, Numbers & Operators

```cp CP Lang 0.2.2
let n = 10                      // immutable binding
var ans = 0                     // mutable binding
const MOD = 1_000_000_007       // compile-time constant (inlined)
```

- **4 numeric types:** `Int` (64-bit signed symmetric `-(2^63-1)..=(2^63-1)`), `U64` (wrapping 64-bit unsigned), `I128` (128-bit signed), `F64` (64-bit IEEE double). Range types (`0..=1_000_000_000`) pack tightly (1/2/4 bytes) and check bounds on entry.
- **`F64` rules:** `1e18` is `Int`; `.` marks `F64` (`0.0`). Integer literals unify with `F64`; `Int` vars need `F64(x)`. `Int(x)` on `F64` is an error — use `floor`/`ceil`/`round`/`trunc`. NaN and Inf trap locally, making `<` total. Bare `print(x)` of `F64` is an error — write `"{x:.6f}"` (never `-0.00`). `==`/`!=` lint toward `close(a, b, eps)` and `sign(x, eps)` (exempting `== 0.0`). Intrinsics: `sqrt`, `sin`, `cos`, `tan`, `atan2`, `exp`, `ln`, `hypot`, `PI`.
- **Scalars, comments & blocks:** `Bool` (`true`, `false`), `Char` (1 ASCII byte, `'a'`), `Str` (`0..len`-indexed ASCII bytes, zero UTF-8). Lossless casts: `Int(ch)` (`0..=127`), `Char(i)`, `Int(b)` (`0 | 1`). Hex `0xFF`, binary `0b1010`. `//` line comments only; indentation defines blocks; lines continue after binary ops, `=`, or inside open brackets.
- **Arithmetic, bitwise & updates:** `+`, `-`, `*`, `/` (Euclidean floor div), `%` (Euclidean, always in `0..|b|`), `**` (power). `..`/`..=` bind looser than arithmetic, tighter than comparisons/`in`. Bitwise: `&`, `|`, `^` (XOR), `~`, `<<`, `>>`, plus `&=`, `|=`, `^=`, `<<=`, `>>=`, `popcount`, `ctz`, `clz`, `bit_len`. Logic: `and`, `or`, `not`, `in`, `not in`, `==`, `!=`, `<`, `<=`, `>`, `>=`, `x += y`, `x min= y`, `x max= y` (any 2-arg fn works as `x f= y`).

---

## 2. Optionals (`?T`) & Tropical DP

No `INF` or `-1` sentinels. `?T` holds any value of `T` or `none` (`?Int` represents every 64-bit `Int` plus `none`), and the compiler packs optional layouts when proven safe.

```cp CP Lang 0.2.2
var dist: [V]?Int = none        // none or bare value `dist[src] = 0`
let d = dist[v] ?? -1           // fallback on none (parens required when mixing `??` with `+ - * / %`)
let u = opt ?? return (none, none)  // diverging RHS (`return`, `break`, `continue`) supported on `??`
let best = min(a, b)            // min/max skip none: (T, ?T) -> T, (?T, ?T) -> ?T; 3+ via min([a, b, c])
dist[v] min= dist[u] + w        // tropical: `+`, `-`, unary `-` on ?Int/?F64 absorb none; min=/max= skip none
ans min= dist[v]                // skips when dist[v] is none even if `ans` is plain Int
if dq.last != x: ...            // `==`/`!=` auto-lift T to ?T (never `<`/`<=`/`>`/`>=`)
let x = opt!                    // unwrap (traps on none; elided when flow-proven); `maybe()?` returns none
(adj[k] ??= []).push(x)         // initialize place lazily if none, then lend mutably
if let a = x and let b = y: ... // chained optional bindings (also `if let` as expression)
```

- **Tropical guardrails:** `*`, `/`, `%`, bitwise ops, `?V + k`, `sum(gen)`, and `+=`/`-=` never lift over `?T`. **`min=`/`max=` skip `none`** (tropical `+inf`/`-inf`); when `none` means *failure/poison* and every branch must succeed, check explicitly (`if let hit = ...: best min= hit else: best = none; break`).
- **Uniform optionality & no `?T` `Ord`:** `v + k`, `v - k`, `xs.last`, `min(gen)`/`max(gen)`, and `min_by`/`max_by` return `?T`. Neither `?T` nor `Mod<P>` implements `Ord`; sort optionals explicitly via `by: x => (x.d == none, x.d ?? 0)`.

---

## 3. Domains, Indices, Boundaries, Ranges & Grids

```cp CP Lang 0.2.2
domain V = 1..=n                // nominal bounded index: 1..=n (0-cost in C++)
let a: [V]Int = read()          // dense array indexed by V (zero checks on a[v])
var dp: [Row, Col]Int = 0       // 2D array indexed by (Row, Col); g[p] works for p: (Row, Col)
var cnt: [0..=k]Int = 0         // structural range array (`['a'..='z']Int`; clones fill per slot)
cnt.fill(0)                     // reset; hot multi-query loops use `[D]Bool` + touched-list reset
let b: [V]Int = [V] v => a[v] * 2   // domain comprehension (list comprehension: `[x * 2 for x in a if x > 0]`)
```

- **Anonymous range vs `domain`:** `[0..n]T`, `[0..=n]T`, `['a'..='z']T` are structural (`for i in 0..n` gives `i: Int`). `domain V = 1..=n` is nominal (`for v in V` gives `v: V`, `v + 1 -> ?V`, `v - 1 -> ?V`, `b - a -> Int`, `v.cyc(k) -> V`, `V.K`, `V.first`, `V.last`, `v.to_idx`, `V.from_idx(i) -> ?V` row-major on tuples).
- **Boundaries (`V.bounds`, `V.K + 1` gaps):** `v.l`/`v.r`, `v.l(k)`/`v.r(k)` (`v.l == v.l(1)`), `b.left -> ?V`, `b.right -> ?V`.
- **One `Range<I>` type:** `l..r`, `l..=r`, `..i`, `..=i`, `i..`, `i.r..`, `l.r..r.l`. Properties/methods: `.l`/`.r` (boundary cuts even if empty; canonical segtree split: `let m = r.mid!; (r.l..m.r, m.r..r.r)`), `.first -> ?I`, `.last -> ?I`, `.mid -> ?I`, `.len`, `.is_empty`, `.rev`, `.step(k)`, `(lo..=hi).clamp(D)`.
- **Slices & grids (`std.grid`):** `a[l..=r]` (mutable slice: `sort(mut a[l..=r])`, `reverse(mut a[l..=r])`, `.sorted()`, `.first_ge`, `.last_le`), `g.row(r)`, `g.col(c)`, `g.transposed` (`[C, R]T`), `swap(mut g.row(r1), mut g.row(r2))`. `[R, C]Char = read()`. `type Cell = (Row, Col)` works with `[Cell]T`, `bfs`, `Dsu<Cell>`, `@memo`, `Dir4` (`.cw`, `.ccw`, `.back`, `Dir4.from("UDLR", ch)`), `p.step(d)`, `p.ray(d)`, `p.adj4`, `p.adj8`.
- **Algebraic `Index` (5 O(1) value-copied shapes):** `Bool`, unit/payload `enum` (on demand), `?T`, `lo..=hi` (`Int`/`Char`), tuples/records `(A, B)`, plus `Subset<D>` (`D.K <= 25`). Arenas use `type Id = 0..=MAX` + `List<Node>` + `.push_id(x)` + `.freeze()`.
- **Escape hatches:** `a.raw[i]` (0-based `Int` index, checked locally) and `unchecked:` block.

---

## 4. Strings (`Str`)

ASCII-only, `0..s.len`-indexed byte array (zero UTF-8, O(1) indexing and slicing).

```cp CP Lang 0.2.2
let s: Str = read()
let ch = s[i]                   // Char; digit: ch - '0', letter: ch - 'a', s.items
let sub = s[l..r]               // O(1) slice; s[..k], s[k..], s.len, s.is_empty, match s:
var out = ""
out.push('a'); out += "bc" * 3  // mutable when `var`; s + t, s + ch, ch + s, Str(chars)
if s.starts_with(x, at: i): ... // s.split(' '), s.split_once(' '), s.trim_start(' '), s.parse_int()
let joined = " ".join("{x}" for x in ans)
let a: [domain A]Char = read()  // opt-in nominal domain for multi-string DP
```

---

## 5. Control Flow & Parallel Assignment

```cp CP Lang 0.2.2
if x < 0: neg()
elif x == 0: zero()                       // always `elif`, never `else if`
else: pos()

let sign = if x >= 0: 1 else: -1          // ONE inline-if syntax
var (r0, r1) = (a, b)
(r0, r1) = (r1, r0 - q * r1)              // parallel tuple assignment (RHS evaluated first)

for v in V if ok(v): ...                  // filtered loop; mutate via `for i in 0..xs.len: xs[i].f()`
for (i, x) in a.items: ...                // (index/key, value) everywhere
for i in (0..n).step(2).rev: ...
while let a = x and let b = y and a > 0:  // chained optional bindings + guard
'outer: for r in Row:
    for c in Col:
        if bad(r, c): break 'outer        // labeled break/continue; top-level `return` ends the program / the current test

let res = match op:                       // exhaustive expression; bare variants, `=>`
    Add(l, r, v) => st.apply(l..=r, v)
    Query(l, r) => st.fold(l..=r)
```

---

## 6. Functions, Generics, Closures & Ownership

```cp CP Lang 0.2.2
fn swap_adj(mut a: [V]Int, i: V):         // mut on param AND at call site
    if let j = i + 1: swap(mut a[i], mut a[j])

var vis: [V]Bool = false
fn dfs(u: V):                             // nested `fn` mutates outer `var` directly
    vis[u] = true
    for v in g[u]: if not vis[v]: dfs(v)

sort(mut es, by: e => (e.w, -e.u))        // ONE lambda syntax: `x => expr`; descending: `by: desc`
```

- **Generics & inference:** Write `<D>` explicitly on generic `fn`s (`fn f<D>(a: [D]Int)`). Array params may size off sibling `Int` params left or right (`fn f(a: [0..r]Bool, r: Int)`). Call sites unify non-closure args (`from:`, `root:`, `g`) before closures; use `f::<T>()` for return-polymorphic calls (`parse::<Int>(tok)`).
- **Ownership & evaluation order:** Small non-heap values (`<= 16 B`) copy implicitly; reading a copyable element releases its borrow before enclosing calls (`merge(hrc[u], v)`). In `a[idx].f = rhs`, `a[idx].f ??= rhs`, and compound `a[idx].f op= rhs`, **all index expressions and `rhs` finish before borrowing the mutable place in `a`** (safe for `trie[step(o)].ends = true` and `trie[o].next[s] ??= trie.push_id(Node())`). Heap values move on last use, clone via `.copy()`, swap in O(1) via `swap(mut a, mut b)` / `a.swap(i, j)`, and move out via `.take()` (small-to-large: `if dst.len < src.len: swap(mut dst, mut src); for x in src.take(): ...`). Destructuring `let (r, c) = q[head]` copies/owns values. No stored or returned references.

---

## 7. Records, Structs & Enums

**7 core traits** derive automatically whenever fields support them: `Index`, `Seq`, `Eq`, `Ord` (enum variants order by declaration order; structs/tuples with `?T` or `Mod<P>` omit `Ord`), `Hash`, `Show`, `Read`. (2 library-author traits: `Indexed`, `Iterable`.)

```cp CP Lang 0.2.2
type Edge = (u: V, v: V, w: Int)          // structural record; e.u or let (u, v, w) = e

struct SegNode:
    best: Int = 0
    cnt: Int = 1
    fn merge(self, o: SegNode) -> SegNode: ...
    fn reset(mut self): self.best = 0

enum Dir = N | E | S | W                  // unit enum (auto Index, K = 4, N < E < S < W)
enum Node: Source | Sink | Build(s: Station) | Serve(u: User)   // payload enum (auto sum Index)
```

---

## 8. I/O, `@!multitest`, `@memo`, Interactive & `@grader`

```cp CP Lang 0.2.2
let sv = Sieve(1e6)                       // above `@!multitest`: runs once (sieves, tables)
@!multitest                               // divider: below runs per test, fresh scope, `@memo` reset; or `@!multitest(until_eof)`
let (n, m): (1..=2e5, 1..=n) = read()     // dependent bounds on scalars, tuples, record fields
let g: [Row, Col]?Int = read(none: "X")   // symmetric `none:` on read / print / read_output
print(ans, none: "IMPOSSIBLE")            // lists space-separated; auto-flushes if program uses `ask(...)`
for (r, c) in read_until((0, 0)): ...     // sentinel stream; read_line() skips trailing ws after read()
assert ok                                 // ALWAYS checked (local + judge); feeds range/disjoint proofs
debug(n, dp)                              // stderr locally (replayed on stress fail); stripped on judge

@tags('.', '*', '1'..='9')
enum Cell: Empty | Wall | Coin(val: 1..=9) // single-char packed tags + char-range payload

@memo                                     // top-level or nested in fn/loop (auto-resets per call/iter)
fn dp(u: V, used: Bool) -> Int: ...       // auto-dispatches flat (<= 2^24) vs salted hash
                                          // permits `ask(...)` & extern `@grader fn` (1 call per arg tuple)

let (k, walls) = read(gen: () => gen_walls(pts)) // typed gen: with optional `witness val`
@gen fn make_case(size: Int): ...         // whole-input generator (choice-stream shrinking)
@brute(main) fn slow(): ...               // default oracle for `cpl stress` (stripped on judge)
@checker fn check(): ...                  // reads solution stdout via read_output(), validates with assert
@interactor fn judge(sol: Interactor):    // read() = hidden test; sol.recv() = sol stdout; sol.send(x) = sol stdin
    ...                                   // pairs with `ask("?", x, limit: K)` (auto-flushes all print())

// Stateful @grader: top-level `var` (or `@grader struct`); bodyless `@grader fn` = extern oracle
var state: ?State = none
@grader fn get_distance(i: Int, j: Int) -> Int
@grader fn init(n: Int, a: [0..n]Int): state = State(a)
@grader fn query(x: Int) -> Int: ...
```

- **`cpl stress`:** Uses in-file `@brute`/`@checker`/`@interactor`, escalates sizes (`1..=20`; `--max-vals`), diffs line-by-line unless `@checker` (`--close 1e-9` for floats), builds with `-DLOCAL`, shrinks failures via choice streams, and replays `debug(...)`.

---

## 9. Collections (Unified Verb Vocabulary)

Every container uses `.len`, `.is_empty`, `.clear()`, and `.take()` (O(1) move-out leaving empty). Discarding `.pop*()`/`.add()`/`.unite()`/`.remove()` results never warns. Construct empty `List`/`Deque` with `= []`, or pass any iterable to `List(it)` / `Set<T>(it)` / `SortedBag<T>(it)` / `MinHeap<T>(it)`.

- **`List<T>`:** `var xs: List<T> = []`, `[expr for x in it if c]`, `List.filled(n, v)`, `.push(x)`, `.push_id(x) -> Int`, `.pop() -> ?T`, `.insert(i, x)`, `.remove(i) -> T`, `.first`, `.last`, `.swap(i, j)`, `xs[l..r]`, `xs + ys`, `xs.freeze() -> (domain D, [D]T)`.
- **`Deque<T>`:** `var q: Deque<T> = []`, `.push(x)`, `.pop() -> ?T`, `.push_front(x)`, `.pop_front() -> ?T`, `.first`, `.last`, `q[i]`.
- **`MinHeap<T>()` / `MaxHeap<T>()`:** `.push(x)`, `.pop() -> ?T`, `.peek -> ?T`. Use `desc(x)` inside tuples for mixed order (`MinHeap<(Int, Desc<V>)>`).
- **`Set<K>()` / `SortedBag<K>()` / `HashSet<K>()`:** `.add(x) -> Bool`, `.remove(x)`, `x in s`, `x not in s`, `|`, `&`, `-`, `^`. Ordered (`Set`, `SortedBag`) also have `.first`, `.last`, `.pop_first()`, `.pop_last()`, `.first_ge(k)`, `.first_gt(k)`, `.last_le(k)`, `.last_lt(k)` (**all `-> ?K`**), `.rank(k)`, `.nth(i)`.
- **`Map<K, V>()` / `HashMap<K, V>()`:** `m[k]` (**traps if missing**), `m[k] = v` (inserts/overwrites), **`m[k] f= x` (auto-inserts from op identity: `0` for `+=`/`-=`, `none` for `min=`/`max=`)**, `m[k] ??= init` (lazy insert + mut lend), `m.get(k) -> ?V`, `m.remove(k) -> ?V`, `k in m`, `k not in m`, `for (k, v) in m.items`. Ordered `Map` also has `.first`, `.last`, `.first_ge(k)`, `.first_gt(k)`, `.last_le(k)`, `.last_lt(k)` (**all `-> ?(K, V)`**). (All hashes are per-process salted flat open-addressed tables.)
- **`Bitset<D>()` (large `D`) & `Subset<D>` (4-B `Index`, `D.K <= 25`):** `.add(v)`, `.remove(v)`, `.toggle(v)`, `v in s`, `v not in s`, `|`, `&`, `-`, `^`, `.len` (popcount), `s.shifted(k)`. `Subset<D>` constructors & iterations: `Subset<D>.empty`, `Subset<D>.full`, `s.first -> ?D`, `s.with(v)`, `s.without(v)`, **`for mask in Subset<D>`** (all `2^K` subsets), **`for v in mask`** (member elements `v: D`), **`for sub in mask.subsets`** (submasks).

---

## 10. Search, Sequences, Math & Randomness Prelude

- **6 canonical search names (plus `r.mid`):**
  - Binary search (monotone, debug-checked; `F64` takes `steps: 80`): `bsearch_first(r, ok) -> ?I`, `bsearch_last(r, ok) -> ?I`.
  - **Sorted lookup return types:** sequences (`List`, `[D]T`, slices; optional `by:`) `.first_ge(x)` / `.last_le(x)` return **index (`?Int` / `?D`)**; `Set`/`SortedBag` return **key (`?K`)**; `Map` returns **pair (`?(K, V)`)**.
  - Linear search: `first(it) -> ?T`, `a.find(x) -> ?I`, `min_by(it, key) -> ?T` / `max_by(it, key) -> ?T` (first wins ties).
- **Sequences & RNG:** `D.windows(k)`, stable `sort(mut a, by:)` / `sorted(it, by:)` (`by: desc` for descending), `reverse(mut a)`, `dedup(mut a)`, `rotate_left(mut a, k)` (signed Euclidean `k`; negative rotates right), `next_permutation(mut a) -> Bool`, `prev_permutation(mut a) -> Bool`, `let xs = compress(all)` (`xs.D`, `xs[v]`, `xs.at(val) -> xs.D`, `xs.first_ge(val)`), `prefix(a)` (`pre.sum(l..=r)` in 1D/2D), `scan(a, +)`, `runs(a)`, `xs.adjacent`, `a.items`, `sum(it)` (non-optional numeric iterables, type-zero seeded), `any(it)`, `all(it)`, `count(it)`, `Perm<V>` (`.inv()`, `.cycles()`), `Rng(seed)` (`.int(lo..hi)`), `rand(lo..hi)`, `shuffle(mut xs)`.
- **Math & geometry:** `abs`, `min`, `max`, `ceil_div(a, b)`, `gcd`, `lcm`, `isqrt`, `pow_mod(a, e, m)`, `Sieve(n)` (`.is_prime`, `.spf`, `.primes`, `.factor`), `factor(x) -> List<(Int, Int)>`, `divisors(x) -> List<Int>`.
  - **`Mod<P>`:** unified for `const` (`type Z = Mod<MOD>`) and runtime `let` (`type Z = Mod<M>`). Any `P >= 1` (incl. composite); 4-byte when `P <= 2^31 - 1`, `I128` above; one-way `Int` reduction; `+`, `-`, `*`, `/`, `**`, `.pow(e)`, `.inv`, `.try_inv() -> ?Z`.
  - **`Comb<P>()`:** immutable `let comb = Comb<MOD>()` (interior-mutability lazy growth) — `.fact(k)`, `.inv_fact(k)`, `.choose(n, k)`, `.arrange(n, k)`.
  - **`Pt<T = Int>`:** `+`, `-`, `a.dot(b)`, `a.cross(b)`, `a.norm2`, `turn(a, b, c)` (`Int`), `turn_eps(a, b, c, eps)`, `unit(angle)`, `ByAngle`.

---

## 11. Core Contest Data Structures & Graphs (`std`)

- **Range structures** (accept bare operators `+`, `*`, `^`, `&`, `|`, `min`, `max`, `gcd` OR `Monoid(id, op)`; empty/inverted ranges are valid no-ops except `SparseTable.fold`):
  - `Fenwick<D, Int>(a)` / `Fenwick2D<R, C, Int>()` — `.add(i, d)`, `.sum(l..=r)`.
  - `SparseTable<D, Int>(min, a)` — `.fold(l..=r) -> T` in O(1) (asserts non-empty range; no identity needed).
  - `SegTree<D, T>(+, a)` or `SegTree<D, T>(Monoid(id, op), a)` — `.set(i, v)`, `.fold(l..=r)`, **`.all`** (O(1) whole-domain fold), `.max_right(l: D.bounds, ok) -> D.bounds`, `.min_left(r: D.bounds, ok) -> D.bounds`.
  - `LazySegTree<D, T>(act, a)` (infers tag type `F` from `act`; stock tags: `AddSum`, `AddMin`, `AddMax`, `AssignSum`, `AssignMin`, `AssignMax`; custom **`Action(id, (new_f, old_f) => ..., (f, val, span) => ..., over: op)`** where `compose(new_f, old_f)` applies `old_f` then `new_f`) — `.apply(l..=r, tag)`, `.set(i, v)`, `.update(i, f)`, `.fold(l..=r)`, **`.all`**, `.max_right(l, ok)`, `.min_left(r, ok)`.
  - `Dsu<V>()` — `.unite(a, b) -> Bool`, `.find(v) -> V`, `.same(a, b)`, `.size(v)`, `.components`, `.groups() -> List<List<V>>`, `.mark()` / `.undo_to(snap)`.
- **Graphs, trees & strings:**
  - `Graph<V>` / `Graph<V, W>` — `Graph.undirected(es)`, `Graph.directed(es)`, `g[u]`, `bfs`, `dijkstra`, `topo`, `scc`, `condense`, `two_edge_components`, `biconnected_components`, `articulation_points`, `two_color`.
  - `Rooted<V>(g, root)` / `Rooted.from_parents(par)` — value-owned preorder domain `t.Pre`, `t.pos(v) -> t.Pre`, `t.preorder: [t.Pre]V`, `t.postorder`, `t.tour` (`(v, entering)`), `.parent(v)`, `.children(v)`, `.depth(v)`, `.size(v)`, `.lca(a, b)`, `.dist(a, b)`, `.is_anc(a, b)`, `.ancestor(v, k)`, `.lift_while(v, pred) -> ?V`, `.sub(v) -> Range<t.Pre>`, `.chains(a, b, edges?: Bool)`, `.fold_path(a, b, ds)`, `.apply_path(a, b, mut ds, tag)`, `.virtual_tree(keys)`.
  - `FlowGraph<Node>()` / `max_weight_matching<L, R>` — pair with payload-enum `Node`.
  - `z_function(s)`, `prefix_function(s)`, `SuffixArray(s)` (`.sa`, `.lcp`), `RollingHash(seq)` (`Str` or `[D]T`, `.get(l..r) -> HashVal`).
- *(Opt-in `std.marks.Marks<D>` [32-bit stamps] and advanced structures — `MonoDeque`, `Steps`, `TwoSat`, `Matrix`, `MinCostFlow`, `Dir8`, `palindromes`, `shortest`, `walk_back`, `mst`, `convex_hull`, `Rope`, `RangeKth`, `LiChao`, `PotentialDsu`, `Cover`, `Line`/`Hom`, `TwoStackFold`, `mo`, `Poly`/NTT, `XorBasis`, `Frac`, `Fixed`, `Tri`, `Counter` — live in `examples/local-lib/`.)*

---

## 12. Reserved Keywords (34 Total) & Decorators

- **Contestant (30, including 3 literals):** `let`, `var`, `const`, `fn`, `struct`, `enum`, `domain`, `type`, `use`, `pub`, `if`, `elif`, `else`, `for`, `while`, `in`, `match`, `return`, `break`, `continue`, `and`, `or`, `not`, `assert`, `mut`, `self`, `unchecked`, `true`, `false`, `none`.
- **Library-author only (4):** `trait`, `impl`, `lends`, `yield`.
- **`@` decorators (7 contestant + 3 library-author, plus the `@!multitest` file directive):** `@memo`, `@brute`, `@tags`, `@gen`, `@checker`, `@interactor`, `@grader`; library-author `@get`, `@test`, `@law`.
