# Syntax Bakeoff #2: CP Lang v2.1 Multi-Topic Ergonomics, Keystroke & Punctuation Shootout

This bakeoff stress-tests the five remaining design tensions in CP Lang v2.1 (`research/archive/architecture/language-architecture-v2.1.md`, `docs/one-page-cheat-sheet.md`) against their strongest competing minimal syntax or zero-keyword stdlib alternatives across **20 genuinely hard, multi-topic problems** already ported in `proto/ports/`.

Every line count, keystroke count, punctuation count, and side-by-side code comparison below is mechanically computed from the actual corpus ports at HEAD (`d6b3c4c`), cross-checked against corpus-wide feature incidence across all **611 canonical ideal ports** in `proto/ports/` (943 `.cp` files total including `today/` workarounds and blind-critic drafts). Per project policy, the bakeoff calls clear winners — including four places where CP Lang v2.1 **loses** outright to simpler syntax or a plain stdlib pattern.

---

## Executive Summary & Winners

Across the 20-problem benchmark suite, the canonical CP Lang v2.1 baseline totals **963 non-comment LOC** against **3,815 raw C++17 LOC** across the 20 original accepted submissions (**74.8% raw LOC reduction**; note that per-file `.strip` library-stripped annotations exist for only 1 of these 20 scout ports, so library-stripped C++ LOC reduction is not estimated here). Every bakeoff arm holds the algorithm and stdlib calls fixed so that deltas isolate pure language-design impact: **33,317 raw characters**, **26,882 non-indent typed keystrokes**, and **6,060 punctuation characters** (**22.54%** punctuation density: `64` `?`, `1` `::`, `650` `:`, `2,714` brackets, `118` `@!?` sigils).

- **Tension 1 (Tropical Optionals vs. strict `?T` + explicit `??` / `if let`) — Winner: CP Lang v2.1 (with mandatory `min=` failure-poisoning guardrail).** Removing Tropical Optionals costs **+4 LOC, +945 typed keystrokes (+9.8% on affected files), +221 punctuation chars, and +36 `?` characters** across 7/20 problems (`yac6p6` alone swells by +606 keystrokes and +26 `?`). Keep Tropical Optionals, but document and lint against the real 2 AM footgun exposed by `etsr23r1p3` (where `none` means "poisoned failure" rather than `+∞` identity, so `best min= hit` silently swallows `none` and causes WA) and note that `*`/`/` do not lift (`sdoi17r1p2`).
- **Tension 2 (`@memo(flat|hash)` & `@multitest` / `@multitest(count:)` vs. zero-annotation auto-dispatch & `for _ in 0..t:`) — Winner: Minimal-Decorator Hybrid (v2.1 LOSES on `@memo(flat|hash)` and `@multitest(count:)`, WINS on bare `@multitest` and nested `@memo`).** Dropping `@multitest(count:)` and `@memo(flat|hash)` saves **-1 LOC, -23 typed keystrokes, -6 punctuation chars, and -1 `@`** with zero loss of expressiveness: in `noi22p4`, `@multitest(count: tests)` + `fn solve():` is strictly worse than `for _ in 0..tests:`, and in `etsr23r2p4`, `@memo(hash)` repeats what auto-dispatch already infers from `ind: Int`. Meanwhile, bare file-top `@multitest` earns its keep by avoiding 4 spaces of whole-file indentation (668 raw chars across 5 files) and 5 `read::<Int>()` turbofish `::` tokens, and block-scoped nested `@memo` cuts `coci21c1p3` RAM 4x.
- **Tension 3 (`for mut` / `.swap` / `.take()` / `.absorb()` / mutable `.last` / RHS-before-LHS vs. plain indexing & minimal core) — Winner: Split Decision (v2.1 WINS on RHS-before-LHS, `.take()`, and `swap`; v2.1 LOSES on `List.absorb()` and is marginal on `for mut`).** Dropping RHS-before-LHS and `.take()` costs **+12 LOC, +176 typed keystrokes, and +36 punctuation chars** across 7/20 problems and reintroduces the C++ `std::vector` reallocation UAF pattern (`noi18p3`, `noi20p5`). However, `.absorb()` hits **0/611** ports and cannot even be called on our two real small-to-large problems (`cco22p5`, `noi22p1`) because of same-container mutable aliasing, per-element side effects, and an O(|other|) vs. O(min) complexity trap between `List.absorb` and `MinHeap.absorb`. Meanwhile, `noi20p5` exposes a spec gap: v2.1 must specify **index-before-place-borrow** (not just RHS-before-LHS) so `trie[step(o, BothGoRight)].ends = true` is safe.
- **Tension 4 (`Marks<D>` vs. `[D]Bool` + `.fill()` / touched-list clear) — Winner: `[D]Bool` + Touched-List / `.fill()` primary; `Marks<D>` LOSES as a core abstraction (keep only as optional `std.marks` helper after shrinking stamps to `I32`).** Dropping `Marks<D>` costs only **+6 LOC and +91 typed keystrokes** on 2/20 problems (`noi21p3`, `ucrpc21g`), and across all 611 ideal ports only 3 files (0.5%) reset a `Bool` array in a loop. Worse, `stdlib/marks.cp` uses 8-byte `[D]Int` stamps: on `ucrpc21g`'s `10^7`-state table, `Marks<(Cell, 0..=coins)>` blows RAM from **10 MB to 80 MB**, whereas a 1-line touched-list reset over the already-built `next` list achieves the identical O(|next|) runtime in 10 MB with zero new types. `Marks<D>` also cannot help value tables (`yac6p6`'s `dp2.fill(none)`).
- **Tension 5 (`ask()`, `assume`, `@interactor`, `read(gen:)` vs. `print(flush: true)` + `read()` + `assert` + explicit channels) — Winner: Split Decision (v2.1 WINS on `ask()` and `read(gen:)`; v2.1 LOSES on `assume` as a separate keyword, one-way interactive sends, and `@interactor`'s overloaded `read`/`print`).** Dropping `ask()` costs **+5 LOC, +105 typed keystrokes, and +20 punctuation chars** across 2 interactive problems and reopens the classic forgotten-flush ILE bug (`cco01p4io`). Conversely, `assume` saves **0 LOC and 0 keystrokes** over `assert` while causing real misuse (`cco24p4` writes `assume` for a postcondition check), `dmopc20c6p6` breaks `ask()`'s shared query budget on one-way commands (`E`/`F`), and `@interactor`'s `read()`/`read_output()`/`print()` overloading invites channel-direction deadlocks compared to an explicit `sol.recv()`/`sol.send(...)` channel.

---

## Methodology & The 20-Problem Multi-Topic Benchmark Suite

### Counting & Scoring Rules

All metrics are computed mechanically over comment-stripped, non-blank source lines (identical to `proto/loc.py` line filtering, with trailing whitespace stripped):

- **LOC:** count of non-blank, non-comment lines.
- **Raw characters:** exact byte length of the comment-stripped program including indentation and newlines (captures indentation tax such as wrapping a file in `for _ in 0..read::<Int>():`).
- **Typed keystrokes:** sum of `len(line.strip())` across all lines (measures indentation-independent typing load).
- **Punctuation & symbol density:** count of all non-alphanumeric, non-underscore, non-whitespace characters (`[^A-Za-z0-9_\s]`), reported both as a raw count and as a percentage of typed keystrokes, broken down into brackets `()[]{}<>`, single colons `:`, double colons `::`, question marks `?`, and headline sigils `@!?`.
- **2 AM Readability & Bug-Proneness:** scored 1–5 (5 = clearest / safest under contest fatigue), matching Syntax Bakeoff #1 (`research/archive/architecture/syntax-bakeoff-1.md`).

Because older ports in `proto/ports/` were written before v2.1 landed (e.g., `coci21c1p3` manually defines `fn plus`, `yac6p6` guards every `?Int + Int` with `if let`, `ucrpc21g` writes `seen.fill(false)` with a `FRICTION F1` comment, and `noip22p2` attempts an illegal `[mut empty, mut single, mut ripe]` literal), we first normalize all 20 ports to a **canonical CP Lang v2.1 baseline** strictly faithful to `research/archive/architecture/language-architecture-v2.1.md`, and then apply targeted transformations for each competing alternative.

### The 20-Problem Suite (7 Multi-Topic Categories)

Every problem is a real, high-difficulty competitive-programming submission ported in `proto/ports/` that combines at least two distinct topics and directly exercises one or more of the five v2.1 tensions.

**DP + Data Structures (3 problems)**
- **`yac6p6`** (`proto/ports/scout_batch_39/yac6p6.cp`, orig `5150025.cpp` 170 LOC): 3D grid prefix/suffix DP with query bucketing; 71 v2.1 LOC, 2,234 typed chars, 551 punct (24.7%). Tests **T1** (7 tropical `?Int` transitions) and **T4** (`dp2.fill(none)` value-table boundary).
- **`oly20practice139`** (`proto/ports/scout_batch_41/oly20practice139.cp`, orig `6415690.cpp` 167 LOC): Sqrt-decomposition convex-hull blocks with coordinate compression; 55 v2.1 LOC, 1,574 typed chars, 341 punct (21.7%). Tests **T3** (`for mut b in blocks` vs. indexed `blocks[i]` and keyed `blocks[bi]`).
- **`etsr23r2p4`** (`proto/ports/scout_batch_43/etsr23r2p4.cp`, orig `5494510.cpp` 87 LOC): Multitest bitmask sliding-window DP over `Subset<Win>` and `?Item`; 22 v2.1 LOC, 597 typed chars, 128 punct (21.4%). Tests **T2** (`@memo(hash)` inside whole-file `@multitest`) and **T1** (`?? (n - 1)`).

**Trees + HLD / Rerooting (3 problems)**
- **`coci21c1p3`** (`proto/ports/scout_batch_35/coci21c1p3.cp`, orig `4080960.cpp` 177 LOC): Unicyclic tree DP combining `Dsu<V>`, `Rooted<V>`, nested `@memo`, and `?Int` min-plus DP; 23 v2.1 LOC, 757 typed chars, 141 punct (18.6%). Tests **T1** (eliminating `fn plus`) and **T2** (lexical block-scoped `@memo`).
- **`noi22p4`** (`proto/ports/scout_batch_50/noi22p4.cp`, orig `4802121.cpp` 174 LOC): Tree isomorphism under `<= 5` leaf deletions with `SortedBag`, `Subset<0..5>`, and shared multitest header `(tc, tests, k)`; 41 v2.1 LOC, 1,345 typed chars, 303 punct (22.5%). Tests **T2** (`@multitest(count: tests)` + nested `@memo fn prune`).
- **`sdoi17r1p2`** (`proto/ports/scout_batch_26/sdoi17r1p2.cp`, orig `5463990.cpp` 376 LOC): Link-cut-style HLD tree coloring with `Rooted<V>`, `LazySegTree<Rooted<V>.Pre>`, and `SegTree.max_right`; 43 v2.1 LOC, 1,331 typed chars, 342 punct (25.7%). Tests **T1** (`2 * f.get(...)!` non-lifting `*` boundary) and **T3** (plain `t.at[p]`/`t.tin[v]` indexing).

**Graphs + Dijkstra / SCC (3 problems)**
- **`etsr23r3p3`** (`proto/ports/scout_batch_43/etsr23r3p3.cp`, orig `5521298.cpp` 194 LOC): Multitest Dijkstra + Kruskal Reconstruction Tree via `nodes.push_id` + `.freeze()` + `Rooted.lift_while`; 34 v2.1 LOC, 1,153 typed chars, 270 punct (23.4%). Tests **T2** (`@multitest`) and **T1** (non-lifting `?Int > Int`).
- **`noi21p3`** (`proto/ports/scout_batch_15/noi21p3.cp`, orig `5669954.cpp` 379 LOC): SCC condensation + DAG spanning tree + `virtual_tree` + per-query forward/backward BFS; 81 v2.1 LOC, 1,794 typed chars, 424 punct (23.6%). Tests **T4** (`Marks<C>` vs. `for u in vt.nodes` reset).
- **`cco22p5`** (`proto/ports/scout_batch_36/cco22p5.cp`, orig `4801885.cpp` 174 LOC): Two-graph connectivity with rollbackable small-to-large DSU and `HashMap` inclusion-exclusion; 74 v2.1 LOC, 1,969 typed chars, 464 punct (23.6%). Tests **T3** (`.take()` vs. illegal `.absorb()`) and **T1** (`ans min=`).

**Strings + Hashing / Automata (3 problems)**
- **`noi18p3`** (`proto/ports/scout_batch_22/noi18p3.cp`, orig `6513115.cpp` 532 LOC): Suffix Automaton + persistent/mergeable segment tree arena; 91 v2.1 LOC, 2,495 typed chars, 524 punct (21.0%). Tests **T3** (RHS-before-LHS arena growth `left[node] = single(...)`).
- **`etsr23r1p3`** (`proto/ports/scout_batch_43/etsr23r1p3.cp`, orig `5494303.cpp` 79 LOC): Multitest backward character automaton + `Subset<Alpha>` bitmask DP where `none` means impossible; 24 v2.1 LOC, 550 typed chars, 113 punct (20.5%). Tests **T1** (absorbing-`none` failure counterexample) and **T2** (`@multitest`).
- **`noi20p5`** (`proto/ports/scout_batch_50/noi20p5.cp`, orig `3803796.cpp` 131 LOC): Multitest binary-tree spine trie arena with `(?V, ?V)` tuple matching; 29 v2.1 LOC, 908 typed chars, 221 punct (24.3%). Tests **T3** (`trie[o].next[s] ??= trie.push_id(Node())` and `trie[step(...)].ends = true`) and **T2** (`@multitest`).

**Geometry / Sweepline (2 problems)**
- **`noi23p1`** (`proto/ports/scout_batch_50/noi23p1.cp`, orig `5723056.cpp` 161 LOC): Horizontal/vertical/diagonal stroke union sweepline with `SortedBag.rank` and enum events; 32 v2.1 LOC, 1,360 typed chars, 315 punct (23.2%). Tests **T3** (mutable `out.last!.hi max=` vs. `out[out.len - 1].hi max=`).
- **`ioi07p2`** (`proto/ports/scout_batch_16/ioi07p2.cp`, orig `5133897.cpp` 162 LOC): Planar half-edge face tracing via `Dsu<Side>` + dual-face BFS + non-crossing lattice generator; 52 v2.1 LOC, 1,706 typed chars, 385 punct (22.6%). Tests **T5** (`read(gen:)` + `assume`) and **T1** (`wet_at[a] + 1`).

**Interactive / Constructive (3 problems)**
- **`dmopc20c6p6`** (`proto/ports/scout_batch_34/dmopc20c6p6.cp`, orig `5200785.cpp` 165 LOC): Interactive Water Jugs with mixed one-way (`E`/`F`) and request-reply (`D`) queries under a shared `4n` budget; 80 v2.1 LOC, 1,566 typed chars, 292 punct (18.6%). Tests **T5** (`ask()` vs. one-way `print()` budget split).
- **`cco01p4io`** (`proto/ports/scout_batch_47/cco01p4io.cp`, orig `5034487.cpp` 85 LOC): Interactive Two-Egg Drop with `@memo` and co-located `@interactor`; 49 v2.1 LOC, 992 typed chars, 212 punct (21.4%). Tests **T5** (`ask()` + `@interactor` channels), **T2** (flat `@memo`), and **T1** (`best min= (cost, i)`).
- **`cco24p4`** (`proto/ports/scout_batch_47/cco24p4.cp`, orig `6435484.cpp` 174 LOC): Constructive Tree Rendezvous schedule over `Rooted<V>` and `[Bool, V, T]V`; 38 v2.1 LOC, 1,051 typed chars, 215 punct (20.5%). Tests **T5** (`assume` postcondition misuse vs. `assert`).

**Messy Simulation (3 problems)**
- **`noip22p2`** (`proto/ports/scout_batch_50/noip22p2.cp`, orig `5445681.cpp` 177 LOC): Multitest constructive card-clearing simulation with `Deque<Card>` stacks and 3 mutable `Set<S>` pools; 54 v2.1 LOC, 1,399 typed chars, 320 punct (22.9%). Tests **T1** (`?Card == Card`), **T2** (`@multitest`), and **T3** (`waiting[c].take()` and illegal `[mut ...]`).
- **`noi22p1`** (`proto/ports/scout_batch_50/noi22p1.cp`, orig `4800882.cpp` 156 LOC): Strict-majority sequence concatenation with `Deque<Int>`, `HashMap`, and 20-bit counters; 47 v2.1 LOC, 1,363 typed chars, 331 punct (24.3%). Tests **T3** (ordered small-to-large `.take().rev` vs. unusable `.absorb()`).
- **`ucrpc21g`** (`proto/ports/scout_batch_31/ucrpc21g.cp`, orig `3944717.cpp` 95 LOC): 105-tick grid coin/bomb simulation over `100 x 100 x 1001 = 10^7` reachable states; 23 v2.1 LOC, 738 typed chars, 168 punct (22.8%). Tests **T4** (`Marks<(Cell, 0..=coins)>` 80 MB RAM cliff vs. 10 MB touched-list clear vs. `.fill(false)` TLE).

---

## Tension 1: Tropical Optionals vs. Strict `?T` + Explicit `??` / `if let`

### The Competing Designs

- **CP Lang v2.1 (Tropical Optionals):** `none` is the identity for `min`, `max`, `min=`, `max=`, and the absorber for numeric `?T` `+`, `-`, `+=`, `-=`, unary `-`, and `sum`. Equality (`==`, `!=`) auto-lifts `T` to `?T`; ordering (`<`, `>`) does not. Unparenthesized `??` mixed with binary arithmetic is a compile error.
- **Alternative 1A (Strict `?T` + Explicit `??` / `if let`):** No implicit lifting on `?T` at all. Every optional relaxation must unwrap with `if let` or `?? Int.MAX`, and `?T == T` must write `== some(x)`.
- **Alternative 1B (Hybrid — `min=`/`max=` & `==` only, no `?T + ?T`):** Keep `min=`/`max=` identity and `==` auto-lift, but require `if let` for arithmetic (`+`, `-`) on `?T`.

### Objective Measurements Across All 20 Problems

Across all 611 canonical ideal ports, `?T` appears in **181 files (29.6%)**, `??` in **94 (15.4%)**, `!` in **220 (36.0%)**, and `min=`/`max=` in **111 (18.2%)** — making optional ergonomics the single highest-frequency tension in this shootout.

- **20-problem v2.1 baseline:** 963 LOC, 33,317 raw chars, 26,882 typed keystrokes, 6,060 punct (22.54%), `64` `?`, `1` `::`, `650` `:`, `2,714` brackets, `118` `@!?`.
- **Alternative 1A (Strict `?T`, 7/20 affected):** 967 LOC (**+4**), 34,330 raw chars (**+1,013**), 27,827 typed keystrokes (**+945**, **+9.8%** on the 7 affected files), 6,281 punct (**+221**, +0.03pp), `100` `?` (**+36**), `673` `:` (**+23**), `2,787` brackets (**+73**), `156` `@!?` (**+38**). Per-problem keystroke penalties: `yac6p6` +606 (+26 `?`), `coci21c1p3` +245 (+4 `?`), `cco22p5` +36 (+4 `?`), `cco01p4io` +32, `etsr23r1p3` +13, `noip22p2` +12, `ioi07p2` +1.
- **Alternative 1B (Hybrid, 3/20 affected):** 966 LOC (**+3**), 33,679 raw chars (**+362**), 27,189 typed keystrokes (**+307**), 6,096 punct (**+36**, -0.12pp), `64` `?` (**+0**), `663` `:` (**+13**), `2,720` brackets (**+6**), `119` `@!?` (**+1**). Per-problem keystroke penalties: `coci21c1p3` +163, `yac6p6` +143, `ioi07p2` +1.

### Side-by-Side Real Corpus Evidence

**Where v2.1 dominates — `yac6p6` (3D grid DP) and `coci21c1p3` (tree DP):**

In `yac6p6`, six grid transitions plus the query-combination loop collapse from defensive `if let` + sentinel plumbing into mathematical DP equations.

CP Lang v2.1 (Tropical Optionals — 7 lines, 344 typed chars, 0 `?` inside the loop):
```cp
for k in 0..=kv:
    if let nr = r + 1 if nr <= x2:
        dp[r, c, k] min= dp[nr, c, k] + a[r, c]
        if k < kv: dp[r, c, k] min= dp[nr, c, k + 1]
    if let nc = c + 1 if nc <= y2:
        dp[r, c, k] min= dp[r, nc, k] + a[r, c]
        if k < kv: dp[r, c, k] min= dp[r, nc, k + 1]
```

Alternative 1A (Strict `?T` + `?? Int.MAX` — 7 lines, 646 typed chars, 8 `?` inside this loop alone):
```cp
for k in 0..=kv:
    if let nr = r + 1 if nr <= x2:
        if let nxt = dp[nr, c, k]: dp[r, c, k] = min(dp[r, c, k] ?? Int.MAX, nxt + a[r, c])
        if k < kv: if let nxt = dp[nr, c, k + 1]: dp[r, c, k] = min(dp[r, c, k] ?? Int.MAX, nxt)
    if let nc = c + 1 if nc <= y2:
        if let nxt = dp[r, nc, k]: dp[r, c, k] = min(dp[r, c, k] ?? Int.MAX, nxt + a[r, c])
        if k < kv: if let nxt = dp[r, nc, k + 1]: dp[r, c, k] = min(dp[r, c, k] ?? Int.MAX, nxt)
```

In `coci21c1p3`, the v2 port had to define a bespoke helper `fn plus(x: ?Int, y: ?Int) -> ?Int: x? + y?` just to write two lines of tree DP. v2.1 deletes the helper completely and expresses the min-plus convolution directly:

CP Lang v2.1:
```cp
for v in tree.children(u):
    let (off, lit) = (blues(v, false, on), blues(v, true, on))
    one = min(one + off, zero + lit)
    zero += off
```

Alternative 1A (Strict `?T`):
```cp
for v in tree.children(u):
    let (off, lit) = (blues(v, false, on), blues(v, true, on))
    let c1 = if let x = one and let y = off: some(x + y) else: none
    let c2 = if let x = zero and let y = lit: some(x + y) else: none
    one = if let x = c1 and let y = c2: some(min(x, y)) else: c1 ?? c2
    zero = if let x = zero and let y = off: some(x + y) else: none
```

**Where v2.1 has a real 2 AM footgun — `etsr23r1p3` (absorbing-`none` failure) and `sdoi17r1p2` (`*` cliff):**

Tropical Optionals assume `?Int`'s `none` plays the role of **tropical `+∞` (or `-∞`)**: an unreachable candidate that `min`/`max` silently ignores while `+` absorbs it. In `etsr23r1p3`, `dp[mask]` requires **every** letter `c in mask` to have a valid previous position `prv[prev_pos, c]` — here `none` means **impossible/poisoned**, NOT `+∞`!

Correct `etsr23r1p3` (explicit failure check required in BOTH v2.1 and Alt 1A):
```cp
var best: ?Int = n
for c in Alpha if c in mask:
    if let prev_pos = dp[mask.without(c)] and let hit = prv[prev_pos, c]:
        best min= hit
    else:
        best = none
        break
dp[mask] = best
```

A fatigued contestant who internalizes "v2.1 `?Int` propagates `none`" and writes `best min= prv[prev_pos, c]` gets a **silent Wrong Answer**: when `prv[prev_pos, c]` is `none`, `min=` treats `none` as `+∞` identity and keeps the finite `best` from other letters instead of setting `best = none`! Notice also the sharp boundary in `sdoi17r1p2` line 65 (`print(f.get(t.tin[x])! + f.get(t.tin[y])! - 2 * f.get(t.tin[top])! + 1)`): because v2.1 lifts `+`/`-` on `?Int` but deliberately does **not** lift `*`, `2 * f.get(...)` still forces `!` unwraps.

### 2 AM Readability & Bug-Proneness

- **CP Lang v2.1 (Tropical Optionals):** Readability **5/5**, Bug-Proneness **4/5** (eliminates `INF + cost` signed-overflow bugs and 12+ lines of sentinel boilerplate; loses 1 point to the `etsr23r1p3` `min=`-skips-`none` vs. `+`-absorbs-`none` asymmetry).
- **Alternative 1A (Strict `?T` + `??`):** Readability **2/5**, Bug-Proneness **3/5** (`?? Int.MAX` reintroduces `Int.MAX + a[r, c]` overflow traps if `??` leaks into arithmetic, and nested `if let` buries DP recurrences).
- **Alternative 1B (Hybrid — no `?T + ?T`):** Readability **3.5/5**, Bug-Proneness **4/5** (still forces `if let` / `fn plus` on every `dp[u] + weight` transition, which is the core of DP!).

**Clear Winner: CP Lang v2.1 (Tropical Optionals).** Saving nearly 1,000 keystrokes and 36 `?` characters across 7/20 hard problems while eliminating `INF` overflow bugs is the single largest ergonomic win in v2.1. Keep Tropical Optionals intact, with two concrete spec tightenings: (1) highlight the tropical-identity vs. failure-poison distinction prominently in docs ("`min=` ignores `none`, `+` absorbs `none`; if a missing lookup must poison a `min=` loop, check `if let` or `break`"), and (2) state explicitly on the cheat sheet that `*` and `/` do not lift on `?T`.

---

## Tension 2: `@memo(flat|hash)` + `@multitest` / `@multitest(count:)` vs. Zero-Annotation Auto-Dispatch & `for _ in 0..t:`

### The Competing Designs

- **CP Lang v2.1:** Nested block-scoped `@memo` with automatic flat/hash dispatch **plus** explicit `@memo(flat)` / `@memo(hash)` parameter grammar; paired with whole-file `@multitest` **plus** `@multitest(count: expr)` on functions.
- **Alternative 2A (Zero-Decorator Minimalism):** Plain `@memo` only (pure auto-dispatch, zero parameters) and plain `for _ in 0..read::<Int>():` loops (no `@multitest` at all).
- **Alternative 2B (Minimal-Decorator Hybrid):** Keep bare file-top `@multitest` and bare block-scoped `@memo`; **delete** `@memo(flat|hash)` and `@multitest(count:)`.

### Objective Measurements Across All 20 Problems

Across all 611 canonical ideal ports: `@memo` appears in **45 files (7.4%)**, while `@memo(flat|hash)` appears in only **3 files (0.5%)** (`@memo(flat)` is **0/611**). `@multitest` appears in **24 files (3.9%)** alongside 20 manual `for _ in 0..t:` loops, while `@multitest(count:)` appears in **1 file (0.2%)** (`noi22p4`).

- **20-problem v2.1 baseline:** 963 LOC, 33,317 raw chars, 26,882 typed keystrokes, 6,060 punct (22.54%), `1` `::`, `650` `:`, `2,714` brackets, `118` `@!?`.
- **Alternative 2A (Drop ALL `@multitest` + `@memo(flat|hash)`, 6/20 affected):** 962 LOC (**-1**), 33,985 raw chars (**+668** from 4-space indentation across 5 files!), 26,939 typed keystrokes (**+57**), 6,094 punct (**+34**, +0.08pp), `6` `::` (**+5** from `read::<Int>()`), `654` `:` (**+4**), `2,728` brackets (**+14**), `112` `@!?` (**-6**).
- **Alternative 2B (Drop ONLY `@memo(flat|hash)` and `@multitest(count:)`, 2/20 affected):** 962 LOC (**-1**), 33,293 raw chars (**-24**), 26,859 typed keystrokes (**-23**), 6,054 punct (**-6**, 0.00pp), `1` `::` (**+0**), `649` `:` (**-1**), `2,708` brackets (**-6**), `117` `@!?` (**-1**). **Strictly dominates v2.1 on every single metric.**

### Side-by-Side Real Corpus Evidence

**Where v2.1 LOSES outright — `noi22p4` (`@multitest(count: tests)` vs. `for _ in 0..tests:`):**

In `noi22p4`, a shared header `(tc, tests, k)` is read once before the test cases. Because `@multitest(count: tests)` attaches to a function, the contestant has to invent a dummy `fn solve():` AND still indent the entire body by 4 spaces! Meanwhile, v2.1's lexical block-scoped `@memo fn prune` ALREADY resets at the end of every `for _ in 0..tests:` iteration automatically.

CP Lang v2.1 (2 header lines, 36 typed chars, 1 `@`, 1 dummy function name):
```cp
let (tc, tests, k): (Int, 1..=500, 0..=5) = read()
@multitest(count: tests)
fn solve():
    let n: 1..=1e5 = read()
    ...
    @memo
    fn prune(u: Int, v: Int) -> Bool:
        ...
```

Alternative 2B (plain `for` loop — 1 line, 19 typed chars, 0 decorators, identical cache reset):
```cp
let (tc, tests, k): (Int, 1..=500, 0..=5) = read()
for _ in 0..tests:
    let n: 1..=1e5 = read()
    ...
    @memo
    fn prune(u: Int, v: Int) -> Bool:
        ...
```

**Where v2.1 LOSES — `etsr23r2p4` (`@memo(hash)` vs. plain `@memo`):**

In `etsr23r2p4`, `dp`'s first parameter is an unbounded `ind: Int`, so v2.1's automatic flat/hash dispatch already chooses hash storage with zero annotation. Writing `@memo(hash)` adds 6 keystrokes, 2 brackets, and a second grammar form for zero semantic effect:

CP Lang v2.1:
```cp
@memo(hash)
fn dp(ind: Int, prv: ?Item, used: Subset<Win>) -> Int:
```

Alternative 2B (zero-annotation auto-dispatch):
```cp
@memo
fn dp(ind: Int, prv: ?Item, used: Subset<Win>) -> Int:
```

Even in the rare case where every parameter has a bounded domain (`0..n`, `0..=top` as in `noip21p3`), v2.1's runtime table-size check (`product_of_bounds <= 10^7` -> flat array, else hash map) already handles sparse states without manual `(flat)`/`(hash)` flags. And if a contestant genuinely needs manual table layout, `var dp = HashMap<...>()` is standard userland code.

**Where v2.1 WINS — bare file-top `@multitest` (`etsr23r3p3`, `etsr23r1p3`, `noi20p5`, `noip22p2`, `etsr23r2p4`) and block-scoped nested `@memo` (`coci21c1p3`):**

Bare file-top `@multitest` keeps 100% of the program unindented (saving 668 raw chars across 5 problems), avoids `read::<Int>()` turbofish `::` in 5 files, and guarantees zero stale global state across test cases. And block-scoped nested `@memo` in `coci21c1p3` lets `blues(u: V, on: Bool, par_on: Bool)` capture `a_on` and `b_on` immutably from the outer `for` loops, shrinking the cache table 4x (`16n` -> `4n`) and dropping 2 arguments from every recursive call.

### 2 AM Readability & Bug-Proneness

- **CP Lang v2.1 (with `(flat|hash)` & `(count:)`):** Readability **4/5**, Bug-Proneness **4.5/5** (`@multitest(count:)` creates a second, redundant way to write `for _ in 0..tests:`, and `@memo(flat)` invites OOM if someone overrides the `10^7` guard).
- **Alternative 2A (No `@multitest` at all):** Readability **3.5/5**, Bug-Proneness **4/5** (indents entire 60-line solutions and forces `read::<Int>()` turbofish noise).
- **Alternative 2B (Bare `@multitest` + bare nested `@memo`):** Readability **5/5**, Bug-Proneness **5/5** (one obvious way for standard `T`-first multitest files, one obvious `for _ in 0..tests:` loop for custom headers, zero decorator-argument syntax to remember).

**Clear Winner: Alternative 2B (Minimal-Decorator Hybrid).** CP Lang v2.1 **loses** on `@memo(flat|hash)` and `@multitest(count:)` — delete both parameter forms from the language while keeping bare file-top `@multitest` and bare lexical block-scoped `@memo`.

---

## Tension 3: Ownership Ergonomics (`for mut`, `.swap`, `.take()`, `.absorb()`, Mutable `.last`, RHS-before-LHS) vs. Plain Indexing & Minimal Core

### The Competing Designs

- **CP Lang v2.1:** Full ownership-ergonomics suite: `for mut x in xs`, `swap`/`a.swap(i, j)`, `.take()` on containers and `?T`, `.absorb(mut other)` on `List`/`Heap`, mutable `.last`, and RHS-before-LHS assignment evaluation (`a[i] = f(mut a)`).
- **Alternative 3A (Plain Indexing + 2-Primitive Core):** Keep `swap` and `.take()`, drop `for mut`, mutable `.last`, `.absorb()`, and RHS-before-LHS self-mutating index assignments.
- **Alternative 3B (Strict Minimalism — also drop `.take()`):** Replace `.take()` with plain indexed loops / `pop()` loops + explicit `clear()` / `= none`.

### Objective Measurements Across All 20 Problems

Corpus-wide incidence across all 611 canonical ideal ports: `swap(` appears in **20 files (3.3%)**, `.take()` in **2 files (0.3%)** (plus 5 more in recent batches/manual `clear()` workarounds), `for mut` in **0/611**, and `.absorb(` in **0/611**.

- **20-problem v2.1 baseline:** 963 LOC, 33,317 raw chars, 26,882 typed keystrokes, 6,060 punct (22.54%), `64` `?`, `118` `@!?`.
- **Alternative 3A (Drop `for mut`, mut `.last`, RHS-before-LHS; 4/20 affected):** 970 LOC (**+7**), 33,473 raw chars (**+156**), 26,971 typed keystrokes (**+89**), 6,075 punct (**+15**, -0.02pp), `62` `?` (**-2**), `115` `@!?` (**-3**). Per-problem deltas: `noi20p5` +3 LOC/+51 chars, `noi18p3` +4 LOC/+20 chars, `oly20practice139` +0 LOC/+11 chars, `noi23p1` +0 LOC/+7 chars.
- **Alternative 3B (Also drop `.take()`; 7/20 affected):** 975 LOC (**+12**), 33,613 raw chars (**+296**), 27,058 typed keystrokes (**+176**), 6,096 punct (**+36**, -0.01pp). Adds `cco22p5` +3 LOC/+68 chars, `noi22p1` +0 LOC/+10 chars, `noip22p2` +2 LOC/+9 chars.

### Side-by-Side Real Corpus Evidence

**Where v2.1 WINS big — RHS-before-LHS arena safety (`noi18p3`, `noi20p5`), plus a spec gap caught in `noi20p5`:**

In `noi18p3` (mergeable segment tree) and `noi20p5` (trie arena), recursive node allocation grows the same `List` that the assignment writes back into:

CP Lang v2.1 (`noi18p3` lines 68–69 and `noi20p5` lines 16–17):
```cp
// noi18p3: single() pushes to `left`/`right`, possibly reallocating the vector
if at < mid: left[node] = single(at, lo, mid)
else: right[node] = single(at, mid, hi)

// noi20p5: push_id() grows `trie` before writing into `trie[o].next[s]`
fn step(o: Int, s: Step) -> Int:
    trie[o].next[s] ??= trie.push_id(Node())
```

Alternative 3A (no RHS-before-LHS — 7 extra lines, 71 extra keystrokes across the two files):
```cp
// noi18p3
if at < mid:
    let ch = single(at, lo, mid)
    left[node] = ch
else:
    let ch = single(at, mid, hi)
    right[node] = ch

// noi20p5
fn step(o: Int, s: Step) -> Int:
    if let nxt = trie[o].next[s]: return nxt
    let id = trie.push_id(Node())
    trie[o].next[s] = id
    id
```

In C++17 (prior to C++17's sequencing guarantee — and still a real footgun under `-O2` when someone captures `auto& slot = trie[o].next[s]` first), growing a vector during assignment causes use-after-free. v2.1's RHS-before-LHS rule prevents that entire bug class while keeping the natural one-liner. **Spec gap caught in `noi20p5`:** `noi20p5` also writes `trie[step(o, BothGoRight)].ends = true`, where `step(...)` mutates `trie` inside the **LHS index expression** rather than the RHS. v2.1 must explicitly require **evaluate all index expressions and the RHS before taking the `&mut` place address of the container**, closing both halves of arena-growth aliasing cleanly.

**Where v2.1 WINS on `.take()` AND LOSES completely on `.absorb()` (`cco22p5`, `noi22p1`):**

Both `cco22p5` and `noi22p1` are textbook small-to-large merges over an array of containers (`self.members: [V]List<V>` and `seqs: List<Seq>` containing `Deque<Int>`). In **both** problems, `.absorb()` fails on three independent counts:

1. **Same-container mutable aliasing:** `self.members[v].absorb(mut self.members[u])` and `seqs[back].items.absorb(mut seqs[front].items)` take two `&mut` borrows into the same container without a disjointness proof, so the borrow checker rejects them anyway.
2. **Hidden complexity trap in `stdlib/`:** `stdlib/heap.cp` implements `MinHeap.absorb` with an O(1) internal `swap` when `self.len < other.len` (O(min log n) unordered merge), whereas `stdlib/seq.cp` implements `List.absorb` as a plain `for x in other: self.push(x)` (O(|other|) ordered append, NO swap!). Calling `a.absorb(mut b)` on two `List`s silently degrades small-to-large to **O(n^2) TLE**!
3. **Per-element side effects:** Real small-to-large merges almost always update companion tables (`self.rep[x] = v` in `cco22p5`; `seqs[back].count(v, 1)` in `noi22p1`) per moved element, which `.absorb()` cannot express.

Meanwhile, `.take()` solves the borrow conflict cleanly in one line on `cco22p5`, `noi22p1`, and `noip22p2` (`waiting[c].take()` on `?Int`):

CP Lang v2.1 (`.take()` — borrow-clean, zero redundant `moved` copy in `cco22p5`):
```cp
// cco22p5
let moved = self.members[u].take()
for x in moved:
    self.rep[x] = v
    self.members[v].push(x)

// noi22p1
if seqs[front].items.len < seqs[back].items.len:
    for v in seqs[front].items.take().rev:
        seqs[back].items.push_front(v)
        seqs[back].count(v, 1)
    store[z] = back
```

Alternative 3B (without `.take()` — extra temp list + index loop + manual `.clear()` in `cco22p5`):
```cp
// cco22p5
var moved: List<V> = []
for i in 0..self.members[u].len:
    let x = self.members[u][i]
    self.rep[x] = v
    self.members[v].push(x)
    moved.push(x)
self.members[u].clear()
```

**Where v2.1 is marginal / a mild lure — `for mut` (`oly20practice139`, `noip22p2`) and mutable `.last` (`noi23p1`):**

Across all 20 hard problems, `for mut` triggers exactly **once** (`oly20practice139` line 58): `for mut b in blocks: best max= b.query()` saves **0 LOC and 11 keystrokes** over `for i in 0..blocks.len: best max= blocks[i].query()` — and 9 lines later in the same file, lines 67–68 still have to use indexed `blocks[bi].add += 1` and `blocks[pos / B].add_prefix(pos % B)`. Worse, in `noip22p2` line 24, the port author was tempted into writing the illegal `for pool in [mut empty, mut single, mut ripe]: pool.remove(s)` (v2.1 forbids stored borrows in list literals!), whereas three plain statements (`empty.remove(s); single.remove(s); ripe.remove(s)`) are immediately legal and clearer. Similarly, mutable `.last` saves 7 keystrokes in `noi23p1` (`out.last!.hi max= s.hi` vs. `out[out.len - 1].hi max= s.hi`) at the cost of adding an unwrap `!`.

### 2 AM Readability & Bug-Proneness

- **CP Lang v2.1 (full toolkit):** Readability **4.5/5**, Bug-Proneness **3.5/5** (`.absorb()`'s `List` vs. `MinHeap` O(|other|)-vs-O(min) split is a real TLE trap, and `for mut` slightly encourages illegal `[mut ...]` attempts).
- **Alternative 3A (drop `for mut`, mut `.last`, RHS-before-LHS):** Readability **4/5**, Bug-Proneness **4/5** (extra temp variables in `noi18p3` and `noi20p5` add clutter).
- **Recommended Split (keep RHS-before-LHS + index-before-place-borrow, `.take()`, `swap`; drop `List.absorb`; treat `for mut` as low-priority sugar):** Readability **5/5**, Bug-Proneness **5/5**.

**Clear Winner: Split Decision.** Keep RHS-before-LHS (extended to **index-before-place-borrow** for `noi20p5`), `.take()` (both container and `?T`), and `swap`. **Delete `List.absorb()`** (keeping `.absorb()` only on unordered heaps/bags if at all, or renaming it to `.merge_from()` so ordered lists never carry a quadratic TLE trap). `for mut` is harmless syntactic sugar with zero new keywords, but it moves the needle by only 11 keystrokes across 20 hard problems.

---

## Tension 4: `Marks<D>` Generation-Stamped Set vs. `[D]Bool` + `.fill()` / Touched-List Clear

### The Competing Designs

- **CP Lang v2.1 (`Marks<D>`):** Dedicated `stdlib/marks.cp` struct storing `stamp: [D]Int`, `gen: Int`, `cnt: Int` with O(1) `.add(v) -> Bool`, `v in m`, `.remove(v)`, and O(1) `.clear()`, alongside `.fill(val)` for general arrays.
- **Alternative 4 (Plain `[D]Bool` + Touched-List Clear / `.fill()` — Zero New Type):** No `Marks<D>` type in the core mental model; use `[D]Bool` with `.fill(false)` for small/one-shot tables and a touched-list reset (`for s in visited: seen[s] = false`) inside hot loops.

### Objective Measurements Across All 20 Problems

Across all 611 canonical ideal ports, `seen`/`vis` `Bool` arrays appear in **32 files (5.2%)**, `.fill(` appears in **4 files (0.7%)**, `Marks<` appears in **1 file (0.2%)**, and explicit loop-reset `Bool` visited arrays appear in only **3 files (0.5%)**.

- **20-problem v2.1 baseline:** 963 LOC, 33,317 raw chars, 26,882 typed keystrokes, 6,060 punct (22.54%).
- **Alternative 4 (No `Marks<D>`, 2/20 affected):** 969 LOC (**+6**), 33,518 raw chars (**+201**), 26,973 typed keystrokes (**+91**), 6,073 punct (**+13**, -0.03pp), `654` `:` (**+4**), `2,720` brackets (**+6**). Per-problem deltas: `noi21p3` +4 LOC/+40 chars, `ucrpc21g` +2 LOC/+51 chars.

### Side-by-Side Real Corpus Evidence

**Where `Marks<D>` LOSES on real judge constraints — `ucrpc21g` (80 MB vs. 10 MB RAM cliff!):**

In `ucrpc21g`, the state space is `100 x 100 x 1001 = 10,010,000` `(Cell, coins)` pairs across 105 ticks. Calling `seen.fill(false)` per tick writes `1.05 * 10^9` bytes (**TLE**). Replacing it with `Marks<(Cell, 0..=coins)>` fixes the TLE — **but triggers an 8x memory regression** because `stdlib/marks.cp` allocates `stamp: [D]Int` (8-byte 64-bit `Int`s = **80 MB**, versus **10 MB** for `[Cell, 0..=coins]Bool`!), and requires product-domain support `Marks<(Cell, 0..=coins)>`. Yet `ucrpc21g` ALREADY builds `next: List<(Cell, Int)>` containing the exact set of states marked during the tick:

CP Lang v2.1 (`Marks` — 80 MB RAM, requires tuple-domain `Marks<(Cell, 0..=coins)>`):
```cp
var seen = Marks<(Cell, 0..=coins)>()
var cur: List<(Cell, Int)> = [(start, 0)]
for t in Tick:
    seen.clear()
    var next: List<(Cell, Int)> = []
    for (p, held) in cur:
        let have = if bomb[t, p]: held / 2 else: held + gain[t, p]
        for q in chain([p], p.adj4):
            if seen.add((q, have)): next.push((q, have))
    cur = next
```

Alternative 4 (Plain `[Cell, 0..=coins]Bool` + 1-line touched-list clear — **10 MB RAM**, identical O(|next|) time, 0 new stdlib types):
```cp
var seen: [Cell, 0..=coins]Bool = false
var cur: List<(Cell, Int)> = [(start, 0)]
for t in Tick:
    var next: List<(Cell, Int)> = []
    for (p, held) in cur:
        let have = if bomb[t, p]: held / 2 else: held + gain[t, p]
        for q in chain([p], p.adj4):
            if not seen[q, have]:
                seen[q, have] = true
                next.push((q, have))
    for (q, have) in next: seen[q, have] = false
    cur = next
```

**Where `Marks<D>` helps (`noi21p3`) — and where it hits a hard wall (`yac6p6`):**

In `noi21p3` (virtual tree per-query BFS), `Marks<C>` shines ergonomically: `if vis_f.add(v): qf.push(v)` replaces 3 lines (`if not vis_f[v]: vis_f[v] = true; qf.push(v)`) and prevents the classic 2 AM typo `if not vis_f[u]: vis_f[v] = true`, saving 4 LOC and 40 keystrokes (`2 * 8 * 3e5 = 4.8 MB`, well inside ML). Yet `noi21p3` STILL has to run `for u in vt.nodes: fwd[u] = []; rev[u] = []` every query, so the touched-list loop does not disappear. And in `yac6p6`, the per-center DP reset is `dp2.fill(none)` on `[0..3, 0..3, 0..=MAXK]?Int` — a value table, not a `Bool` set — where `Marks<D>` cannot apply at all.

### 2 AM Readability & Bug-Proneness

- **CP Lang v2.1 (`Marks<D>`):** Readability **4.5/5**, Bug-Proneness **3.5/5** (`if seen.add(v):` eliminates double-index typos, but 8-byte `[D]Int` backing is a silent MLE trap on `10^7`-state grids like `ucrpc21g`).
- **Alternative 4 (`[D]Bool` + touched-list / `.fill()`):** Readability **4.5/5**, Bug-Proneness **4/5** (predictable 1-byte/elem RAM, works uniformly for `Bool`, `Int`, `?Int`, and `List` adjacency tables; slight risk of forgetting `seen[q, have] = false` on early `continue`).

**Clear Winner: Alternative 4 (`[D]Bool` + Touched-List / `.fill()`) as the primary idiom; demote `Marks<D>` from the core cheat sheet to an optional `std.marks` utility.** Because `Marks<D>` costs zero keywords and only ~40 lines of pure userland CP Lang, there is no harm in keeping `std.marks` for 1D graph problems like `noi21p3` — **provided `stdlib/marks.cp` shrinks `stamp: [D]Int` to `stamp: [D]I32`** (halving RAM from 8x to 4x, still safe for `2 * 10^9` clears!) and the cheat sheet teaches touched-list reset first for memory-tight multi-dimensional DP/BFS.

---

## Tension 5: Interactive & Constructive Ergonomics (`ask()`, `assume`, `@interactor`, `read(gen:)`) vs. Plain `print(flush: true)` + `read()` + `assert`

### The Competing Designs

- **CP Lang v2.1:** Builtin `ask(args...) -> T` (prints space-separated args, flushes, parses typed reply), `assume <cond>` keyword for generator/input invariants, `read(gen: f)`, and `@interactor` / `@checker` blocks (where `@interactor` uses `read()` for judge input, `read_output()` for solution stdout, and `print()` for solution stdin).
- **Alternative 5 (Zero-Keyword Minimal Protocol):** Drop `ask()` in favor of `print(..., flush: true)` + `read()`, drop the `assume` keyword in favor of `assert` + `read(gen: f)` / `@checker`, and give `@interactor` an explicit typed channel parameter (`fn judge(sol: Interactor)` with `sol.recv()` / `sol.send(...)`).

### Objective Measurements Across All 20 Problems

Across all 611 canonical ideal ports: `read(gen:)` appears in **61 files (10.0%)**, `ask(` in **16 files (2.6%)**, `@checker` in **8 (1.3%)**, `@interactor` in **7 (1.1%)**, and `assume` in only **6 files (1.0%)**.

- **20-problem v2.1 baseline:** 963 LOC, 33,317 raw chars, 26,882 typed keystrokes, 6,060 punct (22.54%).
- **Alternative 5 (Drop `ask()` and `assume`, 4/20 affected):** 968 LOC (**+5**), 33,439 raw chars (**+122**), 26,987 typed keystrokes (**+105**), 6,080 punct (**+20**, -0.01pp), `655` `:` (**+5**), `2,724` brackets (**+10**). Per-problem deltas: `cco01p4io` +4 LOC/+84 chars, `dmopc20c6p6` +1 LOC/+21 chars, `cco24p4` **0 LOC / 0 chars**, `ioi07p2` **0 LOC / 0 chars**.

### Side-by-Side Real Corpus Evidence

**Where v2.1 WINS big — `ask(...)` on request-reply queries (`cco01p4io`):**

CP Lang v2.1 (4 lines across the file, auto-flushing by construction):
```cp
let n: 1..=100 = ask("GetN")
let d: 1..=100 = ask("GetD")
...
let broke: 0..=1 = ask("Drop", l + i)
```

Alternative 5 (8 lines, +84 keystrokes, 4 chances to forget `flush: true` and get ILE!):
```cp
print("GetN", flush: true)
let n: 1..=100 = read()
print("GetD", flush: true)
let d: 1..=100 = read()
...
print("Drop", l + i, flush: true)
let broke: 0..=1 = read()
```

**Where v2.1 LOSES (1 of 3) — mixed one-way & request-reply interactors (`dmopc20c6p6`):**

In `dmopc20c6p6` (Water Jugs), `E x` (empty) and `F x` (fill) are **one-way commands** that return no response yet still consume the shared `4n` query budget alongside `D x y` (which returns an `Int`):

```cp
// dmopc20c6p6.cp lines 17-24
fn empty_jug(x: Jug):
    print("E {x}")          // Does NOT auto-flush or count as an interactive query!

fn fill_jug(x: Jug):
    print("F {x}")          // Same bug risk!

fn pour_jug(x: Jug, y: Jug) -> Int:
    ask("D", x, y)          // Auto-flushes and reads Int reply
```

Here `ask()` only solves half the protocol; contestants still have to remember `flush: true` on `print("E {x}")` and `print("Decide", l)` (`cco01p4io` line 27)! Making `print()` auto-flush whenever a program contains `ask(` (or providing a tiny zero-keyword `tell(...)` / `send(...)` partner to `ask(...)`) eliminates this hole completely without adding a keyword.

**Where v2.1 LOSES (2 of 3) — `assume` as a separate reserved keyword (`cco24p4` vs. `ioi07p2`):**

`assume` saves **0 LOC and 0 keystrokes** over `assert` (both are 6 characters!), appears in only 1.0% of ports, and ALREADY caused conceptual confusion in real corpus code. In `ioi07p2`, `assume` guards a generated input (`read(gen: gen_walls)` — where the comment itself notes rejection sampling would fail without `gen_walls`!). In `cco24p4` line 57, the author accidentally wrote `assume` as a **solution postcondition self-check**:

```cp
// cco24p4.cp lines 54-57 — postcondition self-check miswritten with `assume`!
for u in V:
    for v in V if u != v:
        let meet = first(t.to_idx for t in T if ans[false, u, t] == ans[true, v, t])!
        assume meet <= 13 * tree.dist(u, v)   // Should be `assert`!
```

Having both `assert` and `assume` forces contestants at 2 AM to ask "is this a generator precondition (`assume`) or a runtime invariant (`assert`)?" — and in `ioi07p2`, `read(gen: gen_walls)` already constructs valid inputs directly while `assert` validates them in debug/stress mode. Dropping `assume` frees **1 reserved keyword** (`40 -> 39`) and unifies validation around `assert` + `read(gen:)` + `@checker`.

**Where v2.1 LOSES (3 of 3) — `@interactor` channel directionality (`cco01p4io`):**

Look at `cco01p4io` lines 43–64: inside `@interactor fn judge()`, `read()` reads the hidden test case, `read_output()` reads the solution's stdout, and `print()` writes to the solution's stdin:

CP Lang v2.1 (overloaded `read` / `print` — 1 typo deadlocks the pipe):
```cp
@interactor
fn judge():
    let hidden_n: 1..=100 = read()
    let hidden_f: 1..=hidden_n = read()
    ...
    while true:
        let cmd: Str = read_output()
        if cmd == "GetN": print(hidden_n)
        elif cmd == "Drop":
            let x: Int = read_output()
            ...
            print(b)
```

Alternative 5 (explicit `sol` channel — zero channel-direction ambiguity, ~+24 chars total):
```cp
@interactor
fn judge(sol: Interactor):
    let hidden_n: 1..=100 = read()
    let hidden_f: 1..=hidden_n = read()
    ...
    while true:
        let cmd: Str = sol.recv()
        if cmd == "GetN": sol.send(hidden_n)
        elif cmd == "Drop":
            let x: Int = sol.recv()
            ...
            sol.send(b)
```

### 2 AM Readability & Bug-Proneness

- **CP Lang v2.1 (current):** Readability **4/5**, Bug-Proneness **3.5/5** (`ask()` prevents ILE bugs, but `assume` vs. `assert` confusion, un-flushed one-way `print()` in `dmopc20c6p6`, and `read()`/`read_output()`/`print()` overloading in `@interactor` all hurt safety).
- **Alternative 5 (drop `ask()` too):** Readability **3.5/5**, Bug-Proneness **3/5** (reintroduces forgotten `flush: true` ILE bugs).
- **Recommended Split (keep `ask()` + `read(gen:)` + auto-flush `print` when interactive; drop `assume` keyword; pass explicit `sol: Interactor` to `@interactor`):** Readability **5/5**, Bug-Proneness **5/5**.

**Clear Winner: Split Decision.** Keep `ask(...)` and `read(gen:)` (huge ILE-prevention and stress-testing wins). **Drop `assume` as a reserved keyword** (unify with `assert` + `read(gen:)` / `@checker`), auto-flush `print()` in programs using `ask()` (fixing `dmopc20c6p6` and `cco01p4io`'s `Decide`), and replace `@interactor`'s overloaded `read_output()`/`print()` with an explicit `sol.recv()`/`sol.send(...)` channel.

---

## Consolidated Verdict & Actionable Recommendations for the Language Architect

Combining the five clear winners into a **"v2.1-Trimmed"** specification yields a strictly smaller, safer, and cleaner language than v2.1 without surrendering a single major ergonomic win:

1. **Keep unchanged:** Tropical Optionals (`min`/`max`/`min=`/`max=` identity, `+`/`-`/`+=`/`-=` absorption, `==`/`!=` auto-lift, parenthesized `??` + arithmetic rule), bare file-top `@multitest`, bare lexical block-scoped `@memo` (pure auto-dispatch), `.take()` (containers and `?T`), `swap`, `ask(...)`, and `read(gen:)`.
2. **Delete from v2.1 (4 clear losses):**
   - Delete `@memo(flat)` / `@memo(hash)` parameter grammar (0.5% corpus incidence; redundant on `etsr23r2p4`).
   - Delete `@multitest(count: expr)` function form (0.2% corpus incidence; strictly loses to `for _ in 0..tests:` on `noi22p4`).
   - Delete `List.absorb(mut other)` (0% corpus incidence; blocked by same-container aliasing on `cco22p5` and `noi22p1`, and hides an O(|other|) vs. O(min) TLE cliff relative to `MinHeap.absorb`).
   - Delete `assume` from the reserved keyword budget (`40 -> 39` keywords; 0 keystroke savings over `assert`, and caused postcondition misuse in `cco24p4`).
3. **Fix three subtle v2.1 engineering hazards uncovered by this shootout:**
   - **Extend RHS-before-LHS to index-before-place-borrow** so `trie[step(o, BothGoRight)].ends = true` (`noi20p5`) is guaranteed memory-safe alongside `trie[o].next[s] ??= trie.push_id(Node())` and `left[node] = single(...)` (`noi18p3`).
   - **Shrink `Marks<D>` backing from `[D]Int` to `[D]I32`** and demote `Marks<D>` from the core cheat sheet to optional `std.marks`, teaching `[D]Bool` + touched-list clear first so `10^7`-state problems like `ucrpc21g` stay at 10 MB instead of 80 MB.
   - **Upgrade interactive I/O ergonomics:** auto-flush `print()` whenever `ask()` is used (fixing mixed one-way commands in `dmopc20c6p6` and final answers in `cco01p4io`), give `@interactor` an explicit `sol: Interactor` (`sol.recv()`/`sol.send(...)`) channel, and add a cheat-sheet/lint warning for the `etsr23r1p3` Tropical `min=`-skips-`none` footgun.

Across all 20 benchmark problems, applying these trims reduces the suite from **963 LOC / 26,882 typed keystrokes / 6,060 punct** to **962 LOC / 26,859 typed keystrokes / 6,054 punct** (or **968 LOC / 26,950 keystrokes** when `ucrpc21g` and `noi21p3` use `[D]Bool` + touched-list instead of `Marks<D>`), while removing 1 reserved keyword, 2 decorator-parameter grammars, 1 hazardous stdlib method, and 3 concrete contest-time footguns (80 MB `Marks` RAM cliff, `List.absorb` O(n^2) trap, and `@interactor` channel-direction deadlock).

---

## Verification Scope & Untested Boundaries

- **Verified mechanically:** All 20 benchmark file paths, raw C++ LOC counts from `corpus/submissions/`, corpus-wide pattern counts across all 611 canonical ideal ports at HEAD (`d6b3c4c`), and every LOC, raw-character, typed-keystroke, punctuation, `?`, `::`, `:`, bracket, and `@!?` delta across all 5 tensions and sub-variants were computed by a deterministic Python script over the exact source files in `proto/ports/`.
- **Not executed/compiled:** None of the canonical v2.1 or competing-alternative rewrites were compiled or run through `proto/cpc/` or `crates/`, because several v2.1 constructs under evaluation (Tropical Optionals, `for mut`, `Marks<(Cell, 0..=coins)>`, streaming `@interactor`) are specification-stage designs not yet implemented in the Rust compiler or Python prototype. Per-file `.strip` library-stripped C++ counts exist for only 1/20 of these scout ports and were therefore left unreported rather than estimated.
