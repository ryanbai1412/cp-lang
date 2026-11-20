# CP Lang v2: Unified Architecture Specification & Prototype Roadmap (Wave 2 Lock)

This document locks **CP Lang v2** — a single, contradiction-free, minimal, high-ergonomics language and standard-library specification. It reconciles every design artifact landed through commit `8668b7f`:

- **Wave 1 Architecture Synthesis** (`research/language-architecture-synthesis.md` — Batches 1–14, 16, and Blind Critic #1)
- **Stage 3 Subtraction & Concept-Budget Audit** (`research/subtraction-audit-1.md`, `d342632` — 18 cuts, 40-keyword budget)
- **Stage 4 Head-to-Head Syntax Bakeoff #1** (`research/syntax-bakeoff-1.md`, `49a5c1f` — 7 forks scored on the 25-problem benchmark suite)
- **Stage 5 Blind Critic #2 & #3 Audits** (`research/blind-critic-2-debug-pivot.md`, `900c171`; `research/blind-critic-3-cheatsheet-test.md`, `8668b7f` — debug/pivot & cheat-sheet-only contest tests)
- **Stage 6 Max-N Judge & Constant-Factor Audit #1** (`research/max-n-judge-audit-1.md`, `114443b` — 30 max-N judge benchmarks)
- **Prototype Bug-Fix Sprint** (`6f113c3`, `7d1c976` — `?T` ordering and tuple assignment)
- **Scout Batches 15 and 17–22** (`friction-backlog-batch-15.md`, `17`, `18`, `19`, `20`, `21`, `22` — flow/connectivity, statement-first waves, interactive/checkers/unusual I/O, DP/math, geometry/`F64`, hard strings/automata)
- **Foundational specs** (`decisions.md` and `plan/19-index-and-lends.md`)

`decisions.md` §16 and `docs/one-page-cheat-sheet.md` are synchronized to match this document 100%. Every construct has **one canonical spelling**, every tension between earlier stages is resolved below with concrete before/after CP Lang snippets, and §4 gives the exact prioritized checklist for bringing `proto/cpc/` and `stdlib/` up to v2 before locking changes into `crates/`.

---

## 1. Locked Concept Budget

A strong competitive programmer should memorize the entire contestant language from `docs/one-page-cheat-sheet.md` in 15 minutes at 2 AM and compile clean, zero-UB, max-N-fast solutions on the first try. CP Lang v2 locks every surface budget strictly:

- **40 reserved keywords total** — **32 contestant keywords** (including the 3 literals `true`, `false`, `none`; typical single-file solutions use ~25) plus **8 library-author keywords** (`trait`, `impl`, `where`, `lends`, `lend`, `law`, `inline`, `yield`). Permanently deleted: `by`, `with`, `undo`, `requires`, `ensures`, `invariant`, `stress`, `protocol`, `limit`, `dyn`, `do`, `pool`, `decorator`, `static`, `yields`. Batch 19's `require`, `hidden`, `export`, `extern`, `interactor`, `checker`, and `gen` are never reserved as hard keywords — they remain ordinary functions or `@` decorators.
- **4 numeric types:** `Int` (64-bit signed symmetric `-(2^63-1)..=(2^63-1)`, with range types like `0..=1e9` packing tightly to 1/2/4 bytes automatically), `U64` (wrapping 64-bit unsigned), `I128` (128-bit signed), and `F64` (64-bit IEEE double, quiet-NaN niche). Deleted: `I32`, `I64`, `U8`, `U32`.
- **8 + 3 language traits:** 8 auto-derived core traits (`Index`, `Seq`, `Copy`, `Eq`, `Ord` only when every field is `Ord`, `Hash`, `Show`, `Read` — zero `@derive`) plus 3 library-author traits (`Indexed`/`IndexedMut`, `Iterator`, `Gen`). Deleted from the trait surface: `Small`, `Relocate`, `Zero`, `Bits`, `OrdKey`, `Bounded`, `Niche` (internalized into compiler layout), `Shrink` (choice-stream shrinking), `Modulus`, `Handle`, `FenwickIndex`, `Window`, `BagWindow`. (`std.algebra.Reversible` is a library-only hook, not a compiler trait.)
- **7 + 2 `@` decorators:** `@memo`, `@multitest`, `@brute`, `@tags`, `@gen`, `@checker`/`@interactor`, `@grader`, plus library-author `@get` and `@test`. Deleted: `@derive`, `@solution`, `@local`, `@sample`, `@invariant`, `@soa`, `@wire`, `@final`, `@graded`.
- **2 predicates:** `assert` (always checked local + judge) and `assume` (local/debug/stress only, statically pure).
- **1 range type (`Range<I>`), 1 cleanup primitive (`defer`), 1 modular type (`Mod<P>`), 1 tree abstraction (`Rooted<V>`), 1 inline-`if` (`if c: a else: b`), 1 lambda (`x => expr`), 1 `match` arm (`Pattern => expr`), 8 search names.**

---

## 2. All 14 Cross-Stage Tensions Resolved

### T1. Default `Str`: `0..len`-indexed ASCII bytes + opt-in `[domain S]Char`

*Reconciles `decisions.md` §1.6, `plan/19 §4.4`, Wave 1 §2.3B, and Stage 3 against Stage 4 Fork 4 and Batch 19 `B19-9`.*

Stage 4 proved on four real string problems (`4331904`, `noip20p2`, `noi16p1`, `sac22cc3p5`) that auto-opening a nominal index domain `s.Idx` on every `Str` fights string algorithms: Z/KMP/SA/hashing constantly add lengths (`i + x.len`, `2 * w`, `q + w`) and land on the end boundary `s.len`, forcing constant bouncing between `s.Idx`, `s.Idx.bounds`, and `.to_idx` (403 vs. 325 keystrokes on `4331904`). Batch 19 independently showed that real parsing/simulation tasks (`noimps22p1`, `egoi23p3`) need a flat ASCII byte buffer with `.push(ch)`, `+`, `*`, slicing, `starts_with`, `split_once`, `trim_start`, and `parse_int`.

**Locked v2 rule:** `Char` stays 1 ASCII byte with zero UTF-8 anywhere (adding `is_alpha`, `is_alnum`, `digit(base = 10) -> ?Int`, `Char.digit(d)`, Batch 22 `X10`; `Scanner` is dropped per Batch 22 `X13`). Default `Str` is a flat `0..len`-indexed ASCII byte sequence (bounds-checked in debug, elided under `unchecked:`) supporting O(1) `s[i]`, slicing `s[l..r]`/`s[..k]`/`s[k..]` (`X2`), `for c in s`, `s.len`, `s.is_empty`, `s.push(ch)` / `s += t` (when `var`), `s + t`, `s * k`, `sep.join(it)`, `s.starts_with(x, at: i = 0)`, `s.split(sep)`, `s.split_once(sep)`, `s.trim_start(ch)`, `s.parse_int() -> ?Int`, and `Str(chars)`. **Strict value semantics (`X4`):** `Str` and `List` never silently alias on `var t = s` — last-use moves or explicit `s.copy()` apply uniformly so mutating `t[0] = 'J'` can never mutate `let s`! The `[domain]T` kernel package stays intact so multi-string DP (LCS, edit distance) opts into nominal character domains in one line with `let a: [domain A]Char = read()`.

Before (nominal `s.Idx` default — bouncing between `s.Idx` and `s.Idx.bounds` on `4331904`):
```cp
var dp: [s.Idx.bounds]?Int = none
dp[s.Idx.bounds.last] = 0
for i in s.Idx.rev:
    for (c, x) in dict:
        if let end = i.r(x.len) if s[i.l..end] == x:
            if let nxt = dp[end]: dp[i.l] min= 1 + nxt
```

After (CP Lang v2 — `0..len`-indexed ASCII `Str`, plus opt-in `[domain A]Char` when wanted):
```cp
var dp: [0..=s.len]?Int = none
dp[s.len] = 0
for i in (0..s.len).rev:
    for (c, x) in dict:
        if s.starts_with(x, at: i):
            if let nxt = dp[i + x.len]: dp[i] min= 1 + nxt

// Opt-in nominal character domains for 2-string LCS (zero i/j swap risk):
let a: [domain A]Char = read()
let b: [domain B]Char = read()
```

---

### T2. Sliding Windows, Two-Pointers & Keeping One `Range<I>` Type

*Reconciles Wave 1 §2.2, Stage 3 Cuts 7 & 14, and Stage 4 Fork 1.*

Wave 1 proposed `Window<I>`/`BagWindow<I>` traits and callback-driven `slide`/`slide_until(add:, drop:, until:)`/`MonoDeque`; Stage 3 Cut 14 kept callback `sliding(D, add:, drop:, ok:)` and deleted `slide_until` and `Range1<I>`. Stage 4 Fork 1 caught a real bug in callback `MonoDeque` on `noi05p1` (`p.to_idx` breaks on vertical/reversed grid tilts) and showed that imperative `MonoDeque` beats the 4-line callback version on both keystrokes (172 vs. 175) and symbol count (42 vs. 53), while explicit `while` loops beat multi-closure `sliding(add:, drop:, ok:)` on stateful two-pointers (`noi16p4`). Conversely, stateless `slide_until` won decisively on `coci10c7p4` (114 vs. 167 keystrokes) and eliminated the `hi < lo` empty-window bug.

To honor **both** Stage 3 Cut 7 (delete `Range1<I>`, keep one `Range<I>` type) and Stage 4 Fork 1 (no `w.first!` panic noise in `slide_until`), stateless `slide_until(r, ok)` takes `ok(l: I, r: I) -> Bool` and yields non-optional `(lo: I, hi: I)` endpoint pairs directly.

**Locked v2 rule:** Delete `Window<I>`, `BagWindow<I>`, `TwoStackFold`, `GroupWindow`, callback `sliding(D, add:, drop:, ok:)`, and `Range1<I>`. Ship plain imperative `Deque<T>`, imperative `MonoDeque<T>(min | max)` (`push(at, val)`, `drop_before(min_at)`, `clear()`, `best -> ?T`), fixed-width `D.windows(k)` + `sliding_min(a, k)`/`sliding_max(a, k)`, explicit `while` loops for stateful two-pointers, and stateless `slide_until(r, (l, r) => ok)` yielding `(lo, hi)`. Add `(lo..=hi).clamp(D)` for sweep bounds (Batch 21 `E7`).

Before (Wave 1 callback `Window` + buggy `p.to_idx` on `noi05p1`, and `Range1` `w.first!` noise on `coci10c7p4`):
```cp
for run in line.split(p => hall[p] == 'x'):
    for (p, best_prev) in run.slide(seconds + 1, MonoDeque(Greater, p => far[p]? - p.to_idx)):
        best[p] max= best_prev + p.to_idx

for w in levels.slide_until(until: w => delivers(levels[w.first!], levels[w.last!])):
    best min= levels[w.last!] - levels[w.first!]
```

After (CP Lang v2 — imperative `MonoDeque` + stateless `slide_until` yielding `(lo, hi)`, zero `Range1`, zero `!`):
```cp
var q = MonoDeque<Int>(max)
for i in 0..line.len:
    let p = line[i]
    if hall[p] == 'x': q.clear(); continue
    if let slid = far[p]: q.push(i, slid - i)
    q.drop_before(i - seconds)
    if let top = q.best: next[p] = top + i

for (lo, hi) in slide_until(0..levels.len, (l, r) => delivers(levels[l], levels[r])):
    best min= levels[hi] - levels[lo]
```

---

### T3. Bidirectional Generic Inference, `f::<T>()`, & Plain Unweighted Adjacency

*Reconciles `decisions.md` §2.13, Stage 3 Cut 9, Stage 4 Fork 2, and Batch 15 `N8`.*

`stdlib/graph.cp` and `stdlib/tree.cp` currently place unannotated neighbor closures before concrete `from:`/`root:` arguments and require every unweighted neighbor list to wrap each vertex in `(to: v)` records, forcing `bfs::<Cell>(u => [(to: v) for v in adj[u]], from: [src])`.

**Locked v2 rule:** Generic function definitions always declare type parameters explicitly (`fn f<D>(...)`; implicit single-letter generics stay deleted). At call sites, the type checker unifies concrete non-closure arguments (`from:`, `root:`, `g`) before checking closure bodies, and `std.graph`/`std.tree` put concrete domain-carrying arguments first (`bfs(from: [s], next: ...)`, `dijkstra(g, from: s)`, `Rooted(root: r, adj: ...)`, `Rooted(g, root: r)`). Unweighted graph and tree algorithms accept both `Graph<V, E>` and plain `V -> Iter<V>` (via a tiny internal `to`-projection or overload) with zero `[(to: v) ...]` allocations. Explicit `f::<T>()` remains the single unambiguous fallback for genuinely return-polymorphic calls like `parse::<Int>(tok)`; `f<T>()` stays a compile error.

Before:
```cp
let dist = bfs::<Cell>(p => [(to: q) for q in adj[p]], from: [office])
let tree = Rooted::<Pkg>(u => [(to: v) for v in needed_by[u]], root: Pkg.first)
```

After (CP Lang v2 — ~90% of turbofish and all `(to: v)` wrappers vanish):
```cp
let dist = bfs(from: [office], next: p => adj[p])
let tree = Rooted(root: Pkg.first, adj: u => needed_by[u])
let n = parse::<Int>(tok)          // turbofish kept only when return type is unbound
```

---

### T4. Product-Domain Grids: `type Cell = (Row, Col)` + `p.adj4` / `p.adj8`

*Reconciles Stage 3 (`one-page-cheat-sheet.md` §3), Stage 4 Fork 3, and Batch 19 `B19-10`.*

Stage 4 Fork 3 proved that `type Cell = (Row, Col)` deriving `Index` cuts grid-BFS keystrokes and symbol load in half compared to separate `(r, c)` tuples while composing seamlessly with dual grids (`town`, `high`), `Dsu<Cell>`, and `@memo` — unlike a bespoke `Grid<T>` type. It also exposed a silent 2 AM bug in Stage 3's `p.around()`: defaulting "around" to 4 neighbors causes silent WAs on 8-neighbor problems (`coci10c7p4`).

**Locked v2 rule:** Keep `type Cell = (Row, Col)` as a plain product `Index` (no dedicated `Grid<T>` type). Userland `std.grid` provides `Dir4`, `Dir8` (`.cw`, `.ccw`, `.back`, `Dir4.from("UDLR", ch)`), `p.step(d)`, `p.ray(d)`, explicit **`p.adj4`** and **`p.adj8`** (deleting `p.around()` / `p.around8()`), and zero-copy **`g.transposed`** / **`g.flipped`** views.

Before (`ccc18s3` / `coci10c7p4` — ambiguous `around()` and manual flip flag):
```cp
for q in p.around(): ...           // 4 or 8? silent WA on 8-neighbor grids
if flip: g[Row(j), Col(i)] = ch else: g[Row(i), Col(j)] = ch
```

After (CP Lang v2):
```cp
for q in p.adj4: ...               // unmistakable 4-neighbors
for q in p.adj8: ...               // unmistakable 8-neighbors
return construct(m, n, c, r)?.transposed()
```

---

### T5. Unambiguous Search: 8 Canonical Names, Retiring `first_where`

*Reconciles Wave 1 §2.3A, Stage 3 Cut 16, Stage 4 Fork 5, and Batch 18 `SF5`.*

`first_where` caused a proven silent wrong answer (`ioi19p2`) because it binary-searched under a linear-search name. Stage 3 cut 19 search names down to 7; Stage 4 Fork 5 proved that calling binary search as free functions eliminates mandatory `(1..=t).bsearch_first(...)` parentheses and precedence traps, and that `bsearch_first_some` prevents duplicating O(n) witness construction (`coci21c4p4`).

**Locked v2 rule:** Retire `first_where`, `last_where`, `bisect`, `partition_point`, `bsearch_last_some`, `a.where`, and `running_sum`. Lock **8 canonical search names** (plus `r.mid` on `Range<I>`):

1. Monotone binary search (debug-checked monotone on probed endpoints; callable as free functions on ranges and as methods on named domains; `bsearch_first`/`bsearch_last` also take `F64` ranges with `steps: 80`): **`bsearch_first(r, ok) -> ?I`**, **`bsearch_last(r, ok) -> ?I`**, **`bsearch_first_some(r, f) -> ?(I, W)`**.
2. Sorted lookup (uniform across `List`, `[D]T`, `Set`, `Map`, `SortedBag`): **`xs.first_ge(x) -> ?I`** and **`xs.last_le(x) -> ?I`** (plus `.first_gt`, `.last_lt` on ordered sets/maps).
3. Linear search (O(n)): **`first(it) -> ?T`**, **`a.find(x) -> ?I`**, and **`min_by(it, key)`** / **`max_by(it, key)`**.

Before:
```cp
let (reach, built) = (0..=2e14).first_some(parks)!
let mins = (1..=t).first_where(done_in) ?? -1
let bad = (0..n).first_where(i => a[i] != want[i])    // silent WA!
```

After (CP Lang v2):
```cp
let (reach, built) = bsearch_first_some(0..=2e14, parks)!
let mins = bsearch_first(1..=t, done_in) ?? -1
let bad = first(i for i in 0..n if a[i] != want[i])
```

---

### T6. Algebraic `Index` (5 O(1) `Copy` Shapes) & Arenas Without `Pool<T>`

*Reconciles Wave 1 §2.1A & §2.1C, Stage 3 Cuts 5 & 6, Batch 15 `N1`, and Batch 18 Collision 1.*

Wave 1 proposed exponential `[D]E: Index` and a 5-concept `Pool<Node<_>>` / `PoolMap` / `Handle` arena subsystem. Stage 3 Cuts 5 and 6 proved `[D]E: Index` breaks the invariant that every `Index` is strictly O(1) and `Copy` (for just 2/710 corpus problems), while `Pool<Node<_>>` adds the only type-level knot-tying syntax in the language when `type Id = 0..=MAX` already gives 4-byte handles and 4-byte niche-packed `?Id`. Meanwhile, Batch 15 (`N1`) and Batch 18 (Collision 1) confirmed that **sum `Index` on payload `enum`s** is the #1 unlock for typed network flow, matching, and multi-role graphs (`noi09p4`, `dmopc20c7p5`, `noi06p4`) without adding a separate `domain Node = A | B` declaration syntax.

**Locked v2 rule:** Exactly **5 finite type constructors** derive `Index` automatically (strictly O(1), `Copy`, computed on demand so unused large enums never overflow): `Bool`, unit and payload `enum`, `?T` (for `T: Index`), bounded ranges `lo..hi` / `lo..=hi` (**structural plain `Int`-indexed arrays**: identical written bounds in scope unify across function calls, while `domain D = 1..=n` opts into nominal index safety — fixing Blind Critic 3's `expected [range]Int, got [range]Int` error), and tuples/records `(A, B)`, alongside `Subset<D>` (`D.K <= 25`). Whole-array resets use **`a.fill(x)`** (including `seen.fill(none)`), and **`compress(all)`** returns a `Compressed<T>` object with fresh domain `xs.D`, reverse value lookup `xs[v] -> T`, and value-to-index lookup **`xs.at(val) -> xs.D`** / `xs.first_ge(val)` (eliminating both positional `id[2 * i]` guesses and `coord.first_ge(x)!` noise, Blind Critic 3 Top Fix #4). Delete `[D]E: Index` (use `Subset<D>`, `@memo(hash)`, or an explicit 4-line encoder) and delete `Pool<T>`, `Pool<Node<_>>`, `PoolMap`, `Pool.reusable`, and `Handle` (use `type Id = 0..=2_000_000`, `List<Node>`, `.push_id(x)`, `??=`, and `.freeze() -> (domain D, [D]T)`).

Before (`dmopc20c7p5` manual offsets + Wave 1 `Pool<Node<_>>` knot-tying):
```cp
domain Node = 0..n + m + 2
net.add(Node(n + e.to_idx), Node(u - 1), INF)

var nodes = Pool<Node<_>>()
```

After (CP Lang v2 — payload `enum` as `Index` + range-typed `Id` on `List`):
```cp
enum Node: Source | Sink | Item(i: Item) | Pair(e: Pair)
var net = FlowGraph<Node>()
net.add(Pair(e), Item(u), INF)

type Id = 0..=2_000_000
var nodes: List<TrieNode> = [TrieNode()]
o = nodes[o].kid[bit] ??= nodes.push_id(TrieNode())
```

---

### T7. Bare Ops, `LazySegTree.max_right`, & `Rooted.fold_path`/`apply_path` + `Reversible`

*Reconciles Wave 1 §2.1D & §2.3E, Stage 3 Cuts 15 & 18, Stage 4 Fork 7, Batch 15 `N4`, and Batch 18 `SF1` & `SF3`.*

Stage 3 Cut 15 collapsed 20 `stdlib/algebra.cp` wrapper names into bare operators (`+`, `*`, `^`, `&`, `|`, `min`, `max`, `gcd`), `(id, op)` pairs, `desc(x)`, and `Action` (for `LazySegTree`). Stage 3 Cut 18 deleted dual-tree `Hld` and `LazyHld`, leaving raw `tree.chains(a, b)`. Stage 4 Fork 7 then demonstrated that raw `tree.chains` alone invites silent `c.rising` flip-order WAs on non-commutative path queries (`noi21p1`), while dual-tree `Hld` wastes 2× memory and doubles update time.

**Locked v2 rule:** Keep bare ops + `(id, op)` + `desc(x)` + `Action` (stock tags `AddSum`, `AddMin`, `AddMax`, `AssignSum`, `AssignMin`, `AssignMax`), and expose `.max_right(from, ok)` / `.min_left(to, ok)` on **both** `SegTree` and `LazySegTree` (closing Batch 18 `SF1` and matching `stdlib/segtree.cp`). Keep `Rooted<V>` as the **single tree abstraction** (DFS spanning tree/forest, flat CSR `kids`, lazy LCA sparse table & jump table, zero-alloc `chains`), retire `Hld` and `LazyHld`, and add generic **`tree.fold_path(a, b, ds)`** and **`tree.apply_path(a, b, mut ds, tag)`** (with optional `edges?: Bool`) working over any `tree.Pre`-indexed DS. Non-commutative monoids provide a 1-line `rev()` via userland `std.algebra.Reversible` (commutative ops default to identity) so `fold_path` flips rising segments automatically using **one segment tree (1× memory)**. Add `tree.postorder`, `tree.lift_while(v, pred)`, and `tree.virtual_tree(keys)` on `Rooted<V>` (`N4`, `SF3`).

Before (`stdlib/algebra.cp` wrapper ceremony + dual-tree `Hld` / manual `chains` flip):
```cp
sort(mut xs, by: Order<(Int, Int)>.by(p => p.1))
var st = SegTree<V, ?Int>.from(opt(Semigroup<Int>((a, b) => max(a, b))), init)
var hld = LazyHld(tree, EdgeRun.monoid, AssignColor.action, init)   // 2x memory!
```

After (CP Lang v2 — bare ops + single-tree `Rooted.fold_path`/`apply_path` + `Reversible`):
```cp
sort(mut xs, by: p => p.1)
var st = SegTree<V, ?Int>(max, init)
var seg = LazySegTree<tree.Pre, EdgeRun>(EdgeRun.monoid, AssignColor, init)
tree.apply_path(a, b, mut seg, now)
let ans = tree.fold_path(a, b, seg).heavy        // 1x memory; auto-flips rising chains
```

---

### T8. Scope Cleanup (`defer`), Plain Nested `fn`, & Parallel Tuple Assignment `(a, b) = (x, y)`

*Reconciles Wave 1 §2.3D, Stage 3 Cuts 1 & 3, commit `7d1c976` (`decisions.md` §15.12), Batch 18 Collision 4, and Batch 20 `W6`.*

Stage 3 deleted `with` (both struct update and scope guards), `undo` blocks, and `mut fn` (`0` uses in 299 ports; `with` collides with `Subset.with(x)`, and weakening `lends` to allow skipping `lend` breaks its one-path law). Separately, commit `7d1c976` rejected `(a, b) = (x, y)` as a temporary compile error until Wave 2 decided whether parallel tuple assignment belongs in the language.

**Locked v2 rule:**
- **`defer` is the sole scope-cleanup primitive** (`let snap = dsu.mark(); defer dsu.undo_to(snap)`). `lends` keeps its unbroken single-`lend` law (with post-`lend` write-back for `SegTree`/`LazySegTree`).
- **Plain nested `fn` mutates outer `var`s directly** (`mut fn` deleted); the compiler tracks captured reads/writes automatically and forbids escaping.
- **Parallel tuple assignment `(a, b) = (x, y)` IS locked into v2** (resolving `15.12` and Batch 20 `W6`): a tuple of distinct mutable places on the LHS evaluates every RHS expression first into temporaries and then writes each place — making swaps `(a, b) = (b, a)`, extended Euclid `(r0, r1) = (r1, r0 - q * r1)`, Li Chao descent `(node, hi) = (..., mid)`, and rolling DP updates `(prev, cur) = (cur, prev)` one clean line. Duplicate or non-disjoint LHS places (`(x, x) = ...` or unproven `(a[i], a[j]) = ...`) are compile errors.

Before (`noi02p4` workaround after `7d1c976`):
```cp
var (r0, r1, x0, x1) = (a, b, 1, 0)
while r1 != 0:
    let q = r0 / r1
    let nr = r0 - q * r1; r0 = r1; r1 = nr
    let nx = x0 - q * x1; x0 = x1; x1 = nx
```

After (CP Lang v2):
```cp
var (r0, r1, x0, x1) = (a, b, 1, 0)
while r1 != 0:
    let q = r0 / r1
    (r0, r1) = (r1, r0 - q * r1)
    (x0, x1) = (x1, x0 - q * x1)
```

---

### T9. Coherent `?T` Ordering & Uniform Optionality

*Reconciles commit `6f113c3` (`decisions.md` §15.11), Stage 3 Part IV #5, Stage 5 Blind Critic 2, Batch 17 `W13`, and Batch 20 `W11`.*

Commit `6f113c3` added C++ `operator<` on `Opt<T>` (`none` first) so structs containing `?Int` compiled under `g++`, while keeping `a < b` on two `?T` a compile error. That split creates a silent 2 AM disaster: `MinHeap<(?Int, V)>` silently pops unreachable (`none`) entries FIRST (treating `none` as `-infinity`), even though Dijkstra needs `+infinity`! Separately, both Blind Critic 2 (`cco07p2`) and Batch 20 (`W11`) tripped over an invisible special rule where `min(... for k in 0..6)` returned non-optional `T` while `min(... for j in I.first..i)` inside `if i != I.first:` returned `?T`, forcing contestants to guess whether `!` is required or forbidden.

**Locked v2 rule:**
- **`?T` and `Mod<P>` do NOT implement `Ord` (supersedes `15.11`, fixes Blind Critic 3):** Neither `a < b` nor default `sort`/`MinHeap`/`Map` ordering accepts `?T` or `Mod<P>`. Auto-derived `Ord` on a `struct`, record, or tuple is synthesized **only when every field implements `Ord`** (and C++ codegen emits `operator<` only for `Ord` types, fixing `List<(Int, Z, Int)>` and `?Int`-field structs in `g++` cleanly without fabricating an order on `?T`). When sorting or heap-ordering optional fields, state where `none` goes explicitly with `by:` or `??`. `min`, `max`, `min=`, and `max=` continue to skip `none` on **either** side (`ans min= dist[v]` works even when `ans` is plain `Int` and `dist[v]` is `?Int`, Blind Critic 3 Top Fix #5).
- **Uniform optionality:** `v + k`, `v - k`, `xs.last`, and `min(gen)`/`max(gen)` **always** return `?T` (delete the inconsistent constant-range `min` exception). Three complementary features eliminate most `!` sites cleanly without fragile type-changing smart casts: (1) open/half-open range queries on all range structures (`st.fold(..h[i])`), (2) `if let` as an expression (`let below = if let p = h[i] - 1: ... else: 0`), and (3) `while let ... and`. When `!` is written, flow facts elide the runtime check if proven safe, and optional-mismatch diagnostics carry the column, name the optional sub-expression, and print the parenthesized fix (`a + (max(...) ?? 0)` instead of `a + max(...) ?? 0`).

Before (Blind Critic 2 `dpq` + `6f113c3` silent `none`-first hazard):
```cp
best[i] = a[i] + max(best[j] for j in I.first..i if h[j] < h[i]) ?? 0   // precedence trap
var pq = MinHeap<(?Int, V)>()          // 6f113c3 silently popped `none` first!
```

After (CP Lang v2):
```cp
best[i] = a[i] + (max(best[j] for j in ..i if h[j] < h[i]) ?? 0)        // error prints exact parens
let below = st.fold(..h[i])            // half-open range: zero empty-prefix optional
sort(mut xs, by: x => (x.d == none, x.d ?? 0))   // explicit none-last order
```

---

### T10. `F64` Semantics & Exact vs. Approximate Geometry

*Reconciles `decisions.md` §1.5, Stage 3 Cut 8, Wave 1 §2.1F, and Batch 21 (`E1`–`E6`).*

Batch 21 ported six real-valued and four exact geometry problems and surfaced every floating-point footgun in contests: bare `print(x)` emitting `1e-07` or `-0.00`, `Int(-2.7)` disagreeing with C++ truncation, silent NaN poisoning `sort`/`Heap`, and hidden global `eps` tolerances.

**Locked v2 rule:** `F64` is the sole floating-point type (8-byte IEEE double; no `long double`).
- `1e18` stays `Int`; only literals with `.` are `F64`. Integer literals (`0`, `1`) unify with `F64` in typed positions (`var dp: [P]F64 = 0`), while `Int` variables require explicit `F64(x)` (preventing accidental Euclidean-vs-float `/` confusion).
- `Int(x)` on an `F64` is a compile error; write `floor(x)`, `ceil(x)`, `round(x)`, or `trunc(x)`.
- NaN and infinity trap where created in checked runs (`0.0 / 0.0`, `1.0 / 0.0`, `sqrt(-1.0)`, `inf - inf`, overflow to infinity), so quiet NaN is the 8-byte `?F64` niche (`sizeof(?F64) == 8`) and `<` is a total order (`sort`, `MinHeap`, `dijkstra` work unchanged).
- Bare `print(x)` of an `F64` is a compile error suggesting `"{x:.6f}"` (round half away from zero, never `-0.00`, never scientific notation).
- `==`/`!=` on `F64` is a lint suggesting `close(a, b, eps)` / `sign(x, eps)` (exempting literal `== 0.0`); `eps` is always explicit (no hidden default, no `~=` operator).
- Listed math intrinsics (1 C++ `<cmath>` call each): `sqrt`, `sin`, `cos`, `tan`, `atan2`, `exp`, `ln`, `hypot`, `floor`, `ceil`, `round`, `trunc`, and `PI`.
- `std.geometry` makes `Pt<T = Int>` generic (`Pt` aliases `Pt<Int>`), keeps exact `turn(a, b, c)` `Int`-only, and adds `turn_eps` and `unit(angle) -> Pt<F64>`.

---

### T11. Predicates, Stress Hooks, Checkers/Interactors/Graders, & Symmetric I/O

*Reconciles Wave 1 §4, Stage 3 Cuts 2 & 11, Stage 4 Fork 6, and Batch 19 (`B19-1`–`B19-11`).*

Wave 1 proposed a `stress x in 0..=3 by 0.25` grammar DSL and kept `protocol` / `limit` / `@wire` / `@final`. Stage 3 Cut 2 & Cut 11, Stage 4 Fork 6, and Batch 19 `B19-11` all independently converged on deleting BOTH DSLs! Batch 19 also showed how `@checker` needs `read_output()` for variable-size/multi-test outputs (`coci21c6p3`, `dmopc21c5p4`), how IOI graders span both two-phase `extern`/`export` (`ioi16p5`) and reversed-control stateful callbacks (`ioi14p3`), and how unusual I/O needs symmetric `none:` sentinels, `read_line()`, `read_until`, and char-range `@tags`.

**Locked v2 rule (zero new keywords beyond the 40-keyword budget):**
- **2 predicates:** `assert cond` is ALWAYS checked (local + judge; lints against `mut` inside `cond`, preventing `NDEBUG` side-effect erasure, `B19-8`). `assume cond` is local/debug/stress only (stripped on judge; statically **rejects** any `mut` or I/O side effect in `cond`, replacing `requires`, `ensures`, `invariant`, and `read() where`).
- **3 orthogonal stress hooks (NO `stress ... by ...` DSL, NO `Distinct`/`Sorted` wrappers):**
  1. Left-to-right dependent bounds everywhere (`let (n, k): (1..=2e5, 1..=n) = read()`, `[q](l: V, r: l..=V.last)`), plus `Perm<V>` and `Tree<V>` (and plain helper functions in `std.gen` like `tree_edges`, `connected_edges`, `grid_maze`).
  2. Typed `read(gen: f)` (including coupled tuple reads `let (m, walls) = read(gen: () => gen_walls(pts))` fixing `ioi07p2`, and optional `witness val` inside a generator so constructive solutions printing `none`/`IMPOSSIBLE` on a witnessed case fail immediately). `assume` validates `gen:` output and reports `"generator broke assume at line X"` separately from solution crashes.
  3. Whole-input `@gen fn make_case(size: Int): ...` for coupled/grammatical/multi-test inputs (`ceoi22p2`, `dmopc21c5p4`), shrunk automatically via **choice-stream shrinking** over every `rand(...)` draw.
- **Verification & graders under 3 `@` decorators (NO `protocol`, `require`, `export`, `extern` keywords):**
  - `@checker fn check(): ...` inherits top-level input bindings, reads solution stdout via `read_output()` (handling variable-size and multi-test outputs), and asserts with `assert`.
  - `@interactor fn judge(): ...` handles local stdio interactive runs paired with expression-form `ask("?", moves, limit: 2)`.
  - `@grader` covers all three IOI/DMOJ ABIs cleanly: bodyless `@grader fn add_element(x: Str)` = external judge function; `@grader fn restore_permutation(...) -> List<Int>:` = exported entry function (`cpl build --grader` omits `main()`); `@grader struct Game:` = stateful multi-callback grader (`initialize`, `has_edge`) instantiated fresh per `cpl stress` trial.
- **Symmetric unusual I/O:** `none:` sentinel on `read(none: "X")`, `print(ans, none: "IMPOSSIBLE")`, and `read_output(none: -1)`; `read_line()` (after a token `read()`, skips trailing whitespace to the next line; empty line is `""`; strips `\r`, Batch 22 `X1`); `read_until(sentinel)` (`for (r, c) in read_until((0, 0)):`); char-range payload `@tags('.', '*', '1'..='9')`; and `read()` allowed in expressions and assignments (`score[r, c] = read()`).

Before (`plan/16` `protocol` DSL + Wave 1 `stress ... by ...` DSL + `today/ccc04s5.cp`):
```cp
protocol Guess:
    limit 20
    @wire("?") fn ask(x: Int) -> Char
    @final @wire("!") fn answer(x: Int)

let sheep: [S](x: F64, y: F64) = read()
    stress x in 0..=3 by 0.25, y in 0.25..=3 by 0.25
```

After (CP Lang v2 — zero DSLs, zero extra keywords):
```cp
let resp: Char = ask("?", mid, limit: 20)
print("!", ans)

let sheep: [S](x: F64, y: F64) = read(gen: () =>
    [S] _ => (F64(rand(0, 12)) * 0.25, F64(rand(1, 12)) * 0.25)
)

@tags('.', '*', '1'..='9')
enum Cell: Empty | Wall | Coin(val: 1..=9)
```

---

### T12. Fixing `cpl stress` / `locate` False-OK Semantics

*Resolves Stage 5 Blind Critic 2 Top Fixes #1–#2, Batch 20 `W7`/`W13`/`W15`, and Batch 21 `E9`.*

Blind Critic 2 twice received `"stress OK"` on wrong programs: a wrong grid DP (`sac22cc2p4`) passed 1,900 tests because `stress`/`locate` capped sizes at 8×8 without saying so, and a wrong tie-break (`ccc02s4`) passed 500 tests because outputs were flattened into whitespace tokens. Meanwhile, `stress` ran without `-DLOCAL`, parameterless `@brute` ran inside `main()` on judge builds (`W7`), and `@brute` bodies spammed O(n²) warnings on every build (`W15`).

**Locked v2 rule:** Unify `stress` and `locate` into **one honest `cpl stress` command**:
1. Defaults to in-file `@brute(main)` / `@checker` / `@interactor` when no external reference binary is passed (and `cpl build` strips all `@brute`, `@gen`, `@checker`, `@interactor`, `assume`, and `debug` blocks completely — parameterless `@brute` never enters `main()`).
2. Escalates sizes automatically (`1..=max_size`, default `max_size = 20`, configurable via `--max-size N`).
3. Compares output **line-by-line** (trimming trailing whitespace per line) by default unless `@checker` is present, with `--close <eps>` for absolute/relative float token comparison (`E9`).
4. Always builds with local checks enabled (`-DLOCAL` / debug checks on) so `assert`, `assume`, bounds, and overflow traps fire during stress.
5. Shrinks every mismatch via choice-stream shrinking to the minimal valid failing case, and replays **`debug(...)`** output from both solution and oracle side-by-side on the shrunk input.
6. Prints an honest summary on pass: tests passed, max size reached, `assume`-rejected count (with a loud warning if >50% rejected), comparison mode (`line-by-line` / `--close` / `@checker`), and output diversity (e.g. `188/200 printed -1`).
7. Suppresses complexity warnings inside `@brute`, `@gen`, `@checker`, `@interactor`, and `assume` bodies, and warns if a `@brute`-tested function mutates a top-level `var` (`W13`). Default generators never emit empty `Str`, generate permutations for `Perm<V>`, and mix structured maze/corridor grids alongside random cells.

---

### T13. Unified Collection Verbs & Core `std` vs. `examples/local-lib/` Split

*Reconciles Stage 3 Cuts 17 & 18 with Batches 15 (`N2`–`N5`, `N7`, `N11`), 18 (`SF1`–`SF5`), 20 (`W10`), and 21 (`E6`–`E8`).*

- **One verb vocabulary across every container:** `.len` and `.is_empty` are always properties (never `.len()`), `.clear()` empties any container, sequential ends use `.push`/`.pop` (plus `.push_front`/`.pop_front` on `Deque`, `.peek` on heaps), ordered ends use `.first`/`.last`/`.pop_first()`/`.pop_last()`, ordered predecessor/successor uses `.first_ge`/`.first_gt`/`.last_le`/`.last_lt`/`.rank`/`.nth`, sets/bitsets/subsets use `.add`/`.remove`/`x in s`/`| & - ^` (with signed `Bitset.shifted(k)` / `Subset.shifted(k)`, `W10`), and maps use `m[k]`, `m[k] f= x` (monoid identity), `(m[k] ??= init)` (no silent method auto-vivification, rejecting `N7`'s `m[k].push(x)`), `m.get(k)`, `m.remove(k)`, `k in m`, `m.items`.
- **Core `std` (high-frequency contest bread-and-butter, fits on one cheat sheet):** `List`, `Deque`, `MonoDeque`, `MinHeap`, `MaxHeap`, `Set`, `SortedBag`, `HashSet`, `Map`, `HashMap`, `Bitset`, `Subset`, `Fenwick`, `Fenwick2D`, `SparseTable`, `SegTree`, `LazySegTree`, `Steps`, `Dsu` (with `.mark()`/`.undo_to(snap)`), `Graph`, `bfs`, `dijkstra`, `shortest` (auto 0-1 BFS), `topo`, `topo_prefix`, `scc`, `condense`, `two_edge_components`, `biconnected_components`, `articulation_points`, `two_color`, `mst`, `Rooted` (with `fold_path`, `apply_path`, `lift_while`, `virtual_tree`, `postorder`), `FlowGraph`, `MinCostFlow`, `max_weight_matching`, `TwoSat`, `z_function`, `prefix_function`, `palindromes`, `SuffixArray`, `RollingHash`, `std.grid`, `Mod<P>`, `Comb<P>`, `Matrix`, `Pt<T>`, `convex_hull`, `sieve`, `factor`, `Perm`, `compress`, `prefix`, `scan`, `runs`, `slide_until`, `rotate_left`, `next_permutation`.
- **`examples/local-lib/` (advanced / 1–2% structures proving Goal G8):** `Rope`, `RangeKth`, `LiChao`/`LiChaoArena`, `PotentialDsu`, `Cover<Y>`, `Line`/`Hom`, `TwoStackFold`, `mo`, `Poly`/NTT, `linalg`/`XorBasis`, `Functional`, `Congruence`, `Fixed<D>`, `Tri<D>`, `Powers`, `dnc_opt`, `knuth`, `Hash2D`, `Trie<A>`, `SuffixAutomaton<A>`, `fold_balanced`, `fold_others`, `overlaps`, `borders`, `sos_sub`/`sos_sup`, `Counter` (just `HashMap<T, Int>` + `(cnt[x] ??= 0) += 1`), `Frac`. (`Scanner` is dropped per Batch 22 `X13`.)

---

### T14. Max-N Judge Survival & Compiler-Trust Invariants

*Incorporates Stage 6 (`max-n-judge-audit-1.md`), Batch 18 §(e), Batch 19 `B19-8`, and Batch 20 `W5`.*

Every emitted C++17 submission must satisfy these non-negotiable invariants:

1. **Zero `using namespace std;` in emitted C++17** (`P1`, `B19-8`) — qualifies `std::` explicitly so user identifiers (`extent`, `rank`, `norm`, `count`, `next`, `left`, `right`, `move`, `size`) never collide with C++ headers.
2. **64-bit shift literals (`1LL` / `cp_shl`)** (`W5`) — fixes the silent 32-bit `1 << k` UB in `btoi19p2`.
3. **8-byte niche-packed `?Int` (`LLONG_MIN`) and `?F64` (quiet NaN)** (Stage 6 #4) — eliminates the 1.47× `ceoi18p1` and 1.52× `cco18p1` bandwidth/RSS penalties.
4. **Flat `@memo` array lowering for ALL algebraic `Index` parameter tuples** (`Bool`, `Enum`, `lo..=hi`, `(Row, Col)`, `Subset<D>`) up to `2^26` slots, allocated once (Stage 6 #1) — fixes `ioi04p4`'s 31.86× TLE cliff and `dpp`'s 3.92× regression.
5. **Per-process salted open-addressed flat hash table** replacing `std::unordered_map` in `HashMap`/`HashSet` (Stage 6 #2) — fixes both the 44.5× splitmix64 anti-hash exploit and `noi01p5`'s 198 MB MLE.
6. **Lazy `Rooted` LCA sparse table & jump table + flat CSR `kids` + direct unweighted constructor + zero-alloc `chains`** (Stage 6 #3) — fixes 6 tree RSS regressions (1.8×–2.7×) and `dpv`'s 1.91× slowdown.
7. **4-byte `uint32_t` `Mod<P>` when `P <= 2^31 - 1`**, **lazy-sorted `vector` for `Steps`**, **`vector<uint8_t>` never `vector<bool>`**, **single-pass fused `CpPrefix<T>`**, **template-parameterized `SegTree` ops** (no `std::function`), **small-domain stack buffer for compile-time-tiny domains (`1..=5`, `btoi17p2`)**, and **full `@part` tree-shaking + stripping `CP_GEN`/`assume`/`debug`/`@brute`** so typical judge C++17 files are 4–10 KB (always < 64 KB) with the original `.cp` source in a top comment header.

---

## 3. Minimal Compiler Core vs. Userland Separation (Goal G8)

Everything in CP Lang v2 sits cleanly in two layers so the compiler stays small, trustworthy, and easy to port to `crates/`:

- **Minimal compiler core (`plan/19 §7` + v2 completions):**
  1. Scalar & numeric layer (`Int`, `U64`, `I128`, `F64` with quiet-NaN niche, `Bool`, `Char`, range types, Euclidean `/` & `%`, 64-bit shifts, checked arithmetic, listed `<cmath>` intrinsics).
  2. Nominal structs/enums (auto-derived `Eq`, conditional `Ord`, `Hash`, `Show`, `Read`, `Gen`, `Copy`, and 5-shape algebraic `Index`) + structural tuples/records + `?T` niche layout.
  3. Traits with associated types, `const`s, and runtime-frozen `let`s (`3.8`), bidirectional generic inference + `f::<T>()`, fresh index binders (`domain D` and `[domain]T`), and `Indexed`/`IndexedMut`.
  4. Lexical borrows (`mut`, `lends`/`lend` with post-`lend` write-back, second-class lent values, `law disjoint`, `inline` callbacks), `defer`, value-copying destructuring, parallel tuple assignment, `??=`, and `x f= y`.
  5. File-as-body control flow (`if`/`elif`/`else`, `if let`/`match` expressions, `for ... if`, `while let ... and`, `yield` push/pull generators, `unchecked:`, `assert`, `assume`, `debug`) plus the 7 `@` decorators (`@memo`, `@multitest`, `@brute`, `@tags`, `@gen`, `@checker`/`@interactor`, `@grader`) and `Buffer<T>`/`read`/`print`/`ask` I/O primitives.
- **Pure userland (`stdlib/` and `examples/local-lib/`):**
  `Array<I, T>`, `Slice`, `Range<I>`, `Bounds<I>`, `Str` methods, `List`, `Deque`, `MonoDeque`, `MinHeap`, `MaxHeap`, `Set`, `SortedBag`, `HashSet`, `Map`, `HashMap`, `Bitset`, `Subset`, `Perm`, `Mod<P>`, `Comb<P>`, `Matrix`, `Pt<T>`, `std.grid`, `Fenwick`/`Fenwick2D`, `SparseTable`, `SegTree`, `LazySegTree`, `Action`, `Reversible`, `Steps`, `Dsu`, `Graph`, all graph algorithms, `Rooted` (`fold_path`, `apply_path`, `lift_while`, `virtual_tree`), `FlowGraph`/`MinCostFlow`/`max_weight_matching`, `TwoSat`, all string algorithms, `bsearch_*`, `slide_until`, `compress`, `prefix`, `scan`, `runs`, `std.gen`, and every advanced structure in `examples/local-lib/`.

---

## 4. Prioritized v2 Prototype Implementation Checklist (`proto/cpc/` & `stdlib/`)

Execute in strict priority order on the prototype (`proto/cpc/`, `proto/cpi/`, `proto/cp.py`, `stdlib/`). Keep `crates/` untouched until this checklist is green and verified by Stage 5 & Stage 6 regression gates; then mirror the locked v2 spec into `crates/` in one clean pass.

### Priority 0 — Correctness, Compiler-Trust, Sheet-vs-Prototype Parity & False-OK Fixes (`proto/cpc/`, `proto/cpi/`, `proto/cp.py`)

- [ ] **P0.0 First-page cheat-sheet parity & CI gate (`proto/cpc/`, `proto/tests/` — Blind Critic 3 Top Fix #1 & #2):**
  - [ ] Fix **tuple `read()`** (`let (n, m): (Int, Int) = read()`), **string interpolation** (`print("{a} {b}")` — currently silently prints literal braces!), and **`for ch in s`** over `Str` (Errors 1–5 & 23 in Blind Critic 3).
  - [ ] Make anonymous range arrays **`[0..n]T` / `[0..=n]T` structural** across function boundaries (fixing `expected [range]Int, got [range]Int` in `cmt21r1xym` and `aac5p5`), and support **`a.fill(x)`** (incl. `seen.fill(none)`) + **`ans min= opt`** when RHS is `?T` + **`MinHeap.is_empty`** + **`sort(mut xs, by: ...)`**.
  - [ ] Extract every fenced block from `docs/one-page-cheat-sheet.md` into `proto/tests/` and gate CI on compiling them so sheet and prototype never diverge again.
- [ ] **P0.1 Remove `using namespace std;` in `proto/cpc/runtime.hpp` & `compiler.py`:** Qualify `std::` explicitly so user names (`extent`, `rank`, `norm`, `count`, `next`, `left`, `right`, `move`) never clash with C++ headers (`P1`, `B19-8`).
- [ ] **P0.2 Fix 64-bit shift literal codegen & `var t = s` value semantics (`proto/cpc/compiler.py`, `proto/cpi/`):**
  - [ ] Emit `1LL << k` / `cp_shl` on all integer shifts so `1 << 32` is never 32-bit UB (`W5`, `btoi19p2`).
  - [ ] Fix `var t = s` silent aliasing on `Str` and `List` in `proto/cpi/` (`X4`, Batch 22) and add a regression test to `proto/cpi/checks_test.py` ensuring mutating `t[0]` never mutates `let s`.
- [ ] **P0.3 Lock coherent `?T` & `Mod<P>` ordering (`proto/cpc/compiler.py`, `runtime.hpp`):** Revert `Opt<T>::operator<` from `6f113c3`; emit `operator<` on structs/records/tuples ONLY when every field implements `Ord` (never `?T`, never `Mod<P>` — fixing `List<(Int, Z, Int)>` in Blind Critic 3); reject `MinHeap<?T>` / `sort(List<?T>)` cleanly with a help message showing `by: x => (x == none, x ?? 0)` (`16.9`).
- [ ] **P0.4 Implement parallel tuple assignment `(a, b) = (x, y)` (`proto/cpc/compiler.py`):** Evaluate all RHS expressions into temporaries before assigning to distinct mutable LHS places; reject duplicate/non-disjoint LHS places (`16.9`, `W6`, updates `proto/tests/bad/tuple_assign.cp`).
- [ ] **P0.5 Fix remaining silent-WA prototype bugs (`proto/cpc/compiler.py`):**
  - [ ] Rename `first_where`/`last_where` to free-fn `bsearch_first`/`bsearch_last`/`bsearch_first_some` with debug monotonicity checks (`9-T12`, Fork 5).
  - [ ] Fix `read()` inside a top-level `while` loop swallowing subsequent statements (`P3`).
  - [ ] Fix `Subset - Subset` returning `Subset` (not `Int`, `4-B2`), `[Row, Col]Bit` packed reads (`G8`), `[D](lo..=hi)` range-element parsing (`N7`), `a.row(r) = row` copy (`4-B3`), `1 << 30` constant folding & `gcd >= 1` (`G1`, `9-T18`), and multi-dim/struct array broadcast `var a: [D1][D2]T = init` (`N9`).
  - [ ] Strip `@brute` (including parameterless `@brute`) completely in `cpl build` so it never runs in `main()` (`W7`), and suppress complexity warnings inside `@brute`/`@gen`/`@checker`/`@interactor`/`assume` (`W15`, Blind Critic 2).
- [ ] **P0.6 Implement honest unified `cpl stress` (`proto/cp.py`):**
  - [ ] Default to in-file `@brute(main)` / `@checker` / `@interactor` when no ref binary is passed.
  - [ ] Escalate sizes `1..=max_size` (default `20`, `--max-size N`), always compile with `-DLOCAL`, and compare **line-by-line** by default (plus `--close <eps>` for float tolerance, `E9`).
  - [ ] Shrink every failure via choice-stream shrinking, replay `debug(...)` side-by-side on the shrunk input, warn when `@brute`-tested fn mutates top-level `var` (`W13`), and print honest coverage summary (max size, rejected count with >50% warning, diff mode, output diversity).
  - [ ] Fix default generators: non-empty `Str`, permutation for `Perm<V>`, structured maze/corridor grids alongside random cells, and distinguish `"generator broke assume"` from solution crashes (`Blind Critic 2`).

### Priority 0 — Max-N Judge-Killers (`proto/cpc/runtime.hpp`, `compiler.py`)

- [ ] **P0.7 Flat `@memo` array lowering for all algebraic `Index`es (`proto/cpc/compiler.py`):** Lower `@memo` over `Bool`, `Enum`, bounded ranges `lo..=hi`, tuples `(Row, Col)`, and `Subset<D>` (up to `2^26` slots) to a single top-level-sized flat `vector<Opt<R>>` (fixes `ioi04p4` 31.86× and `dpp` 3.92×).
- [ ] **P0.8 Niche-packed 8-byte `Opt<ll>` (`LLONG_MIN`) and `Opt<double>` (quiet NaN) (`proto/cpc/runtime.hpp`):** Fixes `ceoi18p1` 1.47× and `cco18p1` 1.52× RSS.
- [ ] **P0.9 Per-process salted open-addressed flat hash table (`proto/cpc/runtime.hpp`):** Replace `std::unordered_map`/`unordered_set` in `HashMap`/`HashSet`, add runtime salt to `CpHash`/`CpArrHash` (fixes 44.5× anti-hash exploit and `noi01p5` 198 MB MLE).
- [ ] **P0.10 Lean `RootedTree` (`proto/cpc/runtime.hpp`, `compiler.py`):** Lazy `sp` (LCA sparse table) and `jump`, direct unweighted constructor (no 4× adj copies), flat CSR `kids`, and zero-alloc `chains` iteration (fixes 6 tree 1.8×–2.7× RSS regressions).
- [ ] **P0.11 Constant-factor runtime fixes (`proto/cpc/runtime.hpp`, `compiler.py`):**
  - [ ] Lazy-sorted `vector` for `CpSteps` (fixes `cco20p1` 2.97×).
  - [ ] `vector<uint8_t>` everywhere instead of `vector<bool>` + fused/narrowed `CpPrefix<T>` (fixes `ioi10p3` 1.46×).
  - [ ] 4-byte `uint32_t` `Mod<P>` for `P <= 2^31 - 1`, parameterized `AddMax<T>`/`AssignSum<T>`, templated `SegTree` ops (no `std::function`), proven-non-negative `/` & `%` elision, hoisted `cp_win`, and full `@part` tree-shaking + `CP_GEN` stripping + `.cp` comment header (target 4–10 KB emitted C++).

### Priority 1 — Core Language v2 Spec Alignment (`proto/cpc/` & `proto/cpi/`)

- [ ] **P1.1 Keyword & syntax lock (`proto/cpc/lexer.py`, `parser.py`):**
  - [ ] Enforce the 40-keyword budget (delete `with`, `undo`, `by`, `requires`, `ensures`, `invariant`, `stress`, `protocol`, `limit`, `dyn`, `do`, `pool`, `decorator`, `static`, `yields`, `mut fn`).
  - [ ] Support `assert` (always on; lint on `mut`), `assume` (local-only; error on `mut`/I/O side effects), `defer`, `??=`, `debug(...)`, `if let` & `match` expressions, `for ... if`, `while let ... and`, `f::<T>()`, bidirectional non-closure arg unification, and line continuation after binary ops/`=`/open brackets.
  - [ ] Enforce uniform `?T` optionality on `i + k`, `xs.last`, `min(gen)`/`max(gen)` (remove constant-range non-optional `min` exception), and emit column + sub-expression + parenthesized-fix diagnostics (`a + (max(...) ?? 0)`).
  - [ ] Support open/boundary ranges (`..i`, `i..`, `l.r..r.l`) across all range structures and `b - a -> Int` on same-domain positions.
- [ ] **P1.2 Complete 5-shape algebraic `Index` (`proto/cpc/types.py`, `compiler.py`):** Support `Bool`, unit/payload `enum` (on demand), `?T`, structural range equality `lo..hi`/`lo..=hi`, and tuples/records `(Row, Col)` everywhere (`for`, `[Cell]T`, `Dsu`, `bfs`, `FlowGraph<Node>`, `@memo`).
- [ ] **P1.3 `0..len`-indexed ASCII `Str` & core missing prototype containers (`proto/cpc/compiler.py`, `runtime.hpp` — Blind Critic 3 Top Fix #3 & #4):**
  - [ ] Support `s[i]`, `s[l..r]`, `s[..k]`, `s[k..]`, `for c in s`, `s.len`, `s.is_empty`, `s.push(ch)`, `s + t`, `s += t`, `s * k`, `sep.join`, `s.starts_with(x, at:)`, `s.split`, `s.split_once`, `s.trim_start`, `s.parse_int()`, `Str(chars)`, `Char` helpers (`is_alpha`, `is_alnum`, `digit(base)`, `Char.digit`), while keeping opt-in `[domain S]Char = read()`.
  - [ ] Align `proto/cpc` to one range-structure spelling (`.sum(l..=r)`, `.fold(l..=r)`, `.apply(l..=r, tag)` across `Fenwick`, `SegTree`, `LazySegTree`) and ship `Fenwick<D, Int>()`, `LazySegTree<D, T>(...)`, `Deque<T>`, `MonoDeque<T>`, `Bitset<D>`, `Comb<P>(n)`, `p.adj4`/`p.adj8`, and `let xs = compress(all)` with `xs.D`, `xs[v]`, `xs.at(val)` (eliminating 55+ lines of Blind Critic 3 hand-rolled workarounds).
- [ ] **P1.4 `F64`, `U64`, `I128`, & unified `Mod<P>` (`proto/cpc/types.py`, `compiler.py`, `runtime.hpp`):**
  - [ ] Implement `F64` with quiet-NaN niche, local NaN/Inf traps, literal unification + explicit `F64(x)` for `Int` vars, refuse `Int(f)` (suggest `floor`/`ceil`/`round`/`trunc`), refuse bare `print(f)` (suggest `"{x:.6f}"`, never `-0.00`), lint `==`/`!=` toward `close(a, b, eps)`/`sign(x, eps)` (exempt `== 0.0`), and wire math intrinsics (`sqrt`, `sin`, `cos`, `tan`, `atan2`, `exp`, `ln`, `hypot`, `floor`, `ceil`, `round`, `trunc`, `PI`).
  - [ ] Implement `U64`, `I128`, `popcount`/`ctz`/`clz`/`bit_len`, and `<<=`/`>>=`.
  - [ ] Unify `Mod<P>` for both `const` and runtime-frozen `let` moduli (with `mul_mod` past `3.03e9`, one-way `Int` reduction, `sum`/`product` identity, and `.pow(e)`).
- [ ] **P1.5 I/O & `proto/cpi` parity (`proto/cpc/`, `proto/cpi/`):**
  - [ ] Support tuple `read()` with dependent bounds, record-field dependent bounds (`(l: V, r: l..=V.last)`), `read()` in assignments/expressions, symmetric `none:` on `read`/`print`/`read_output`, `read_line()` (skip trailing whitespace after token `read()`, empty line `""`, strip `\r`, `X1`), `read_until(sentinel)`, char-range `@tags('.', '*', '1'..='9')`, and `{val:.6f}`/`{n:04}` format checking.
  - [ ] Sync `proto/cpi/` to eliminate dialect drift (`enum`/`match`, `min=`/`max=` on `?Int`, open slices `s[..k]`/`s[k..]` fixing `X2` crash, `m[k] f= x`, `if let` expressions, `Mod + Int`, `any`/`all`, unified container verbs, `U64` + `Int` 64-bit read bounds `X9`) so `proto/cpc` and `proto/cpi` run the exact same v2 dialect.

### Priority 1 — Userland `stdlib/` v2 Alignment (`stdlib/*.cp`)

- [ ] **P1.6 `stdlib/algebra.cp`:** Trim contestant exports to bare-op support, `Monoid(id, op)`, `Action`, `Desc`/`desc(x)`, and add 1-method `Reversible` (retire `Semigroup`, `Group`, `Semiring`, `Sum`, `Prod`, `Max`, `Min`, `Xor`, `MaxOpt`, `MinOpt`, `MaxPlus`, `MinPlus`, `Order`, `Less`, `Greater`, `flip`, `both`, `opt` from contestant docs/prelude).
- [ ] **P1.7 `stdlib/deque.cp` & `stdlib/seq.cp`:** Ship imperative `Deque<T>` and `MonoDeque<T>(min | max)` (`push`, `drop_before`, `clear`, `best`); ship stateless `slide_until(r, (l, r) => ok)` yielding `(lo, hi)`, `sliding_min`/`sliding_max`, free-fn `bsearch_first`/`bsearch_last`/`bsearch_first_some`, uniform `.first_ge`/`.last_le`, `first`, `.find`, `min_by`/`max_by`, `dedup`, `rotate_left`, `next_permutation`, `(lo..=hi).clamp(D)`; delete `running_sum` and `pairs`.
- [ ] **P1.8 `stdlib/grid.cp`:** Implement `Dir4`, `Dir8` (`.cw`, `.ccw`, `.back`, `.from`), `p.step(d)`, `p.ray(d)`, **`p.adj4`**, **`p.adj8`**, `g.transposed`, `g.flipped` purely over `(Row, Col): Index`.
- [ ] **P1.9 `stdlib/ordered.cp`, `stdlib/heap.cp`, `stdlib/bitset.cp`, `stdlib/steps.cp`:**
  - [ ] Unify container verbs (`.len`, `.is_empty`, `.clear()`, `.add`/`.remove`/`in`/`| & - ^`, `.first`/`.last`/`.pop_first()`/`.pop_last()`, `.first_ge`/`.last_le`/`.rank`/`.nth`) across `Set`, `SortedBag`, `Map`, `MinHeap`, `MaxHeap`, `Bitset`, `Subset`.
  - [ ] Support signed `Bitset<D>.shifted(k)` and `Subset<D>.shifted(k)` (`W10`).
  - [ ] Back `Steps<K, V>` with lazy-sorted `List` (and cover interval sets via `Steps<K, Bool>`).
- [ ] **P1.10 `stdlib/segtree.cp` & `stdlib/lazy.cp`:** Accept bare ops (`+`, `min`, `max`, `^`, `gcd`) and `(id, op)` directly; add `.max_right(from, ok)` and `.min_left(to, ok)` to `LazySegTree` (`SF1`); ship the 6 stock actions (`AddSum`, `AddMin`, `AddMax`, `AssignSum`, `AssignMin`, `AssignMax`) and post-`lend` write-back `at`.
- [ ] **P1.11 `stdlib/tree.cp`:** Switch `Rooted.new` from BFS to DFS; put `root:`/`g` first for bidirectional inference; accept plain `V -> Iter<V>` unweighted adjacency; add `postorder`, `lift_while(v, pred)`, `virtual_tree(keys)`, and generic **`fold_path(a, b, ds)`** / **`apply_path(a, b, mut ds, tag)`** using `Reversible`; retire dual-tree `Hld` (and do not add `LazyHld`).
- [ ] **P1.12 `stdlib/graph.cp`, `stdlib/flow.cp`, `stdlib/dsu.cp`:**
  - [ ] Put `from:`/`g` first and accept plain `V -> Iter<V>` unweighted adjacency in `bfs`, `dijkstra`, `shortest` (auto 0-1 BFS for `0..=1` weights), `topo`, `scc`, `components`, `condense`.
  - [ ] Promote `two_edge_components`, `biconnected_components`, `articulation_points`, `two_color`, `topo_prefix`, `mst`, `TwoSat<V>` into `stdlib/graph.cp` (`N2`, `SF4`).
  - [ ] Promote `MinCostFlow<Node>` and `max_weight_matching<L, R>` into `stdlib/flow.cp` (`N3`).
  - [ ] Add `.mark()` / `.undo_to(snap)` rollback directly on `Dsu<V>` (`N5`, `SF2`).
- [ ] **P1.13 `stdlib/mod.cp`, `stdlib/strings.cp`, `stdlib/geometry.cp`:**
  - [ ] Retire `ModR` in `stdlib/mod.cp`; unify `Mod<P>` over `const` and runtime-frozen `let` `P`; keep `Comb<P>` (`.fact`, `.inv_fact`, `.choose`, `.arrange`).
  - [ ] Generalize `stdlib/strings.cp` (`z_function`, `prefix_function`, `palindromes`, `SuffixArray`, salted `RollingHash`) over `0..len`-indexed `Str` and any `Indexed<I>` without `List<T>` copies.
  - [ ] Generalize `Pt<T = Int>` in `stdlib/geometry.cp` (exact `turn` `Int`-only; add `turn_eps`, `unit(angle)`, `close(a, b, eps)`, `sign(x, eps)`).
- [ ] **P1.14 Update `stdlib/README.md`:** Align examples to v2 syntax and fix the outdated lines 263–271 claim about write-back `lends mut`.

### Priority 2 — Verification Tooling, Graders & `examples/local-lib/` (`proto/`, `examples/local-lib/`)

- [ ] **P2.1 Verification blocks (`proto/cpc/`, `proto/cp.py`):** Wire co-located `@checker` (`read_output()` + `assert`), `@interactor` (paired with expression-form `ask(..., limit: K)`), `@grader` (bodyless extern fn, exported entry fn, stateful grader struct), `read(gen:)` with `witness` + coupled tuple reads, whole-input `@gen`, and Hypothesis-style choice-stream shrinking over `rand(...)` draws.
- [ ] **P2.2 `examples/local-lib/` curation:** Keep advanced/niche structures (`Rope`, `RangeKth`, `LiChao<T>`, `PotentialDsu`, `Cover<Y>`, `Line`/`Hom`, `TwoStackFold`, `mo`, `Poly`/NTT, `XorBasis`, `Frac`, `Fixed<D>`, `Tri<D>`, `Trie<A>`, `SuffixAutomaton<A>`, `dnc_opt`, `knuth`, `sos_sub`/`sos_sup`, `std.gen` geometric helpers `rand_convex`/`rand_disjoint_rects`) cleanly tested under `examples/local-lib/` so core `std` never bloats.
- [ ] **P2.3 Rust graduation gate (do NOT start until P0–P1 pass on prototype):** Once the cheat-sheet CI gate (`P0.0`), the 25-problem bakeoff suite (`research/syntax-bakeoff-1.md`), Blind Critic 1–3, and Stage 6 max-N benchmarks pass on the v2 prototype, mirror the locked v2 spec into `crates/` (`cpl-syntax`, `cpl-check`, `cpl-ir`, `cpl-interp`, `cpl-codegen`) and update `editors/vscode/` (`languageData.ts`, `cp.tmLanguage.json`).
