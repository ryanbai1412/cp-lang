# Friction Backlog — Scout Batch 51: Hardest Remaining APIO, EGOI, CCO & CCC Classics against CP Lang v2.1

This report evaluates the locked **CP Lang v2.1** design (`docs/one-page-cheat-sheet.md` at `e7e6794`, `research/language-architecture-v2.1.md`) on 10 unported, full-score accepted Olympiad submissions from `corpus/submissions/` (`apio23p2`, `egoi22p3`, `egoi22p4`, `egoi22p7`, `cco18p2`, `cco03p4`, `ccc23s2`, `ccc23s1`, `ccc96s3`, `ccc20s1`). Every problem has both an idiomatic v2.1 port under `proto/ports/scout_batch_51/` and a runnable `today/` implementation verified against the original accepted C++17 submission by `proto/ports/scout_batch_51/check.py` (326 seeded random and edge-case tests, 10/10 passing).

## Candidate Selection Note

After Batches 1 through 48, `apio23p2`, `egoi22p3`, `egoi22p4`, and `egoi22p7` were the four largest remaining full-score accepted Olympiad submissions in the `apio`/`egoi`/`btoi`/`ceoi`/`joi`/`cco`/`ccc` prefixes (every remaining unported `btoi*`, `ceoi*`, `apio23p3`, `egoi22p6`, and `cco20p*`–`cco24p*` file in `corpus/submissions/` is a partial-score submission). We paired those four flagship Olympiad problems (totaling 1,021 raw C++ LOC) with the six most varied remaining full-score `cco*`/`ccc*` problems so the batch exercises every major v2.1 addition: Tropical Optional arithmetic, `Marks<D>`, `Range.l`/`Range.r`, `Bool` and `Subset<D>` indices, RHS-before-LHS + `Copy`-by-value arena evaluation, block-scoped `defer`, `@grader`, `Action` lazy segment trees, `Rooted<V>`, ordered `Map` predecessor queries, and `F64` formatting.

## Dual LOC Measurement

Counting non-blank, non-comment code lines across all 10 problems, raw C++ drops from **1,347 LOC to 355 LOC (-73.6%)**, while runnable `today/` ports total **599 LOC (-55.5%)**. After stripping C++ template headers, debug macros, `main()` fast-IO shells, local `Grader` test harnesses, and standard reusable data structures (`ST1`/`ST2` in `apio23p2`, `TDS` binary-lifting LCA in `egoi22p4`, and `DSU` in `egoi22p7` — while keeping problem-specific monoids, ODT, and treaps counted in both languages), library-stripped C++ drops from **695 LOC to 355 LOC (-48.9%)**.

Per-problem breakdown (raw C++ / library-stripped C++ -> v2.1 spec / `today/`):

- **apio23p2** (`5593302.cpp`): 274 / 148 -> 70 / 135 (-74.5% raw, -52.7% library-stripped).
- **egoi22p3** (`5199301.cpp`): 214 / 99 -> 54 / 69 (-74.8% raw, -45.5% library-stripped).
- **egoi22p4** (`5199667.cpp`): 321 / 238 -> 107 / 149 (-66.7% raw, -55.0% library-stripped).
- **egoi22p7** (`5203797.cpp`): 212 / 81 -> 52 / 99 (-75.5% raw, -35.8% library-stripped).
- **cco18p2** (`4801864.cpp`): 58 / 28 -> 12 / 18 (-79.3% raw, -57.1% library-stripped).
- **cco03p4** (`5024536.cpp`): 53 / 22 -> 13 / 23 (-75.5% raw, -40.9% library-stripped).
- **ccc23s2** (`5350805.cpp`): 58 / 27 -> 12 / 18 (-79.3% raw, -55.6% library-stripped).
- **ccc23s1** (`5350804.cpp`): 55 / 24 -> 11 / 15 (-80.0% raw, -54.2% library-stripped).
- **ccc96s3** (`5515585.cpp`): 51 / 13 -> 19 / 22 (-62.7% raw, +46.2% library-stripped — explained below).
- **ccc20s1** (`4323572.cpp`): 51 / 15 -> 5 / 51 (-90.2% raw, -66.7% library-stripped).

On the four major Olympiad problems alone, library-stripped code is cut literally in half: **566 LOC -> 283 LOC (-50.0%)**.

## Where CP Lang v2.1 Shines on Hard Olympiad Problems

### 1. Tropical Optional Arithmetic Eliminates Sentinel Plumbing Across Complex Monoids

In `apio23p2.cp`, the original C++ needed explicit `INF = 1e9` sentinels in `NP`, `NMax`, and both segment trees, plus defensive overflow guards because `INF + y` can drift. Under v2.1's Tropical Optional Rule (`+` absorbs `none`, `min`/`max` skip `none`), the entire prefix-balance monoid and range-chmax `LazySegTree` collapse to four lines with zero magic constants:

```cp
struct NP:
    yl: ?Int = none
    yr: ?Int = none
    y: Int = 0
    x: Int = 0
    fn merge(self, o: NP) -> NP:
        NP(min(self.yl, self.y + o.yl), max(self.yr, self.y + o.yr), self.y + o.y, self.x + o.x)
```

When `self` is the identity `NP(none, none, 0, 0)`, `min(none, 0 + o.yl)` yields `o.yl` automatically. When `o` is identity, `self.y + none` yields `none`, and `min(self.yl, none)` yields `self.yl`. The same rule turns the range-chmax / range-max `LazySegTree` in `solve_overlap` into a one-liner `Action`:

```cp
let chmax_max = Action(
    id: none,
    compose: (a: ?Int, b: ?Int) => max(a, b),
    apply: (v: ?Int, tag: ?Int, _) => max(v, tag),
    over: Monoid(none, max),
)
```

And in `ccc23s2.cp`, `ans[2 * j + 1 + off] min= cur` removes the `INF` array initializer completely.

### 2. Copy-by-Value + RHS-Before-LHS Unblocks Arena Treaps (Proven Live Against `proto/cpc`)

In `egoi22p4.cp`, each city owns a lazy-add treap over a flat `List<Node>` arena. In v2.1, `merge` reads naturally:

```cp
if nodes[u].pri < nodes[v].pri:
    push(u)
    nodes[u].rc = merge(nodes[u].rc, v)
    u
```

When we compiled the runnable `today/egoi22p4.cp` with the pre-v2.1 `proto/cpc` compiler, the borrow checker rejected the exact analogous call:

```
error: line 65: `t_lc[u]` can't be passed to `split`: `split` also changes `t_lc` directly, so it would reach the same storage under two names
```

Even though `t_lc[u]` is an `Int` (`Copy` scalar!) passed by value, pre-v2.1 required introducing manual `let lc = t_lc[u]` / `let rc = t_rc[u]` temporaries at every single recursive treap call (six extra lines of ceremony across `split`, `merge`, and `move_add`). That live failure directly validates v2.1 U4's Copy-by-value release and RHS-before-LHS indexed assignment.

Side bonus on `egoi22p4`: the original C++ submission (`5199667.cpp`) allocated a 23 MB `struct TDS` local variable on the stack inside `solve()`, causing an immediate SIGSEGV under Linux's default 8 MB stack during differential testing (DMOJ uses an unlimited stack). CP Lang's heap-backed `Rooted<City>` eliminates that entire class of silent contest/local-testing mismatch while cutting 55 lines of binary-lifting LCA down to a single line.

### 3. `Marks<D>` and `Range.l`/`Range.r` Clean Up D&C Segment Trees

In `egoi22p7.cp`, the original C++ tracked seen left/right DSU roots during each D&C merge with two `vector<int>`s and an O(n) linear scan `c_vis(vis, i)` because `memset` per merge would blow O(n^2). In v2.1, `Marks<Pin>` gives O(1) `.clear()` and O(1) `.add(root) -> Bool` membership in two lines:

```cp
seen_l.clear()
seen_r.clear()
ver[o] = ver[rc]
for i in left:
    if not seen_l.add(dsu.find(i)): continue
```

Combined with v2.1's `Range.l`/`Range.r` boundary shorthands, splitting a segment-tree range drops every `+ 1` / `- 1` index error:

```cp
let m = r.mid!
let (left, right) = (r.l..m.r, m.r..r.r)
```

### 4. `Bool` and `Subset<D>` Indices Plus Block-Scoped `defer`

- In `ccc23s1.cp`, `[Bool, Col]0..=1` lets `for r in Bool:` and `wet[not r, c]` replace `for (int r = 0; r < 2; r++)` and `wet[1 - r][c]` with zero magic `0..2` row domains.
- In `cco03p4.cp`, rethinking the original's O(n! * k) `next_permutation` scan into a `Subset<V>` bitmask DP (`for s in Subset<V>`, `v not in s`, `(need[v] & s) == need[v]`, `s.with(v)`) eliminates every bit-shift bug (`1 << v`, `(1 << n) - 1`) in 13 lines.
- In `ccc96s3.cp`, block-scoped `defer cur.pop()` inside `if ones > 0:` and `if rem > ones:` proves why v2.1 U6's block-scoped (not function-scoped!) `defer` matters: each branch pops its own pushed digit immediately when the recursive call returns, before entering the sibling `if rem > ones:` branch.

## Remaining Friction & Feature Collisions

### F1. Early-Exit Optional Unwrapping in Non-`?T` Functions (High Friction on Treaps & Trees)

CP Lang v2.1 supports `let u = o?` only when the enclosing function returns `?T`. In `egoi22p4.cp`, `split(o: ?Id, key: Pos) -> (?Id, ?Id)` returns a *tuple* of optionals, and `merge(a: ?Id, b: ?Id) -> ?Id` returns the *other* argument when one is `none`. Writing `let u = o?` is illegal in `split`, and `let u = a ?? return b` relies on `return` being an expression inside `??`, which the cheat sheet does not specify. We had to fall back to explicit `== none` checks followed by `!`:

```cp
fn split(o: ?Id, key: Pos) -> (?Id, ?Id):
    if o == none: return (none, none)
    let u = o!

fn merge(a: ?Id, b: ?Id) -> ?Id:
    if a == none: return b
    if b == none: return a
    let (u, v) = (a!, b!)
```

Allowing diverging `return`/`break`/`continue` on the RHS of `??` (so `let u = o ?? return (none, none)` and `let u = a ?? return b` work with zero new keywords, matching Kotlin's Elvis operator `?: return`) would eliminate both the double-check and the `!` bang on every treap, link-cut tree, and persistent segment tree.

### F2. `List.push_id` vs Refined Arena-Index Niche Types (Medium Friction)

In `egoi22p4.cp`, defining `type Id = 0..=200_005` lets `?Id` niche-pack into 4 bytes instead of 16 bytes, keeping each treap `Node` compact in L2 cache. However, `nodes.push_id(Node(...))` returns plain `Int`, requiring an explicit narrowing binding (`let id: Id = nodes.push_id(...)`) and leaving no static guarantee that `nodes.len <= 200_005`. Either documenting that `let id: Id = nodes.push_id(...)` is the canonical idiom (with a single checked conversion) or letting `Arena<T>` / `List.push_id` stay simple without a new type is worth making explicit in the spec.

### F3. Missing `prev_permutation` Caused Our Only Library-Stripped LOC Regression (`ccc96s3`)

The v2.1 prelude includes `next_permutation(mut a)` (cheat sheet section 10) but omits `prev_permutation(mut a)`. Because `ccc96s3` requires descending lexicographical order of bit strings, C++ solves it in 13 library-stripped lines with `do { ... } while (prev_permutation(...))`, whereas CP Lang needed 19 lines of recursive backtracking (+46.2% library-stripped LOC — the only regression in the batch). Adding `prev_permutation(mut a)` beside `next_permutation(mut a)` (zero new concepts, trivial stdlib mirror) would shrink `ccc96s3.cp` to 8 lines.

### F4. Reading `0`/`1` Grids into `Bool` vs `0..=1` (Low-Medium Friction)

In `ccc23s1.cp`, each input row contains space-separated `0` and `1` tokens. Using `[Bool, Col]` for the row index was great, but we had to declare the element type as `[Bool, Col]0..=1` instead of `[Bool, Col]Bool` because `read()` for `Bool` is unspecified (would it accept `0`/`1`, `true`/`false`, or `YES`/`NO`?). Consequently, every condition still wrote `wet[r, c] == 1` instead of `wet[r, c]`. Clarifying whether `read()` on `Bool` parses `0`/`1` (or leaving `0..=1` explicit as the rule) removes hesitation under contest pressure.

### F5. `Range.mid` on Non-Empty Ranges + Non-Inclusive `mid + 1` Stepping (Low-Medium Friction)

In `egoi22p7.cp`, `r.mid!` returns `?Pin`, so inside a `r.len > 1` branch you still unwrap with `!` even though `r.len == 1` was just handled above. Even more subtly, splitting `r` into left and right halves requires knowing whether `m` belongs to `left` (`r.l..m.r` vs `m.r..r.r`): that worked cleanly because v2.1 added `Range.l` and `Range.r`, avoiding `(m + 1)!`, which otherwise triggers a redundant optional unwrap on `Pin`. Documenting the canonical segment-tree split `(r.l..m.r, m.r..r.r)` on the cheat sheet would save contestants from writing `r.first!..=m` and `(m + 1)!..=r.last!`.

### F6. `@grader` Signature Mismatch Between 1-Based External API and 0-Based `.to_idx` / `.from_idx` (Low Friction)

In both `egoi22p3.cp` and `egoi22p7.cp`, the external `@grader` functions pass and expect 1-based `Int` vertex/pin IDs (`GetMove() -> Int`, `Connected(a, i, j) -> Int`, `DescribeDesign(List<(Int, Int)>)`). Even when `domain V = 1..=n` is itself 1-based (`egoi22p3.cp`), the cheat sheet only exposes 0-based `V.from_idx(k)` and `v.to_idx`, forcing manual `- 1` and `+ 1` offsets at every grader call site:

```cp
let u = V.from_idx(raw - 1)!
for v in pair_path[u][1..]: MakeMove(v.to_idx + 1)
```

Exposing label-space conversions (`V.from_label(raw)` / `v.label` or `Int(v)` returning the declared 1-based label) eliminates those off-by-one sites at `@grader` boundaries.

### F7. Prototype Drift Observed in `today/` Builds

While `proto/cpc` compiled all 10 runnable ports cleanly once adjusted, four gaps accounted for the 244-LOC gap between v2.1 (355 LOC) and `today/` (599 LOC):

- **No `F64` or `"{x:.5f}"` formatting:** turned 5-line `ccc20s1.cp` into 51 lines of integer cross-multiplication and manual decimal digit emission in `today/ccc20s1.cp` (10x bloat).
- **No `LazySegTree` / `Action` or `compress()`:** forced 45 lines of hand-written chmax segment tree and binary-search coordinate compression in `today/apio23p2.cp`.
- **No `Map.last_le` / `.last_lt` or Copy-by-value borrow release:** forced a max-`SegTree` ODT stand-in plus 6 manual `let lc = t_lc[u]` copies in `today/egoi22p4.cp`, and parallel arrays to dodge `Opt<Int>` inside structs.
- **`SegTree.query(l, r)` vs v2.1 `.fold(l..=r)`** and **`List<Char>` concatenation:** `Str + Str` is rejected (`+ needs Int operands`), though `print(List<Char>)` printing with zero spaces turned out to be a pleasant surprise for `ccc96s3` and `ccc20s1`.

## Summary Verdict

CP Lang v2.1 held up remarkably well on the hardest remaining Olympiad problems in the corpus: cutting library-stripped code by **50.0%** across `apio23p2`, `egoi22p3`, `egoi22p4`, and `egoi22p7` with zero new keywords needed. The top two highest-leverage follow-ups from this batch are tiny, zero-keyword polish items: (1) allow diverging `return`/`break`/`continue` on the RHS of `??` (`let u = a ?? return b`) for recursive pointer/arena data structures, and (2) add `prev_permutation(mut a)` alongside `next_permutation(mut a)`.
