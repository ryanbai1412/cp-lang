# Language Architecture Synthesis #3: Structural Syntax Leaps & Explicit Performance Knobs

*CP Lang 0.2.2 · Language Architect Synthesis #3 · Based on Fresh-Context Audit #2, Blind Contest Sim #2, `research/perf/corpus-p95-report.md`, `decisions.md` §19, and a fresh scan of all 988 active ports in `proto/ports/`.*

---

## Executive Summary

Synthesis #2 cleared the obvious surface friction (`if x min= y`, `a[l..=r]`, domain-keyed `Map`/`Set`, `V.tail`, `parent[V.tail] = read()`). Four deeper structural taxes remain across the 988 active ports in `proto/ports/` and the max-N benchmark harness (`research/perf/corpus-p95-report.md`, currently at 3.94x p95 against hand-written `g++ -O2` across 94 measured ports):

1. **Input-parsing boilerplate and the `cpl stress` cliff**: 449 standalone `let x: Int = read()` lines (including 129 consecutive runs totaling 308 lines) and 194 read-and-push loops (plus 65 indexed-assign read loops). Worse, 124 corpus problems could not be benchmarked at max-N solely because `cpl-stress` cannot extract an `InputSpec` from loop-based reads or `@!multitest` dividers.
2. **The postfix `!` tax**: 938 occurrences across 345 of 988 active ports (34.9%). Empirical classification shows that `!` is not one problem: 177 are `D.from_idx(...)!`, 50 are already eliminated by Synthesis #2's `if x min= y:`, 83 are local `!= none` / `== none` guards, 31 are guarded container `.first`/`.last`/`.pop()` calls, and ~200+ are honest algorithmic invariants that should keep `!`.
3. **Subscript domain wrappers**: 692 `D(...)` constructors inside `[...]` across 171 files plus 177 `D.from_idx(...)!` calls across 30 files (98.4% of all `from_idx` calls immediately unwrap). Most come from misusing nominal `domain`s for numeric counts/weights, missing optional-index `a.get(?I)` composition, and a spec mismatch on `from_idx`'s return type.
4. **Missing compile-time specialization knobs**: `Rooted` unconditionally allocates 10 arrays and runs two DFS passes even when a solution only needs `par`/`dep`, `SegTree` carries a runtime `Monoid` closure, and `@memo` accounts for the worst runtime outliers (`ioi09p4` at 6.21x, `dpn` at 6.03x, `ioi04p4` at 3.94x) because `crates/cpl-ir/src/lower/memo.rs` emits two separate vectors (`seen` + `vals`) and checks `ready`/`flat` inside every recursive call.

All four leaps are solved below with **zero new keywords** (the language stays at 34 keywords), **zero new decorators** (still 11 decorators, reusing `@law`), **zero weakening of strict nominal domains**, and **zero bespoke compiler special cases for stdlib types**. Every mechanism fits on `docs/one-page-cheat-sheet.md` and respects `decisions.md` §19.

---

## Leap 1: Declarative Input Shapes & Multi-Binding `read()`

### What the corpus actually does

Scanning all 988 active ports in `proto/ports/` exposes four concrete causes behind the 449 scalar `let x: Int = read()` lines and 194 read-and-push loops:

- **Consecutive same-type scalar reads** (129 blocks, 308 lines: 97 pairs, 18 triples, 10 quadruples, 4 quintuples). Writing `let (x0, y0, x1, y1, tint): (Int, Int, Int, Int, Int) = read()` repeats `Int` five times, so authors fall back to five separate lines.
- **Flat fixed-count loops** (73 read-push loops + 65 indexed-assign loops) and **adjacency double-push loops** (24 loops). `crates/cpl-syntax` already parses `let es: [n - 1](a: Int, b: Int) = read()`, yet ports still write loops because `[n - 1]T` was not lowered everywhere as a structural `0..(n - 1)` array (forcing throwaway `domain E = 1..n` declarations) and `Graph.undirected` required a named `domain E`.
- **Jagged nested reads** (28 loops): either length-prefixed rows (`k` followed by `k` items) or triangular/dependent rows (row `r` has `r + 1` items).
- **Tagged query streams** (13 loops): `for _ in 0..q: let kind = read(); if kind == 1: ...`.

Every single loop read breaks `cpl-stress`: `crates/cpl-stress/src/spec.rs` only builds an `InputSpec` from top-level typed `let` bindings (`Shape::Int`, `Shape::Array`, `Shape::Tuple`), and it does not yet wrap declarations below `@!multitest` in a generated test count `t`. That is why 124 corpus ports were excluded from `research/perf/corpus-p95-report.md`.

### The design: three orthogonal rules, zero mini-DSL

Resist adding a custom `read` statement mini-DSL (`read n, m: Int`, `read mut a: [V]Int`, `read edges: [n - 1]...`). Turning `read` into a declaration keyword adds a second variable-binding form alongside `let`/`var` and immediately forces duplicate spellings whenever an input is mutable (`var a: [V]Int = read()`), carries a sentinel (`read(none: "X")`), or attaches a custom generator (`read(gen: random_tree)`). Instead, three tiny general rules eliminate the loops and scalar stacks everywhere without adding a single keyword:

**Rule L1.1 — Homogeneous multi-binding in `let` / `var`.**
When a comma-separated list of identifiers precedes a single `: T`, the type distributes across a tuple pattern:

```cp
let n, m: 1..=2e5 = read()
// desugars identically to:
let (n, m): (1..=2e5, 1..=2e5) = read()
```

This is pure parser sugar — zero new IR node, zero `read`-specific semantics. It works with `read()`, with `var lo, hi: Int = (0, n)`, and with ranges or domains (`let u, v: V = read()`). Heterogeneous or cross-dependent tuples keep the existing tuple annotation `let (n, m): (1..=2e5, 1..=n) = read()`. To prevent ambiguity with scalar broadcast, `var a, b: Int = 0` is a type error (`expected (Int, Int), found Int; write var (a, b) = (0, 0)`).

**Rule L1.2 — Universal count-sized structural arrays `[expr]T` plus stdlib `Read` instances.**
Lock `[expr]T` everywhere in the spec, `cpl-check`, `cpl-ir`, and `cpl-stress` as exact sugar for the structural range array `[0..expr]T` (for any `Int` expression `expr` frozen before the declaration, including `n - 1` or `2 * m`). Pair it with three existing userland/language mechanisms that compose cleanly:

- **Inline structural records in `read()`**: `let edges: [n - 1](u: V, v: V, w: Int) = read()` reads `n - 1` triples with zero throwaway `domain E` and zero `struct` boilerplate. `Graph.undirected(edges)` and `Rooted.from_edges(edges, root: V.first)` accept any `Index` domain `E` (including structural `0..(n - 1)`), removing the 24 adjacency-list double-push loops.
- **Length-prefixed jagged rows in userland `stdlib/`**: `impl Read for List<T>` reads a non-negative count `k` followed by `k` elements of `T`, so `let plans: [m]List<City> = read()` is one line with no compiler special case.
- **Dependent-length rows via pure comprehension `read()`**: for triangular grids or step-dependent counts, `[Row] r => read::<[0..=r.to_idx]Int>()` expresses the exact dependent shape without `var` or `push`. Tagged streams use the already-specified `@tags(1, 2) enum Query: ...` with `let qs: [q]Query = read()`.

**Rule L1.3 — One `InputSpec` for `read()`, `@!multitest`, and `cpl stress`.**
Because every input declaration is an ordinary typed binding, `cpl-stress` (`crates/cpl-stress/src/spec.rs`) extracts `InputSpec` directly from the typed AST with zero separate schema language:

- Extend `Shape` with `Record(Vec<(Name, Shape)>)`, `TaggedEnum(Vec<(Tag, Shape)>)`, `List { max_len, elem }`, and `DependentArray { domain, elem_fn }` (for `[D] r => read::<...>()`).
- Record each binding's position relative to `@!multitest`. Bindings above `@!multitest` are generated once per test file (typically constants or precomputation bounds); bindings below `@!multitest` are generated `t` times with `t` shrunk first (toward `t = 1`), and dependent sizes like `n - 1` or `1..=n` re-evaluate per case inside the same shrinking loop. This directly unifies `@!multitest` with `cpl stress` and `cpl bench` and fixes Blind Contest Sim #2's multitest stress crash.

### Real corpus before/after

From `proto/ports/scout_batch_21/ccc14s4.cp` (lines 1–13):

```cp
// Before (13 lines, un-generatable by cpl stress)
let n: Int = read()
let t: Int = read()
struct Pane(x0: Int, y0: Int, x1: Int, y1: Int, tint: Int)
var panes: List<Pane> = []
for _ in 0..n:
    let x0: Int = read()
    let y0: Int = read()
    let x1: Int = read()
    let y1: Int = read()
    let tint: Int = read()
    panes.push(Pane(x0, y0, x1, y1, tint))

// After (3 lines, fully typed for cpl stress & cpl bench)
let n: 1..=1000 = read()
let t: 1..=1e9 = read()
let panes: [n](x0: Int, y0: Int, x1: Int, y1: Int, tint: Int) = read()
```

From `proto/ports/scout_batch_32/today/aac5p3.cp` (lines 1–13):

```cp
// Before (12 lines, manual 1-to-0 shift + double-push loop)
let (n, k): (Int, Int) = read()
var adj: List<List<(w: Int, v: Int)>> = [0..n] _ => []
for _ in 0..n - 1:
    let (u, v, w): (Int, Int, Int) = read()
    adj[u - 1].push((w: w, v: v - 1))
    adj[v - 1].push((w: w, v: u - 1))

// After (4 lines, strict vertex domain + CSR Graph, zero shift bugs)
let n, k: 1..=2e5 = read()
domain V = 1..=n
let edges: [n - 1](u: V, v: V, w: Int) = read()
let g = Graph.undirected(edges)
```

From `proto/ports/blind_critic_5/noi04p3.cp` (lines 5–35):

```cp
// Before (17 lines of scalar reads and two push loops)
let n: Int = read()
let m: Int = read()
let sh: Str = read()
let sv: Str = read()
...
var costh: List<Int> = []
for _ in 0..n:
    let c: Int = read()
    costh.push(c)
...
for _ in 0..q:
    let x1_in: Int = read()
    let y1_in: Int = read()
    let x2_in: Int = read()
    let y2_in: Int = read()

// After (6 lines)
let n, m: 1..=2000 = read()
let sh, sv: Str = read()
let costh: [n]Int = read()
let costv: [m]Int = read()
let q: 1..=2e5 = read()
let qs: [q](x1: 1..=n, y1: 1..=m, x2: 1..=n, y2: 1..=m) = read()
```

From `proto/ports/btoi17p3.cp` (lines 4–20), showing throwaway `Track` + manual `adj` loop (note `plans: [m]List<City> = read()` is already the target jagged-row idiom):

```cp
// Before (14 lines: 3 scalar reads + throwaway Track domain + 4-line adj loop)
let n: 2..=1e5 = read()
let m: 1..=5e4 = read()
let k: 1..=m = read()
domain City = 1..=n
domain Track = 1..n
...
let tracks: [Track](u: City, v: City) = read(gen: random_tree)
let plans: [m]List<City> = read()

var adj: [City]List<City> = []
for (u, v) in tracks:
    adj[u].push(v)
    adj[v].push(u)
let tree = Rooted(adj, City.first)

// After (6 lines: no throwaway Track domain, no manual adj loop)
let (n, m, k): (2..=1e5, 1..=5e4, 1..=m) = read()
domain City = 1..=n
let tracks: [n - 1](u: City, v: City) = read(gen: random_tree)
let plans: [m]List<City> = read()
let tree = Rooted.from_edges(tracks, root: City.first)
```

### `decisions.md` §19 check

- **Zero new keywords**: `let`, `var`, and `read` stay unchanged (34 keywords total).
- **No mini-DSL**: no format strings, no separate `input { ... }` block, and no new generator language — every rule extends existing types and `read()` (§16.10).
- **Taste Call #1** (see final section): whether to stop at `let n, m: T = read()` (recommended) or also accept `read n, m: T` as pure 1-line sugar for immutable top-level `let ... = read()`.

---

## Leap 2: Eliminating the Postfix `!` Tax Without Spooky Action-at-a-Distance

### What the 938 `!` sites actually are

Synthesis #2 deferred flow-typed `!` removal because naive proof-dependent typing causes spooky action-at-a-distance: changing a loop bound or distant range flips whether a line twenty lines later compiles. To solve this cleanly without guessing, we classified every single postfix `!` across all 988 active ports (938 occurrences in 345 files, 34.9%):

- **177 (18.9%)** are `D.from_idx(...)!` — not an optional-narrowing problem at all; solved cleanly in Leap 3.
- **50 (5.3%)** are `x == none or cand < x!` — already eliminated by Synthesis #2's `if x min= cand:`.
- **83 (8.8%)** are local `x!` guarded by `if x != none:` or `if x == none: return | continue | break`.
- **31 (3.3%)** are container `.last!`, `.first!`, `.pop()!`, `.pop_front()!` guarded by `while not xs.is_empty:`, `if not xs.is_empty:`, or `if xs.len > 0:` (another 88 container pops are multi-pop invariants like `heap.pop()! + heap.pop()!`, where `!` genuinely asserts `len >= 2`).
- **~125 (13.3%)** are domain steps (`l.next!`, `r.prev!`, `m.next!`) and non-empty aggregate folds (`min(... for m in l..r)!`).
- **~200+ (22%+)** are unguarded algorithmic invariants (`dp[v]!`, `dijkstra(...)[out]!`, `bsearch_first(...)!`, `state!.h`).

This decomposition is the key architectural insight. Trying to eliminate **arithmetic-order** `!`s (`l.next!`, `min(V)!`) through type narrowing is the exact source of spooky action-at-a-distance — and in `proto/ports/dpl.cp` (`if l == r: return a[l]; max(a[l] - lead(l.next!, r), ...)`) it is actually **unsound**, because `l != r` does not imply `l < r` unless a precondition guarantees `l <= r`! By contrast, **syntactic `none` guards** and **syntactic `is_empty` guards** depend solely on the condition sitting right above the line.

### The design: two purely syntactic, single-scope narrowing rules

**Rule N1 — Syntactic `none` narrowing on local bindings.**
Given a local binding `x: ?T` (a `let` or a `var` — never an arbitrary expression or array slot `a[i]`):

- `if x != none: <then>` narrows `x` to `T` inside `<then>` and on the RHS of `x != none and <rhs>`.
- `if x == none: <then> else: <else>` narrows `x` to `T` inside `<else>` and on the RHS of `x == none or <rhs>`.
- `if x == none: <diverge>` (where every path out of `<diverge>` exits via `return`, `break`, `continue`, or `assert false`) narrows `x` to `T` for the remaining statements of the **same lexical block**.
- `while x != none: <body>` narrows `x` to `T` inside `<body>`.

**Rule N2 — Single-step non-empty narrowing via `@law` (zero hardcoded container types).**
Instead of hardcoding `List`, `Deque`, `Set`, `MinHeap`, and `MaxHeap` inside `cpl-check`, reuse the existing `@law` decorator (`decisions.md` §19.1) on container methods in `stdlib/`:

```cp
// Inside stdlib/list.cp, deque.cp, heap.cp, set.cp — and any user local-lib container:
@law some_when(not self.is_empty)
pub @get fn last(self) -> ?T: ...

@law some_when(not self.is_empty)
pub fn pop(mut self) -> ?T: ...
```

A purely syntactic guard `not c.is_empty`, `c.len > 0`, `c.len >= 1`, `c.len != 0`, or an early exit `if c.is_empty: <diverge>` on a local binding `c` establishes the single lexical fact `non_empty(c)`:

- While `non_empty(c)` is live, any call to a method on `c` carrying `@law some_when(not self.is_empty)` narrows its return type from `?T` to `T`.
- Pure `&self` reads (`c.first`, `c.last`, `c.peek`) **preserve** `non_empty(c)`, so `if not active.is_empty: let lo = active.first; let hi = active.last` narrows both reads, and `while not st.is_empty and a[st.last] < a[r]:` narrows `st.last` directly in the condition.
- **Any `mut self` call on `c` — including `c.pop()` itself! — immediately kills `non_empty(c)`** after that call returns. Inside `while not pq.is_empty:`, the first `let (d, u) = pq.pop()` returns `T`, and a second `pq.pop()` on the next line returns `?T` again (catching the classic double-pop bug unless the guard explicitly checked `pq.len >= 2` or the author writes `!`).

### The four anti-spookiness invariants

How do we guarantee that an innocent edit can **never** cause spooky action-at-a-distance type errors? Four hard compiler invariants enforce strict locality:

1. **Syntactic-guard-only provenance**: The checker never consults distant `let` range types (`1..=n` vs `0..=n`), outer loop bounds, arithmetic inequalities (`i < n`), or earlier `assert` statements to compute a narrowed type. Only the immediately enclosing `if`/`elif`/`while`/`and`/`or` condition or an early-exit `if` in the same block can introduce a narrowing fact.
2. **Local bindings only — never places or fields (`a[i]`, `s.x`)**: Narrowing `a[i]` breaks if `i` changes or `a` is mutated through a helper; `a[i]` continues to use `if let v = a[i]:` or `if a[i] min= cand:`. That eliminates aliasing puzzles completely.
3. **Conservative, immediate kill on mutation and joins**: Assigning to `var x`, passing `mut x` to a function, calling any nested `fn` that captures `var x` mutably, or joining a control-flow branch that mutates `x`/`c` kills the narrowing fact on the very next statement — with a 1-line diagnostic pointing to the exact mutating line (`x may have been reassigned on line 42; re-check or bind with let`).
4. **Adding a guard never breaks existing `!` code**: Applying postfix `!` to a flow-narrowed value (one whose declared type is `?T`, currently narrowed to `T`) is a **warning/lint** ("unnecessary `!`: `x` is already non-none here"), not a compile error, while `!` on a genuinely non-optional `Int` remains a hard error. Refactoring or wrapping old code in `if x != none:` never triggers cascading errors on existing `x!`.

### What deliberately stays out of narrowing

- **Interval splits (`dpn.cp`, `ioi09p4.cp`)**: solved in userland by `(l..=r).splits` (Synthesis #2 U6), not by flow-proving `m < r => m.next: I`.
- **Unguarded invariants (`dpl.cp` `lead(l.next!, r)`, `noip04p2.cp` `heap.pop()! + heap.pop()!`, `dijkstra(...)[out]!`)**: keep `!`. When a program relies on a mathematical invariant the guard did not state, `!` is honest documentation and a loud runtime trap if the invariant fails.

Together, Rules N1 + N2 (114 sites), Synthesis #2's `if x min= y:` (50 sites), Leap 3's `from_idx` fix (177 sites), and userland `while let x = pq.pop():` / `(l..=r).splits` / `D.tail` cut postfix `!` across `proto/ports/` by **well over half**, leaving almost exclusively genuine invariants.

### Real corpus before/after

From `proto/ports/scout_batch_43/today/etsr23r2p2.cp` (lines 34–38):

```cp
// Before (Rule N1 + Synthesis #2 min= + Leap 3 structural Val all absent)
if c != none:
    let nv = Val(min(lhs * vi, CAP))
    let cand = 1 + c!
    let cur = dp1[l][r][nv]
    if cur == none or cand < cur!:
        dp1[l][r][nv] = cand

// After (zero `!`, zero `Val(...)` wrapper)
if c != none:
    dp1[l, r, min(lhs * vi, CAP)] min= 1 + c
```

From `proto/ports/scout_batch_24/noip22p4.cp` (lines 84–86, monotonic stack peek + pop):

```cp
// Before (3 bangs in 3 lines)
while not st_a.is_empty and a[st_a.last!] < a[r]:
    st_a.pop()
let la = if st_a.is_empty: I.first else: (st_a.last! + 1)!

// After (Rule N2 narrows st_a.last in both the while guard and the else branch;
// only the non-trivial domain step +1 keeps an explicit invariant or if-let)
while not st_a.is_empty and a[st_a.last] < a[r]:
    st_a.pop()
let la = if st_a.is_empty: I.first else: (st_a.last + 1)!
```

From `proto/ports/scout_batch_36/cco23p2.cp` (lines 42–44, multiple pure reads after one guard):

```cp
// Before
if not active.is_empty:
    let first = active.first!
    let last = active.last!

// After (pure &self getters preserve non_empty(active))
if not active.is_empty:
    let first = active.first
    let last = active.last
```

From `proto/ports/scout_batch_2/ioi08p2.cp` (line 44, illustrating zero spookiness):

```cp
// Before
if short != none: best = max(best, reach[b] + at + short!, reach[b] + around - at + long!)

// After: only `short` is narrowed; `long` is NOT silently narrowed because the guard did not check it.
if short != none and long != none:
    best = max(best, reach[b] + at + short, reach[b] + around - at + long)
```

### `decisions.md` §19 check

- **Zero new keywords, zero new decorators**: Rule N2 reuses the existing `@law` decorator (§19.1).
- **No SMT in typechecking**: preserves the `< 50 ms` frontend latency P0 and keeps typechecking purely lexical and deterministic.
- **No `min=` return-type change**: composes cleanly with Synthesis #2's `if x min= y:` without reviving §19.7's rejected expression-level `Bool` return.

---

## Leap 3: Eliminating Subscript Wrappers Without Weakening Strict Domains

### Why 869 subscript wrappers exist in `proto/ports/`

Scanning all 988 active ports finds **692 `D(...)` wrapper calls inside `[...]`** (across 171 files) plus **177 `D.from_idx(...)!` calls** (across 30 files):

- **463** wrap a plain integer loop variable (`best[Load(above)]`, `dp[I(i), I(j)]`, `used[I(first)]`).
- **105** wrap integer arithmetic (`nxt[I(i + 1), I(j + 1)]`, `best[Load(above + b.w)]`, `fact[F(down + right)]`).
- **89** wrap literals (`best[Spare(0)] = 0`, `dp[I(0), I(0)] = Z(1)`, `east[X(n), i]`).
- **35** wrap `.to_idx` or `.label` arithmetic (`pick[V(k.label)]`, `east[X(i.to_idx), j]`).
- **177** call `D.from_idx(expr)!` (vs only **3** genuine non-`!` calls across the entire 988-file corpus — 98.4% immediately unwrap!).

### Weighing the four approaches

1. **Relaxed `Int` indexing on nominal `[D]T` arrays — REJECT.**
   Allowing `dp[i + 1, j]` when `dp: [Row, Col]Int` and `i, j: Int` directly violates Ryan's locked decision (`decisions.md` §2.8, §2.11, `research/archive/RECOMMENDATIONS.md` §3). The moment bare `Int` indexes a nominal `[Row, Col]Int`, accidentally writing `dp[c.to_idx, r.to_idx]` or mixing a vertex `Int` with an edge `Int` compiles silently, destroying the #1 bug-prevention feature of CP Lang.

2. **Structural range arrays `[0..=n]T` and `type` aliases for counts, weights, and ASCII buckets — ENFORCE & COMPLETE.**
   Roughly 60% of the 692 `D(...)` wrappers are **not nominal entity IDs at all**: they are knapsack weights (`domain Load = 0..=20000` in `dpx.cp`), ball counts (`domain I = 0..=n` in `noi09p5.cp`), factorials (`domain F = 0..=h+w` in `dpy.cp`), or letters (`domain Letter = 0..26` in `dmpg16g2.cp`). Authors reached for `domain` only because earlier prototype backends failed to unify two `[0..=n]T` arrays (`dp = nxt` failed!) or rejected `[0..m]T` without a named `domain`.
   The unified rule is simple and strict:
   - **Rule of thumb taught on the cheat sheet**: use `domain` for *identities you must never swap* (`Row`, `Col`, `V`, `E`); use structural ranges `[0..=n]T` / `['A'..='Z']T` (optionally named with a structural `type Load = 0..=20000`) for *numeric quantities you do arithmetic on*.
   - **Structural equality on frozen bounds**: two structural ranges `a..=b` and `c..=d` denote the **same type** whenever their endpoints are syntactically equal frozen `let`/`const` expressions (`var dp, nxt: [0..=n, 0..=n]Z = 0; dp = nxt` always typechecks!). Plain `Int` (or `Char` for `'A'..='Z'`) indexes a structural range directly with local range-checked bounds (hoisted/eliminated when proven in bounds) and zero wrapper.

3. **Strict-domain checked-offset indexing via optional-accepting `a.get(...)` — ACCEPT.**
   When `dp: [Row, Col]Int` *is* over strict nominal domains, `r - 1` has type `?Row` and `c - 1` has type `?Col`. Today `dp[r - 1, c]` fails to typecheck, tempting contestants into `dp[Row(r.to_idx), c]` or `dp.raw[...]`.
   Extend `a.get(...)` (already in the language for safe indexing) with **one orthogonal rule**: for any array `a: [D1, ..., Dk]T`, `a.get(i1, ..., ik)` accepts `Dj` or `?Dj` in each axis (strictly typed to that axis's exact domain `Dj`!) and returns `?T` (`none` if any coordinate is `none`). Look at how this composes with tropical `max`/`min`/`??`:

   ```cp
   // Strict Row and Col domains: zero boundary `if`s, zero `Row(...)`, zero `!`
   dp[r, c] = grid[r, c] + (max(dp.get(r - 1, c), dp.get(r, c - 1)) ?? 0)

   // Swapping Row and Col is STILL a compile error!
   dp.get(c, r - 1)   // ERROR: expected Row or ?Row for axis 0, found Col
   ```

   For **guarded writes** to an offset (`nxt[r + 1, c] += cur`), `if let nr = r + 1: nxt[nr, c] += cur` already replaces the manual `if i < n:` check cleanly with zero wrapper and zero trap. For boundary endpoints (`X(0)` / `X(n)`), `X.first` / `X.last` already exist; for grid fences (`X = 0..=n` next to `B = 0..n`), `B.bounds` (`i.l` / `i.r`) already exists and removes every `.to_idx + 1` in `noi10p3.cp`; for sub-domains (`domain Kid = 2..=n`), `V.tail` eliminates the second domain entirely.

4. **Align `D.from_idx(i) -> D` (trapping) with `plan/19-index-and-lends.md` and `cpl-ir`, plus `D.try_from_idx(i) -> ?D` — ACCEPT.**
   Right now the cheat sheet says `D.from_idx(i) -> ?D`, while `plan/19-index-and-lends.md` (§3, `impl Index`) and `crates/cpl-ir` BOTH treat `D.from_idx(i)` as returning `D` directly! Because 98.4% of corpus calls (177 vs 3) immediately write `!`, making `D.from_idx(i: Int) -> D` trap if `i` is outside `0..D.K` (matching `D(label) -> D`, with zero runtime branch when the range checker proves `0 <= i < D.K`) and providing `D.try_from_idx(i: Int) -> ?D` (alias `D.try_at(i)`) for the 3 fallible calls erases 177 `!`s and resolves Blind Contest Sim #2's spec contradiction in one stroke.

5. **Keep `a.raw[i]` unchanged as the single explicit raw escape hatch.**
   Bit-twiddling DP, FFT butterflies, and radix passes keep `a.raw[i]` (bounds-checked in debug/default, `@unchecked` opt-out for hot loops) — visible in code review, never silent.

### Real corpus before/after

From `proto/ports/scout_batch_23/noi09p5.cp` (lines 12–28, count DP misused as nominal `domain I`):

```cp
// Before (10 `I(...)` wrappers in 12 lines)
domain I = 0..=n
var dp: [I, I]Z = Z(0)
dp[I(0), I(0)] = Z(1)
for s in 0..n + m:
    var nxt: [I, I]Z = Z(0)
    for i in max(0, s - m)..=min(n, s):
        for j in max(0, s - m)..=min(n, s):
            let cur = dp[I(i), I(j)]
            if cur == Z(0): continue
            if i < n and j < n and s1[i] == s1[j]: nxt[I(i + 1), I(j + 1)] += cur

// After (structural range `type Take = 0..=n`, zero wrappers, `dp = nxt` unifies)
type Take = 0..=n
var dp: [Take, Take]Z = 0
dp[0, 0] = 1
for s in 0..n + m:
    var nxt: [Take, Take]Z = 0
    for i in max(0, s - m)..=min(n, s):
        for j in max(0, s - m)..=min(n, s):
            let cur = dp[i, j]
            if cur == 0: continue
            if i < n and j < n and s1[i] == s1[j]: nxt[i + 1, j + 1] += cur
```

From `proto/ports/dpx.cp` (lines 11–18, knapsack weight table):

```cp
// Before
domain Load = 0..=20000
var best: [Load]Int = 0
for b in sorted(blocks, by: b => b.w + b.s):
    for above in (0..=b.s).rev:
        best[Load(above + b.w)] max= best[Load(above)] + b.v

// After (structural `type Load = 0..=20000`, zero wrappers)
type Load = 0..=20000
var best: [Load]Int = 0
for b in sorted(blocks, by: b => b.w + b.s):
    for above in (0..=b.s).rev:
        best[above + b.w] max= best[above] + b.v
```

From `proto/ports/scout_batch_27/noi10p3.cp` (lines 19–27, grid cells vs fences):

```cp
// Before (manual `/ n`, `% n`, `B(...)`, `X(...)`, `.to_idx + 1`)
let (i, j) = (B(u.to_idx / n), B(u.to_idx % n))
let (top, bottom, left, right) = (X(i.to_idx), X(i.to_idx + 1), X(j.to_idx), X(j.to_idx + 1))
to.push((to: if i.to_idx == 0: out else: block(i.to_idx - 1, j.to_idx), cost: east[top, j]))

// After (`east: [B.bounds, B]Int`, tuple cell `(i, j): (B, B)`, zero wrappers or `.to_idx`)
to.push((to: if let up = i - 1: (up, j) else: out, cost: east[i.l, j]))
```

From `proto/ports/scout_batch_39/dmpg16g2.cp` (lines 9, 47–53, ASCII letter bucket):

```cp
// Before
enum Letter = A | B | C | D | E
var last: [Col, Letter]?Row = none
...
last[c, Letter.from_idx(g[bot, c] - 'A')!] = bot

// After (structural Char range `'A'..='E'`, zero `from_idx!` or `-'A'`)
type Letter = 'A'..='E'
var last: [Col, Letter]?Row = none
...
last[c, g[bot, c]] = bot
```

From `proto/ports/blind_critic_4/ioi15p6.cp` (lines 9–16):

```cp
// Before (40 `T.from_idx(...)!` / `V.from_idx(...)!` in one file)
domain T = 0..m
var adj: [T]List<(Int, Int)> = []
for _ in 0..m - 1:
    let u: Int = read()
    let v: Int = read()
    let w: Int = read()
    adj[T.from_idx(u)!].push((v, w))
    adj[T.from_idx(v)!].push((u, w))

// After (read directly as `T`, plus `T.from_idx(i) -> T` where genuine 0-based conversion is needed)
domain T = 0..m
let edges: [m - 1](u: T, v: T, w: Int) = read()
let g = Graph.undirected(edges)
```

### `decisions.md` §19 check

- **Strict domains preserved 100%**: no implicit `Int -> D` conversion, no relaxed `Int` indexing on `[D]T`, and no `Sub<V> -> V` coercion.
- **No new syntax**: structural range types, `type` aliases, `B.bounds`, `a.get(...)`, and `D.from_idx` all exist in 0.2.2; we are simply finishing structural range unification, extending `a.get(...)` to `?I`, and aligning `D.from_idx`'s signature with `plan/19-index-and-lends.md` and `cpl-ir`.

---

## Leap 4: Explicit Compile-Time Performance Knobs (`if const`, `const` Config Structs, and `@memo`)

### Why this matters now

`research/perf/corpus-p95-report.md` measured 94 corpus ports at max-N against original `g++ -O2`: p95 runtime is **3.94x** (target: under 1.1x across 200+ ports, worst-case cap 1.5–2x). Inspecting the worst outliers and `stdlib/` reveals two distinct performance leaks:

1. **All-or-nothing userland data structures**: `stdlib/tree.cp`'s `Rooted<V, Pre>` unconditionally allocates 10 arrays (`par`, `dep`, `far`, `sz`, `kid`, `heavy`, `top`, `tin`, `at`) and runs two DFS passes even when a solution only reads `par` and `dep`. `stdlib/segtree.cp` stores `m: Monoid<T>` and `source: ?(D -> T)` as runtime fields on every segment tree, blocking GCC inlining of `+`/`max`. `stdlib/graph.cp` always stores `(to: V, e: E)` arcs even for unweighted graphs where `e` is unused.
2. **`@memo` lowering overhead**: `ioi09p4.cp` (6.21x, 56 MB RSS), `dpn.cp` (6.03x), and `ioi04p4.cp` (3.94x) are the three slowest measured ports. Reading `crates/cpl-ir/src/lower/memo.rs` (lines 262–385) exposes why:
   - Every recursive call checks `if (!__memo_ready)` and `if (__memo_flat && ...)` on entry.
   - Dense memo tables allocate **two separate vectors** (`seen: vector<bool>` and `vals: vector<T>`), doubling cache misses and memory footprint instead of using one flat niche-packed `?T` array (`LLONG_MIN` for `Int`, `NaN` for `Float`, `0xFF` for `Bool`).
   - Top-down recursion prevents GCC from unrolling and vectorizing tight inner DP loops.

### Piece 4A: Minimal `const` config structs + `if const` specialization

`plan/13-abstraction.md` already specifies `if const` for compile-time facts and `const N: Int` generic parameters. Only **three small, orthogonal rules** are needed so `stdlib/` gets full zero-cost specialization with zero compiler magic:

**Rule P1 — Compile-time POD structs as `const` generic and function parameters.**
Any plain `struct` whose fields are compile-time scalar types (`Bool`, `Int`, `Char`, fieldless `enum`, or another `const` struct) can serve as a `const` parameter, with field defaults:

```cp
pub struct TreeCfg:
    hld: Bool = true
    euler: Bool = true
    weighted: Bool = false

pub const TREE_BASIC = TreeCfg(hld: false, euler: false)
pub const TREE_EULER = TreeCfg(hld: false, euler: true)
pub const TREE_HLD   = TreeCfg(hld: true, euler: true)
```

Allowing `const cfg: TreeCfg = TREE_HLD` as a function/constructor parameter that binds a `const` type parameter on the returned struct eliminates positional generic noise (`Rooted<_, _, TREE_BASIC>`):

```cp
// Default (zero ceremony): full Rooted with HLD + Euler tour
let tree = Rooted(adj, City.first)

// Tight memory/time limit (n = 5e5): opt out of HLD & Euler with one named const arg
let tree = Rooted(adj, City.first, cfg: TREE_BASIC)
```

**Rule P2 — `if const` in both statements/expressions AND struct field types, with C++17 `if constexpr` typing.**
Allow a struct field's type to be `if const <cond>: T else: ()`:

```cp
pub struct Rooted<V, Pre = V, const C: TreeCfg = TREE_HLD>:
    root: V
    par: [V]?V
    dep: [V]Int
    far:   if const C.weighted: [V]Int else: ()
    sz:    if const C.hld or C.euler: [V]Int else: ()
    heavy: if const C.hld: [V]?V else: ()
    top:   if const C.hld: [V]V else: ()
    tin:   if const C.euler or C.hld: [V]Pre else: ()
    at:    if const C.euler or C.hld: [Pre]V else: ()
```

Two critical lowering rules make this genuinely zero-cost and sound:

- **Zero-sized `()` field erasure**: During monomorphization / C++17 codegen, every field of type `()` is omitted from the emitted C++ `struct` and constructor completely (`sizeof` drops by 6 vectors on `TREE_BASIC`, and zero dead initialization code is emitted).
- **C++17 `if constexpr` branch checking (amending `plan/13-abstraction.md`)**: `plan/13-abstraction.md` originally wrote that both branches of `if const` are type-checked. When a struct field has conditional type `if const C.hld: [V]?V else: ()`, type-checking `self.heavy[u]` when `C.hld == false` would fail on `()[u]`! Therefore `if const` must **parse both branches**, evaluate the condition once `const` parameters are instantiated, **type-check only the taken branch**, and erase the untaken branch before C++17 emission — matching `cpl-ir`'s existing monomorphizing pipeline and C++17 `if constexpr` directly.

**Rule P3 — `const assert` for actionable compile-time errors.**
Calling `tree.lca(u, v)` on `Rooted<V, Pre, TREE_BASIC>` hits `const assert C.hld: "Rooted.lca requires TreeCfg(hld: true)"` during monomorphization, producing a clear compile-time error instead of a cryptic `()[u]` failure.

Notice the generality: the exact same three rules give `stdlib/segtree.cp` zero-overhead monomorphized ops (`SegTree<D, T, const OP>` erasing `source` unless built via `SegTree.of`) and `stdlib/graph.cp` unweighted CSR graphs (`GraphCfg(store_edge_id: false)` cutting arc memory in half) — all in pure `stdlib/` with zero compiler knowledge of trees, graphs, or segment trees.

### Real `stdlib/tree.cp` before/after

From `stdlib/tree.cp` (lines 6–60):

```cp
// Before: always allocates 10 arrays and always runs both HLD DFS passes
pub struct Rooted<V, Pre>:
    root: V
    par: [V]?V
    dep: [V]Int
    far: [V]Int
    sz: [V]Int
    kid: [V]List<V>
    heavy: [V]?V
    top: [V]V
    tin: [V]Pre
    at: [Pre]V

// After: same default call `Rooted(adj, root)`, zero wasted memory when `cfg: TREE_BASIC`
pub struct Rooted<V, Pre = V, const C: TreeCfg = TREE_HLD>:
    root: V
    par: [V]?V
    dep: [V]Int
    far:   if const C.weighted: [V]Int else: ()
    sz:    if const C.hld or C.euler: [V]Int else: ()
    heavy: if const C.hld: [V]?V else: ()
    top:   if const C.hld: [V]V else: ()
    tin:   if const C.euler or C.hld: [V]Pre else: ()
    at:    if const C.euler or C.hld: [Pre]V else: ()

    pub fn lca(self, a: V, b: V) -> V:
        const assert C.hld: "lca requires TreeCfg(hld: true)"
        // existing HLD climb compiles only when C.hld is true
        ...
```

### Piece 4B: `@memo` — Fix default lowering first, and reject magical `@memo(iterative)`

`decisions.md` §19.3 explicitly deleted `@memo(flat)`, `@memo(hash)`, `@memo(capacity: N)`, and `@memo(reset: ...)` so that CP Lang's 11 decorators have **no parameter grammar of their own**. Evaluating `@memo(iterative)` honestly from a compiler-engineering standpoint reveals two decisive facts:

1. **Fixing `crates/cpl-ir/src/lower/memo.rs` eliminates most of the 4–6x slowdown automatically, with zero syntax change.**
   Three mechanical improvements to `memo.rs` apply to every bare `@memo` function:
   - **Single niche-packed `?T` table**: replace the parallel `seen: vector<bool>` + `vals: vector<T>` arrays with one flat vector storing `T`'s niche sentinel (`INT64_MIN` for `Int`, `NaN` for `Float`, `0xFF` for `Bool`, falling back to `seen` only for types with no niche). Cuts cache misses and memory in half.
   - **Entry wrapper + pure recursive inner function**: lift `if (!__memo_ready)` allocation and `flat` capacity check out of the recursive body into a 3-line entry function `f(args)` that initializes once and calls `f__flat(args)` (with zero `ready`/`flat` branches inside the recursion!).
   - **Static dense fast-path**: when all parameter domains are bounded and small, skip emitting the hash fallback entirely.
   On `dpn.cp`, `ioi04p4.cp`, and `ioi09p4.cp`, this removes every branch and half the cache traffic from the inner loop without adding a single character of syntax.

2. **`@memo(iterative)` as automatic recursion-to-loop inversion is fragile compiler magic — do NOT add it.**
   To compile an arbitrary recursive `@memo` function into bottom-up loops, the compiler must discover a valid topological evaluation order:
   - In `dpn.cp` (`cost(l, r)`), `l` goes *down* while `r` goes *up* (order by interval length `r - l`).
   - In `ioi09p4.cp` (`pay(r1, r2, c1, c2)`), order by `(r2 - r1) + (c2 - c1)`.
   - In `dpp.cp` (tree DP), order by reverse BFS / post-order DFS.
   - In TSP bitmask DP, order by `popcount(mask)` or numeric mask order.
   - Under tight memory limits (`ioi09p4` / knapsack), the contestant also needs a **rolling buffer** (dropping a dimension), which no single keyword can guess reliably.

   If the compiler tries to infer iteration order automatically, it silently fails or rejects half of real DPs. If the programmer specifies order and rolling dimensions inside the decorator (`@memo(iterative, order: len_asc, roll: 2)`), we have invented a complex decorator DSL that directly violates `decisions.md` §19.3 and is *harder* to remember under contest pressure than writing a 4-line `for` loop!

   Meanwhile, with Leap 3's wrapper-free structural arrays, the **honest, zero-magic iterative escape hatch** in ordinary CP Lang is already 5 lines and compiles to 1.00x vectorizable C++17:

```cp
// proto/ports/dpn.cp — Before (bare @memo: concise, 1.2–1.5x after memo.rs fix)
@memo
fn cost(l: I, r: I) -> Int:
    if l == r: return 0
    min(cost(l, m) + cost(m.next!, r) for m in l..r)! + pre.sum(l..=r)

// Explicit iterative escape hatch when squeezing 1.00x speed or rolling memory
// (zero decorator DSL, zero `I(...)` wrappers via Leap 3 structural range):
var dp: [0..n, 0..n]Int = 0
for len in 2..=n:
    for l in 0..=n - len:
        let r = l + len - 1
        dp[l, r] = min(dp[l, m] + dp[m + 1, r] for m in l..r)! + pre[r + 1] - pre[l]
```

And if a contestant wants a compile-time guarantee on `@memo`'s storage backend (flat vs hash) rather than loop inversion, that is a tiny, well-defined **Taste Call #2** for Ryan below.

### `decisions.md` §19 check

- **Zero new keywords**: reuses `if`, `const`, `struct`, `assert`.
- **Zero compiler magic**: `Rooted`, `SegTree`, and `Graph` stay 100% userland `stdlib/`.
- **§19.3 preserved by default**: bare `@memo` stays parameterless unless Ryan explicitly votes for the narrow `@memo(flat)`/`@memo(hash)` exception in Taste Call #2.

---

## Consolidated `decisions.md` §19 Compliance Audit

Every proposed change was checked line-by-line against `decisions.md` §19 ("What 0.2.2 cuts so the language stays small"):

- **Keyword budget**: **34 -> 34** (zero added; rejected a new `read` declaration keyword by default).
- **Decorator budget**: **11 -> 11** (zero added; Rule N2 reuses existing `@law`; `@memo(iterative)` decorator DSL rejected per §19.3).
- **No relaxed domain conversions**: strict nominal domains (`§2.8`, `§2.11`) and `a.raw[i]` escape hatch remain intact; structural ranges `[0..=n]T` stay distinct from nominal `domain`s.
- **No SMT in typechecking**: Rules N1 and N2 are strictly lexical/syntactic, single-scope, and deterministic (protecting the `< 50 ms` front-end check P0 and preventing spooky action-at-a-distance).
- **No `min=` expression-level `Bool` revival**: Leap 2 leaves Synthesis #2's `if x min= y:` intact (§19.7).
- **One deliberate amendment to `plan/13-abstraction.md`**: `if const` switches from "type-check both branches" to C++17 `if constexpr` semantics (parse both, type-check only the taken branch after monomorphization) so conditional `()` struct fields work cleanly.
- **One spec bug fixed**: `D.from_idx(i) -> D` (trapping) with `D.try_from_idx(i) -> ?D` aligns `docs/one-page-cheat-sheet.md` with `plan/19-index-and-lends.md` and `crates/cpl-ir`.

---

## Ranked Prototype Roadmap

Ordered by `(corpus friction removed + p95 impact) / implementation risk`, testing each step in `proto/cpc` and the corpus before graduating to `crates/`:

1. **Fix `memo.rs` default lowering (niche-packed single table + entry wrapper + flat fast-path)** — *Zero syntax change, highest immediate p95 win.* Attacks the top three runtime outliers (`ioi09p4` 6.21x, `dpn` 6.03x, `ioi04p4` 3.94x) directly in `crates/cpl-ir/src/lower/memo.rs`.
2. **Finish structural range unification (`[expr]T` == `[0..expr]T`, frozen-bound equality, `type` aliases) + align `D.from_idx(i) -> D` / `D.try_from_idx(i) -> ?D`** — *Largest immediate corpus-friction drop.* Eliminates ~400+ fake-domain `D(...)` wrappers and 177 `from_idx!` sites while fixing the `cpl-ir` vs cheat-sheet mismatch. Touchpoints: `proto/cpc`, `crates/cpl-check`, `crates/cpl-ir`, `docs/one-page-cheat-sheet.md`.
3. **Homogeneous multi-binding `let a, b: T = expr` + unified `@!multitest` `InputSpec` + stdlib `impl Read for List<T>`** — *Unblocks 124 max-N benchmarks and kills ~300+ scalar read lines and ~140+ read-push loops.* Touchpoints: `crates/cpl-syntax`, `crates/cpl-stress/src/spec.rs`, `stdlib/list.cp`, `stdlib/graph.cp`.
4. **Optional-accepting strict `a.get(?I)` / `a.get(?R, ?C)`** — *Tiny, surgical, high-delight win for 1D/2D grid and sequence DP.* Keeps strict domain safety while removing boundary `if`s, `R(...)` wrappers, and `!` in one stroke. Touchpoints: `proto/cpc`, `crates/cpl-check`, `crates/cpl-ir`.
5. **Syntactic local optional narrowing (Rule N1 `!= none` + Rule N2 `@law some_when(not self.is_empty)`)** — *Bake off on 30 real `proto/ports/` files in `proto/cpc` first* to verify the four anti-spookiness invariants and diagnostic clarity on mutation-kill edge cases before implementing in `crates/cpl-check`.
6. **`const` POD config structs + `if constexpr`-style `if const` (statements + `()` field erasure) + `const assert`** — *Enables zero-cost `Rooted<V, Pre, const C: TreeCfg>`, monomorphized `SegTree` ops, and lean `Graph` CSR in pure `stdlib/`.* Implement in `crates/cpl-syntax`, `crates/cpl-check`, `crates/cpl-ir`, then refactor `stdlib/tree.cp`, `stdlib/segtree.cp`, and `stdlib/graph.cp`.

---

## Two Genuine Taste Calls for Ryan's Vote

Everything above has a clear engineering default except **two genuine taste calls** where either side is coherent:

### Taste Call #1 — `let n, m: T = read()` uniformity vs optional `read n, m: T` header sugar

Compare the top of a typical problem under both options:

```cp
// Option A (Recommended — 0 new keywords, uniform with `var`, `gen:`, `none:`):
let n, m: 1..=2e5 = read()
domain V = 1..=n
let edges: [n - 1](u: V, v: V, w: Int) = read()
var a: [V]Int = read()

// Option B (Add contextual statement sugar `read <pat>: <Type>` for immutable top-level `let ... = read()`):
read n, m: 1..=2e5
domain V = 1..=n
read edges: [n - 1](u: V, v: V, w: Int)
var a: [V]Int = read()   // still uses `var ... = read()` when mutable or passing `gen:`/`none:`
```

**Default lean**: **Option A** — saves a contextual keyword and keeps one universal binding rule (`let`/`var ... = read(...)`), while homogeneous multi-binding and `[n - 1](...)` already capture 90% of the conciseness win. Choose **Option B** if you value an unmistakable, ultra-terse "input schema header" at the top of every contest file enough to accept a second binding form for immutable `read()`.

### Taste Call #2 — Keep bare `@memo` (§19.3 pure) vs narrow `@memo(flat)` / `@memo(hash)` storage hint

Assuming we ship the default `memo.rs` speedup (niche-packed single table + entry wrapper) AND reject magical `@memo(iterative)` recursion-to-loop inversion (using ordinary wrapper-free `for` loops as the 1.00x iterative / rolling-buffer escape hatch):

- **Option A (Recommended — preserve `decisions.md` §19.3)**: Keep bare `@memo` with zero decorator parameter grammar; runtime chooses flat vs hash automatically inside the 3-line entry wrapper, and explicit `for` loops serve as the iterative/rolling escape hatch.
- **Option B (Narrow §19.3 exception)**: Reopen §19.3 *solely* for a 1-word storage hint `@memo(flat)` / `@memo(hash)` so a contestant can force a flat table above `DENSE_LIMIT` or force hashing on a sparse medium-sized domain (still rejecting `@memo(iterative)`).

**Default lean**: **Option A** until post-`memo.rs`-fix max-N benchmarks across 200+ ports show real cases where the automatic `DENSE_LIMIT` heuristic picks the wrong backend.
