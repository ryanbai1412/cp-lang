# Blind Full-Contest Simulation #1 — CP Lang 0.2.2

**Date:** 2026-11-20
**Compiler tested:** `cpl 0.2.2` (`cargo build --release -p cpl-cli` at commit `1634433`), verified with `cpl check`, `cpl run`, `cpl build` + `g++ -std=c++17 -O2 -fsanitize=address,undefined`, and `cpl stress`.
**Strict blind rules:** Only `docs/one-page-cheat-sheet.md` was consulted for language reference. No compiler source, stdlib source, prototype code, or existing C++ solutions were read. All six problems were fetched live from `atcoder.jp` (`codeforces.com/api/contest.list` succeeded, while `codeforces.com/contest/.../problem/...` returned HTTP 403 Cloudflare; `atcoder.jp` served full English statements and samples cleanly) and solved from scratch under contest pressure.

## Executive Summary

Six fresh 2026 AtCoder problems spanning easy implementation through ABC-G DP and ARC-B constructive math were solved from scratch in CP Lang 0.2.2:

1. **ABC484 C — Move One Card** (Easy implementation / sequence) — **0 first-draft errors**, 19-line solution, strictly cleaner than C++.
2. **ABC483 D — Glitchy Robot** (Medium-easy greedy / prefix-suffix) — **0 first-draft errors**, 27-line solution, roughly tied with C++ (slightly cleaner).
3. **ABC482 E — Diameter Endpoint Colors** (Medium-hard tree / diameter / center counting) — **19 cumulative errors across 3 failing drafts**, 63-line final solution, strictly slower to write and debug than C++ because of severe `cpl check` vs `cpl run`/`cpl build` divergence and undocumented `Graph`/`Rooted` traps.
4. **ABC484 F — Make them monochrome** (Hard data structures / custom-monoid `SegTree` + payload `enum`) — **15 cumulative errors across 2 failing drafts**, 36-line final solution, strictly cleaner than C++ once working (custom monoid `SegTree` + payload `enum` is a genuine joy).
5. **ABC484 G — Wildcard Construction** (Hard NFA-to-DFA subset-construction DP + `Str` + `Mod<998244353>` + bitmasks) — **0 first-draft errors**, 78-line solution, strictly cleaner than C++.
6. **ARC231 B — Three Mex** (Hard ARC math / bitwise XOR constructive + `@multitest` + `@checker`) — **0 first-draft errors**, 25-line solution, strictly cleaner than C++ (though `@checker` and `@multitest` stress generation both broke).

**Overall verdict:** When you stay on the happy path (`Str`, `[0..n]T`, `Mod<P>`, `SegTree`, `@multitest`, slice equality, `min=`/`max=`, nested `fn` capturing outer `var`, list comprehensions), CP Lang 0.2.2 already feels **dramatically faster to type and cleaner to read than C++17** — four of the six problems (including ABC-G and ARC-B!) compiled on the very first try, passed ASan/UBSan and brute-force stress tests immediately, and ran in 2–172 ms at maximum judge constraints with 4–23 KB of clean C++17.

However, the moment a contestant touches **`Graph`, `Rooted`, `max_by`, `[1..=n]T`, single-line `enum`, tuple-of-enum `match`, `read(none: ...)`, or in-file `@brute`/`@checker` with `cpl stress`**, they hit a brutal **false-green compiler gap**: `cpl check` reports `no errors`, and then `cpl run` and `cpl build` explode with up to 16 errors — several pointed at line `1:1` with leaked stdlib-internal type names. In a live 100-minute contest, that false-green gap and the broken in-file `@brute`/`@checker`/`@multitest` `cpl stress` pipeline are the biggest threats to contest speed.

---

## Top 10 Contest-Speed Bottlenecks (Ranked)

### 1. `cpl check` silently accepts programs that fail in `cpl run` and `cpl build` (P0)
On `ABC482 E`, `cpl check` reported `no errors` on three consecutive drafts that then failed `cpl run` and `cpl build` with 1, 3, and 16 errors respectively. The same false-green happened on `read(none: "-1")` (`probe_read_none.cp`). Because LSP squiggles come from the same frontend pass as `cpl check`, a contestant finishes writing a 60-line tree solution thinking it is green, runs sample 1, and gets ambushed by a wall of type errors during lowering.

**Fix:** Unify `cpl check` with the exact typed-IR lowering checks that `cpl run` and `cpl build` run, so `cpl check` never reports green on anything `cpl run` or `cpl build` will reject.

### 2. In-file `@brute` and `@checker` do not work in `cpl stress`, and `read_output()` is missing (P0)
`docs/one-page-cheat-sheet.md` promises `@brute(main) fn slow(): ...` and `@checker fn check(): ...` using `read_output()` as the default local stress workflow. In reality, `cpl check` accepts `@brute(main)` and `cpl build` strips it cleanly, yet `cpl stress` completely ignores it (`error: there is nothing to compare ... with: pass --brute <program>`). Meanwhile `@checker` fails `cpl check` with `error[E0201]: no read_output in scope`, and `cpl stress` reports `error: ... declares the checker 'check' (line 28), which cpl can't build yet; pass it built with --checker <program>`. One pleasant surprise: `--brute foo.cp` *does* accept a separate `.cp` file even though `cpl stress --help` only mentions `.cpp`, `.py`, or an executable.

**Fix:** Wire in-file `@brute` and `@checker` (with `read_output()`) directly into `cpl stress`, and document `.cp` support in `cpl stress --help`.

### 3. `cpl stress` type-driven input generator ignores `@multitest` (and emitted C++ reads past EOF as `0`) (P0)
Running `cpl stress p6_arc231_b.cp --checker p6_checker.py` on an `@multitest` file generated only 3 integers (`1 0 0`, labeled `a = 1  b = 0  c = 0`) with no leading test count `T`. The solution consumed `t = 1`, then read `a = 0, b = 0`, hit EOF on `c` (silently receiving `0`!), and printed `Yes / 0 / 0`, triggering a fake WA on case 1. Passing a `--gen` script that prints `1` before `a b c` immediately passed 200/200 cases.

**Fix:** Make the `cpl stress` generator prepend `T` (e.g., `1` or `1..=size`) whenever a file carries `@multitest`, and make local/debug `read()` trap on unexpected EOF instead of returning `0`.

### 4. `Graph.undirected(es)` and `g[u]` have undocumented shape traps and leak stdlib internals at line `1:1` (P0)
Writing the natural `let es: [0..(n - 1)](V, V) = read(); let g = Graph.undirected(es)` passes `cpl check`, then explodes in `cpl run`/`cpl build` with errors anchored at line `1:1` (`cpl run doesn't know the name E here` and `expected List<(to: V, e: Int)>, found List<(to: V, e: _)>`) plus `expected [Int](u: V, v: V), found [Int](V, V)`. The caller must declare a named `domain E = 0..(n - 1)` AND use labeled edge fields `(u: V, v: V)`. Even worse, iterating `for v in g[u]:` on an unweighted `Graph<V>` yields `(to: V, e: E)` instead of bare `V` (`E0302: can't compare a (to: V, e: E) with a V`), directly contradicting line 117 of the cheat sheet (`for v in g[u]: if not vis[v]: dfs(v)`).

**Fix:** Accept both plain `(V, V)` tuples and anonymous `[0..m]` arrays in `Graph.undirected`/`Graph.directed`, make `g[u]` on unweighted `Graph<V>` yield `V` directly (with `g.edges(u)` for `(to, e)`), and never anchor stdlib instantiation errors at user line `1:1`.

### 5. `Rooted<V>.parent(u)` loses its domain type (`?_`) in `cpl run`/`cpl build` (P0)
Even after fixing `Graph.undirected`, `var cur: ?V = V.first; cur = t1.parent(V.first)` passed `cpl check` and failed `cpl run`/`cpl build` with `error[E0302]: cur is a ?V, but t1.parent(V.first) is a ?_`. This blocked standard parent-pointer tree path reconstruction on `ABC482 E` and forced a workaround using `t2.ancestor(t, r)!`.

**Fix:** Preserve concrete domain substitution on `Rooted<V>.parent(u) -> ?V` through `cpl-ir` lowering, and add a differential test for `Rooted<V>` parent walking.

### 6. `max_by(V, ...)` fails to iterate over a `domain` in `cpl run`/`cpl build` (P1)
After adding `use std.seq.{max_by}`, `let s = max_by(V, v => t1.depth(v))!` passed `cpl check` and failed `cpl run`/`cpl build` with `error[E0302]: a for can't walk over a V` pointed at line `1:1`, cascading into `expected V, found Int` on every downstream use of `s`. On `ABC482 E`, this forced writing two boilerplate 4-line argmax loops by hand.

**Fix:** Lower generic iterable parameters in `std.seq` (`max_by`, `min_by`, etc.) so named `domain` values satisfy `Iterable<V>` identically to `for v in V:`.

### 7. Inconsistent prelude vs `use std.*` boundary contradicts the cheat sheet (P1)
`Graph`, `Rooted`, `SegTree`, and `Mod` are auto-imported without `use`, while `Deque` (`use std.deque.{Deque}`), `max_by` (`use std.seq.{max_by}`), `scan` (`use std.seq.{scan}`), `Monoid` (`use std.algebra.{Monoid}`), and `bfs` (`use std.graph.{bfs}`) all fail with `E0201`/`E0204` — even though `docs/one-page-cheat-sheet.md` puts `max_by` and `scan` under "Prelude" and shows zero `use` lines for any of them. The compiler's `= help: add use std.xxx.{...}` message is great, but every missing import still costs a compile-edit-compile round trip in a contest.

**Fix:** Either put every item on the one-page cheat sheet in the default prelude (preferred for contest speed!) or mark the exact `use` path on the cheat sheet beside every non-prelude symbol.

### 8. Non-zero-based structural range arrays `[1..=n]T` fail with `E0501` in `cpl run`/`cpl build` (P1)
`var total: [1..=n]Int = 0` is legal syntax, passes `cpl check`, and mirrors the cheat sheet's `['a'..='z']Int` example, yet fails `cpl run`/`cpl build` with `error[E0501]: cpl run and cpl build can't handle array dimensions that don't start at 0 yet`. Under contest pressure, this forces either wasting slot `0` with `[0..=n]Int` (losing the 1-based lower-bound check) or inventing an extra `domain C = 1..=n`.

**Fix:** Support arbitrary lower bounds (`lo..hi`, `lo..=hi`, `'a'..='z'`) in structural range arrays in `cpl-ir`, `cpl-interp`, and `cpl-codegen`.

### 9. Cheat-sheet syntax & feature mirages: single-line `enum`, tuple-of-enum `match`, and `read(none: ...)` (P1)
Three features copied straight from `docs/one-page-cheat-sheet.md` failed:
- Single-line `enum Mono: Any | Val(v: Int) | Bad` (cheat sheet line 141) failed parsing with `E0104: expected an indented block after enum Mono`.
- Matching a pair of payload enums with `(Val(x), Val(y)) => ...` silently failed to bind `x` and `y` (`E0201: no x in scope`), forcing nested `match` expressions for custom `SegTree` monoid merges.
- `let a: [V]?Int = read(none: "-1")` (cheat sheet line 151) passed `cpl check` and failed `cpl run`/`cpl build` with `E0310: read has no argument named none`.

**Fix:** Implement single-line `|` enum syntax, recursive pattern bindings inside tuple patterns, and `read(none: ...)` — or remove them from the cheat sheet until shipped.

### 10. Underspecified cheat-sheet signatures (`bfs`, `scan`) and `domain` vs `Int` index-shift hesitation (P2)
`bfs` appears on the cheat sheet as a bare name with no hint that it takes `List<V>` (`bfs(g, [V.first])`) rather than `V` (`E0301: bfs takes a List<V> for from, but V.first is a V`). `scan(a, +)` has no signature showing whether it is inclusive (length `n`) or exclusive (length `n + 1`), causing hesitation that led me to write an explicit 2-line prefix loop instead. Finally, on index-shifting sequence problems (`ABC484 C`, `ABC483 D`), nominal `domain` indexing (`v + 1 -> ?V`) creates split-second friction around `a[lo + 1..=hi]` vs boundary syntax (`lo.r..=hi`), pushing contestants under time pressure to fall back to `[0..n]Int` with plain `Int` indices.

**Fix:** Add terse one-line signatures to the cheat sheet for `bfs(g, [src]) -> [V]?Int`, `scan(a, op)` (inclusive vs exclusive), and mixed/boundary slice ranges, and support `lo.r..=hi` or concise non-optional shifts when bounds are proven.

---

## Problem-by-Problem Contest Log

### Problem 1: ABC484 C — Move One Card (Easy Implementation / Sequence)
**URL:** `https://atcoder.jp/contests/abc484/tasks/abc484_c`
**Idea:** Strip the longest common prefix (`0..lo`) and longest common suffix (`hi+1..n`) of `A` and `B`. If `lo == n`, output `Yes`. Otherwise `A` can become `B` in one move iff either `A[lo]` moves right to `hi` (`A[lo] == B[hi]` and `A[lo+1..=hi] == B[lo..=hi-1]`) or `A[hi]` moves left to `lo` (`A[hi] == B[lo]` and `A[lo..=hi-1] == B[lo+1..=hi]`).

**(a) First-draft compile errors / type fights:**
None — **0 errors** across `cpl check`, `cpl run`, and `cpl build` + `g++ -std=c++17 -O2 -fsanitize=address,undefined` on the very first draft. Slice equality `a[lo + 1..=hi] == b[lo..=hi - 1]` worked immediately in both interpreter and emitted C++.

**(b) Moments of hesitation where C++ would have been faster:**
1. Deciding between `domain I = 0..n` and structural `[0..n]Int`: with a nominal domain `I`, `lo + 1` returns `?I`, and I was unsure whether mixed boundary/index ranges like `lo.r..=hi` compile. Under contest pressure I immediately dropped `domain` and used `[0..n]Int`.
2. Brief doubt about whether `a[lo + 1..=hi] == b[lo..=hi - 1]` compares slices by value (in C++ I'd write `std::equal` without checking). It worked on the first try and beat C++ decisively.
3. Writing an in-file `@brute(main) fn slow():` per the cheat sheet, only to discover `cpl stress` ignores it and forces extracting the brute into a separate file.

**(c) Discrepancies between `cpl check`, `cpl run`, `cpl build`, and `cpl stress`:**
- `cpl check`, `cpl run`, and `cpl build` agreed 100% (and `cpl build` properly stripped `@brute(main) fn slow()` from emitted C++).
- **Discrepancy:** `cpl stress` ignored both `@brute(main)` and `@brute`:
```text
error: there is nothing to compare /tmp/sim1/p1_abc484_c.cp with: pass `--brute <program>`, or a `--checker` or `--interactor` that can judge an output on its own
```
Extracting the brute to a separate file (`--brute p1_brute.py` and `--brute p1_brute.cp`) worked cleanly (500/500 `cpl stress` cases + 300 targeted small-alphabet Yes/No-balanced cases passed in both `cpl run` and ASan/UBSan `g++`).

**(d) Final CP Lang 0.2.2 solution & verdict:**
```cp
let n: 1..=200_000 = read()
let a: [0..n]Int = read()
let b: [0..n]Int = read()

var lo = 0
while lo < n and a[lo] == b[lo]:
    lo += 1

if lo == n:
    print("Yes")
    return

var hi = n - 1
while hi > lo and a[hi] == b[hi]:
    hi -= 1

let move_right = a[lo] == b[hi] and a[lo + 1..=hi] == b[lo..=hi - 1]
let move_left = a[hi] == b[lo] and a[lo..=hi - 1] == b[lo + 1..=hi]
print(if move_right or move_left: "Yes" else: "No")
```
**Verification:** 4/4 AtCoder samples pass (`cpl run` + ASan/UBSan); 500 `cpl stress` cases + 300 targeted Yes/No cases pass; max-N (`2e5`) runs in **21.0 ms** (`-O2`), emitted C++ is **4,902 bytes**.
**Cleaner than C++?** **Strictly cleaner.** Zero `std::equal` iterator noise, concise `read()`, top-level `return`, and inline `if ...: ... else: ...`.

---

### Problem 2: ABC483 D — Glitchy Robot (Medium-Easy Greedy / Prefix-Suffix)
**URL:** `https://atcoder.jp/contests/abc483/tasks/abc483_d`
**Idea:** Because the robot moves continuously, the count of distinct integer coordinates visited is `max(P) - min(P) + 1`, where `P_0 = 0, P_k = A_1 + ... + A_k`. Flipping `A_i` shifts every `P_j` (`j >= i`) by `-2 * A_i`, so the new span is `max(pre_max, suf_max[i] - 2*A_i) - min(pre_min, suf_min[i] - 2*A_i) + 1`.

**(a) First-draft compile errors / type fights:**
None — **0 errors** across `cpl check`, `cpl run`, and `cpl build` + ASan/UBSan `g++` on the first draft.

**(b) Moments of hesitation where C++ would have been faster:**
1. Staring at `scan(a, +)` on line 201 of the cheat sheet: it gives no signature, so I couldn't tell if `scan(a, +)` produces length `n` (inclusive) or length `n + 1` (starting with `0`). I bypassed it and wrote a 2-line loop. A post-solve probe confirmed a second trap — `scan` isn't even in the prelude:
```text
error[E0201]: no `scan` in scope
  --> /tmp/sim1/probe_scan.cp:3:9
  |
3 | let s = scan(a, +)
  |         ^^^^
  = help: `scan` is in the standard library: add `use std.seq.{scan}`
```
2. Minor check on `(0..n).rev` precedence (parentheses needed so `.rev` binds to the range).

**(c) Discrepancies between `cpl check`, `cpl run`, `cpl build`, and `cpl stress`:**
None on the solution itself (`cpl check`, `cpl run`, `cpl build`, and `cpl stress --brute p2_brute.py` all agreed cleanly across 500 stress cases + 100 large `±1e9` cases).

**(d) Final CP Lang 0.2.2 solution & verdict:**
```cp
let n: 1..=200_000 = read()
let a: [0..n]Int = read()

var p: [0..=n]Int = 0
for i in 0..n:
    p[i + 1] = p[i] + a[i]

var suf_min: [0..=n]Int = 0
var suf_max: [0..=n]Int = 0
suf_min[n] = p[n]
suf_max[n] = p[n]
for i in (0..n).rev:
    suf_min[i] = min(p[i], suf_min[i + 1])
    suf_max[i] = max(p[i], suf_max[i + 1])

var ans = suf_max[0] - suf_min[0] + 1
var pre_min = 0
var pre_max = 0
for i in 0..n:
    let shift = -2 * a[i]
    let lo = min(pre_min, suf_min[i + 1] + shift)
    let hi = max(pre_max, suf_max[i + 1] + shift)
    ans max= hi - lo + 1
    pre_min min= p[i + 1]
    pre_max max= p[i + 1]

print(ans)
```
**Verification:** 2/2 AtCoder samples pass; 500 `cpl stress` cases + 100 `±1e9` overflow cases pass (`cpl run` + ASan/UBSan); max-N (`2e5`) runs in **15.2 ms**, emitted C++ is **4,110 bytes**.
**Cleaner than C++?** **Slightly cleaner** (no `long long`, `(0..n).rev`, `ans max= ...`, `pre_min min= ...`), roughly the same typing speed.

---

### Problem 3: ABC482 E — Diameter Endpoint Colors (Medium-Hard Tree / Diameter / Center)
**URL:** `https://atcoder.jp/contests/abc482/tasks/abc482_e`
**Idea:** Find a tree diameter `(s, t)` of length `D` via two rootings. If `D = 2r` is even, every diameter path passes through the unique center vertex `center = ancestor(t, r)` and joins two vertices at depth `r` in *different* subtrees of `center`; count cross-branch same-color pairs incrementally in `O(N)`. If `D = 2r + 1` is odd, every diameter path crosses the central edge `(c1, c2)` and joins a depth-`r` vertex on `c1`'s side with a same-color depth-`r` vertex on `c2`'s side.

**(a) First-draft compile errors / type fights:**
This problem triggered the biggest compiler fight of the entire simulation — **four drafts** to reach a clean build:

*Draft 1* wrote `let es: [0..(n - 1)](V, V) = read()`, `max_by(V, v => t1.depth(v))!`, `var cur: ?V = t; while let u = cur: path.push(u); cur = t2.parent(u)`, `var total: [1..=n]Int = 0`, `var q: Deque<(V, V, Int)> = []`, and `for v in g[u]:`.
`cpl check` caught only the missing imports:
```text
error[E0201]: no `max_by` in scope
  --> /tmp/sim1/p3_abc482_e.cp:9:9
  = help: `max_by` is in the standard library: add `use std.seq.{max_by}`
error[E0204]: no type or domain named `Deque`
  --> /tmp/sim1/p3_abc482_e.cp:25:12
  = help: `Deque` is in the standard library: add `use std.deque.{Deque}`
```

*Draft 2* added `use std.seq.{max_by}` and `use std.deque.{Deque}`. Now **`cpl check` passed with 0 errors**, yet `cpl run` and `cpl build` rejected it:
```text
error[E0302]: `cur` is a ?V, but `t2.parent(u)` is a ?_
  --> /tmp/sim1/p3_abc482_e.cp:22:11
   |
22 |     cur = t2.parent(u)
   |           ^^^^^^^^^^^^
```
Isolating the failure with a 6-line probe (`probe_rooted.cp`) revealed that `Graph.undirected(es)` on `[0..(n - 1)](V, V)` passes `cpl check` and explodes in `cpl run`/`cpl build` with line `1:1` stdlib leaks:
```text
error[E0201]: `cpl run` doesn't know the name `E` here
  --> /tmp/sim1/probe_rooted.cp:1:1
  |
1 | let n: 2..=10 = read()
  | ^
error[E0302]: expected `List<(to: V, e: Int)>`, found `List<(to: V, e: _)>`
  --> /tmp/sim1/probe_rooted.cp:1:1
error[E0302]: expected `[Int](u: V, v: V)`, found `[Int](V, V)`
  --> /tmp/sim1/probe_rooted.cp:4:26
  |
4 | let g = Graph.undirected(es)
  |                          ^^
```
Fixing `es` to `domain E = 0..(n - 1); let es: [E](u: V, v: V) = read()` fixed `Rooted<V>(g, V.first)` — yet a minimal probe (`probe_parent.cp`) proved `t1.parent(V.first)` STILL returns `?_` in `cpl run`/`cpl build` (while passing `cpl check`!):
```text
error[E0302]: `cur` is a ?V, but `t1.parent(V.first)` is a ?_
  --> /tmp/sim1/probe_parent.cp:8:7
  |
8 | cur = t1.parent(V.first)
  |       ^^^^^^^^^^^^^^^^^^
```

*Draft 3* ditched `.parent(u)` in favor of `t2.ancestor(t, r)!`. Once again **`cpl check` passed with 0 errors**, and `cpl run`/`cpl build` erupted with **16 errors** exposing three more independent bugs:
```text
error[E0302]: a `for` can't walk over a `V`
  --> /tmp/sim1/p3_abc482_e.cp:1:1
error[E0302]: expected `V`, found `Int`
  --> /tmp/sim1/p3_abc482_e.cp:13:33
   |
13 | let s = max_by(V, v => t1.depth(v))!
error[E0501]: `cpl run` and `cpl build` can't handle array dimensions that don't start at 0 yet
  --> /tmp/sim1/p3_abc482_e.cp:20:13
   |
20 | var total: [1..=n]Int = 0
error[E0302]: can't compare a `(to: V, e: E)` with a `V`
  --> /tmp/sim1/p3_abc482_e.cp:30:16
   |
30 |             if v != p:
```

*Draft 4* replaced `max_by(V, ...)` with manual argmax loops, changed `[1..=n]Int` to `[0..=n]Int`, and unpacked `for edge in g[u]: let v = edge.to`. It compiled and passed on the first run.

**(b) Moments of hesitation where C++ would have been faster:**
1. Wanted to use `bfs(g, V.first)` for the two diameter passes, but the cheat sheet gives no signature for `bfs`. A post-solve probe confirmed two hurdles: it needs `use std.graph.{bfs}`, and `bfs(g, V.first)` fails with `error[E0301]: bfs takes a List<V> for from, but V.first is a V` (you must pass `[V.first]`).
2. `Graph.undirected(es)` requiring both a dummy `domain E` and named record fields `(u: V, v: V)` — zero clue from the cheat sheet.
3. Unweighted `g[u]` yielding `(to: V, e: E)` instead of `V` — directly contradicts cheat sheet line 117 (`for v in g[u]: if not vis[v]: dfs(v)`).
4. Debugging `t2.parent(u)` -> `?_` and `max_by(V, ...)` -> `a for can't walk over a V` at line `1:1` would have burned 10+ minutes in a live contest; in C++ I would have typed `vector<vector<int>> adj(n+1)` and two BFS functions in 90 seconds.

**(c) Discrepancies between `cpl check`, `cpl run`, and `cpl build`:**
Massive: `cpl check` returned `no errors` on Draft 2, Draft 3, `probe_rooted.cp`, and `probe_parent.cp`, while `cpl run` and `cpl build` failed on every single one of them (16 errors on Draft 3!). Once Draft 4 built, `cpl run` and `cpl build` + ASan/UBSan matched on all samples and random trees.

**(d) Final CP Lang 0.2.2 solution & verdict:**
```cp
use std.deque.{Deque}

let n: 2..=200_000 = read()
domain V = 1..=n
domain E = 0..(n - 1)
let es: [E](u: V, v: V) = read()
let col: [V](1..=n) = read()

let g = Graph.undirected(es)

let t1 = Rooted<V>(g, V.first)
var s = V.first
for v in V:
    if t1.depth(v) > t1.depth(s):
        s = v

let t2 = Rooted<V>(g, s)
var t = s
for v in V:
    if t2.depth(v) > t2.depth(t):
        t = v
let diam = t2.depth(t)

var ans = 0
var total: [0..=n]Int = 0

fn collect_at_depth(start: V, blocked: V, target: Int, mut out: List<Int>):
    var q: Deque<(V, V, Int)> = []
    q.push((start, blocked, 0))
    while let (u, p, d) = q.pop_front():
        if d == target:
            out.push(col[u])
            continue
        for edge in g[u]:
            let v = edge.to
            if v != p:
                q.push((v, u, d + 1))

if diam % 2 == 0:
    let r = diam / 2
    let center = t2.ancestor(t, r)!
    for edge in g[center]:
        let ch = edge.to
        var cols: List<Int> = []
        collect_at_depth(ch, center, r - 1, mut cols)
        for c in cols:
            ans += total[c]
        for c in cols:
            total[c] += 1
else:
    let r = (diam - 1) / 2
    let c1 = t2.ancestor(t, r)!
    let c2 = t2.ancestor(t, r + 1)!
    var left_cols: List<Int> = []
    var right_cols: List<Int> = []
    collect_at_depth(c1, c2, r, mut left_cols)
    collect_at_depth(c2, c1, r, mut right_cols)
    for c in left_cols:
        total[c] += 1
    for c in right_cols:
        ans += total[c]

print(ans)
```
**Verification:** 3/3 AtCoder samples pass; 250 random trees (paths, stars, double brooms, random trees; 30 in `cpl run`, 250 in ASan/UBSan `g++`) match an `O(N^2)` all-pairs BFS oracle; max-N (`2e5`-node path, confirming `Rooted` is iterative and stack-safe) runs in **171.9 ms**, emitted C++ is **22,993 bytes**.
**Cleaner than C++?** **Final code is slightly cleaner** (`Rooted<V>` + `.ancestor(t, r)!` + `while let (u, p, d) = q.pop_front():`), **but getting there was strictly slower than C++** because of the five compiler/stdlib traps above.

---

### Problem 4: ABC484 F — Make them monochrome (Hard Data Structures — Custom-Monoid `SegTree`)
**URL:** `https://atcoder.jp/contests/abc484/tasks/abc484_f`
**Idea:** A subarray `A[l..=r]` can be made monochrome by replacing `-1` wildcards iff all non-`-1` values in `A[l..=r]` are equal. Model each segment with a 3-state monoid `enum Mono` (`Any` identity, `Val(v)`, `Bad`), store it in `SegTree<V, Mono>`, and answer point updates and range folds in `O(log N)`.

**(a) First-draft compile errors / type fights:**
*Draft 1* copied the single-line payload enum syntax from cheat sheet line 141 (`enum Mono: Any | Val(v: Int) | Bad`), used `Monoid(Any, combine)` without `use`, and matched `(a, b)` with `(Val(x), Val(y)) => ...`. `cpl check` returned 12 errors:
```text
error[E0104]: expected an indented block after `enum Mono`
  --> /tmp/sim1/p4_abc484_f.cp:1:11
  |
1 | enum Mono: Any | Val(v: Int) | Bad
  |           ^
  = help: indent the next line by 4 spaces
...
error[E0201]: no `Monoid` in scope
  --> /tmp/sim1/p4_abc484_f.cp:18:27
  |
18 | var st = SegTree<V, Mono>(Monoid(Any, combine), init)
   |                           ^^^^^^
   = help: `Monoid` is in the standard library: add `use std.algebra.{Monoid}`
```

*Draft 2* indented the `enum` variants onto separate lines and added `use std.algebra.{Monoid}`. Now `match (a, b):` failed across `cpl check`, `cpl run`, and `cpl build`:
```text
error[E0201]: no `x` in scope
  --> /tmp/sim1/p4_abc484_f.cp:15:32
   |
15 |         (Val(x), Val(y)) => if x == y: Val(x) else: Bad
   |                                ^
```
Nested enum-variant patterns inside a tuple pattern (`(Val(x), Val(y))`) do not bind `x` or `y`, even though flat tuple patterns `(Any, x)` bound `x`.

*Draft 3* rewrote `combine` with a nested `match a:` / `match b:`, and immediately passed `cpl check`, `cpl run`, and `cpl build` + ASan/UBSan.

**(b) Moments of hesitation where C++ would have been faster:**
1. Wondering whether to use `let a: [V]?Int = read(none: "-1")` (cheat sheet line 151) vs reading `[V]Int`. I read `[V]Int` to stay uniform with type-1 update queries (`1 i x` where `x` can also be `-1`). A post-solve probe showed `read(none: "-1")` would have been a trap anyway — it passes `cpl check` and fails `cpl run`/`cpl build` with `error[E0310]: read has no argument named none`.
2. Checking whether `SegTree` and `Monoid` need `use` statements (`SegTree` is in the prelude; `Monoid` is not).
3. Rewriting the clean 4-arm `match (a, b):` into a 7-line nested `match` because nested tuple-variant bindings broke.

**(c) Discrepancies between `cpl check`, `cpl run`, and `cpl build`:**
- On Draft 1, `cpl check` reported 12 errors (including `no Monoid in scope`), whereas `cpl run` and `cpl build` reported 11 errors (omitting `no Monoid in scope`).
- On the `read(none: "-1")` probe, `cpl check` reported `no errors` while `cpl run` and `cpl build` rejected it with `E0310`.
- On Draft 3, all three agreed cleanly.

**(d) Final CP Lang 0.2.2 solution & verdict:**
```cp
use std.algebra.{Monoid}

enum Mono:
    Any
    Val(v: Int)
    Bad

fn to_mono(x: Int) -> Mono:
    if x == -1: Any else: Val(x)

fn combine(a: Mono, b: Mono) -> Mono:
    match a:
        Any => b
        Bad => Bad
        Val(x) =>
            match b:
                Any => Val(x)
                Bad => Bad
                Val(y) => if x == y: Val(x) else: Bad

let (n, q): (1..=300_000, 1..=300_000) = read()
domain V = 1..=n
let a: [V]Int = read()

let init: [V]Mono = [V] i => to_mono(a[i])
var st = SegTree<V, Mono>(Monoid(Any, combine), init)

for _ in 0..q:
    let tp: Int = read()
    if tp == 1:
        let (i, x): (V, Int) = read()
        st.set(i, to_mono(x))
    else:
        let (l, r): (V, V) = read()
        let res = st.fold(l..=r)
        print(if res == Bad: "No" else: "Yes")
```
**Verification:** Sample passes; 250 random query sequences (30 in `cpl run`, 250 in ASan/UBSan `g++`) match a brute-force oracle; max-N (`N = 3e5, Q = 3e5`) runs in **151.6 ms**, emitted C++ is **11,847 bytes**.
**Cleaner than C++?** **Strictly cleaner — a showcase win.** A 36-line ABC-F with a tagged-union monoid, 1-indexed domain `V`, auto-derived `Eq` on `Mono` (`res == Bad`), and zero segment-tree boilerplate beats C++ hands down.

---

### Problem 5: ABC484 G — Wildcard Construction (Hard DP + Automaton / Bitmasks + `Str` + `Mod<998244353>`)
**URL:** `https://atcoder.jp/contests/abc484/tasks/abc484_g`
**Idea:** Count distinct binary strings matching `S` (`|S| <= 2000`) that can be tiled by `X` (`|X| <= 8`) and `Y` (`|Y| <= 8`) with no two adjacent `Y`s. Build the `(|X| + |Y|)`-state NFA where bit `0` means "ready to start `X`" (also the exact accepting condition after `|S| >= 1` chars!), bits `1..|X|-1` track progress inside `X`, bit `|X|` means "ready to start `Y`" (only set at start and after finishing `X`), and bits `|X|+1..|X|+|Y|-1` track progress inside `Y`. Run BFS subset construction over reachable bitmasks (`<= 65536`), then run `O(|S| * |DFA|)` DP modulo `998244353`.

**(a) First-draft compile errors / type fights:**
None — **0 errors** across `cpl check`, `cpl run`, and `cpl build` + ASan/UBSan `g++` on the very first draft! `Mod<998_244_353>`, `Str` indexing (`x[j] == '?'`), bitshifts (`1 << (lx + ly)` without 32-bit UB!), `[0..total_masks]?Int = none`, `if let id = id_of[m]:`, nested `fn get_id` mutating outer `var`s, `ndp.fill(0)`, `swap(mut dp, mut ndp)`, and `cur != 0` on `Mod` all worked seamlessly on attempt 1.

**(b) Moments of hesitation where C++ would have been faster:**
1. Wondering whether `Mod<MOD>` needed `use std.mod.{Mod}` (unlike `Monoid`/`Deque`/`max_by`, it is in the prelude!) and whether `var dp: [0..num_states]Z = 0`, `dp[start_id] = 1`, and `cur != 0` accept bare `Int` literals without writing `Z(0)`/`Z(1)`. They all worked cleanly.
2. Mild caution around bitwise precedence in `((mask >> j) & 1) != 0` and `(masks[st] & 1) != 0`, so I added explicit parentheses just as in C++.
3. Minor hesitation about whether `swap(mut dp, mut ndp)` works on anonymous `[0..num_states]Z` arrays (it did, in O(1)!).

**(c) Discrepancies between `cpl check`, `cpl run`, and `cpl build`:**
None. All three agreed on every sample and all 150 randomized brute-force stress cases.

**(d) Final CP Lang 0.2.2 solution & verdict:**
```cp
const MOD = 998_244_353
type Z = Mod<MOD>

let s: Str = read()
let x: Str = read()
let y: Str = read()

let lx = x.len
let ly = y.len
let total_masks = 1 << (lx + ly)

fn step_mask(mask: Int, ch: Char) -> Int:
    var nm = 0
    for j in 0..lx:
        if ((mask >> j) & 1) != 0 and (x[j] == '?' or x[j] == ch):
            if j + 1 < lx:
                nm |= 1 << (j + 1)
            else:
                nm |= 1 | (1 << lx)
    for j in 0..ly:
        if ((mask >> (lx + j)) & 1) != 0 and (y[j] == '?' or y[j] == ch):
            if j + 1 < ly:
                nm |= 1 << (lx + j + 1)
            else:
                nm |= 1
    nm

var id_of: [0..total_masks]?Int = none
var masks: List<Int> = []
var nxt0: List<Int> = []
var nxt1: List<Int> = []

fn get_id(m: Int) -> Int:
    if let id = id_of[m]:
        return id
    let id = masks.len
    id_of[m] = id
    masks.push(m)
    nxt0.push(0)
    nxt1.push(0)
    id

let start_mask = 1 | (1 << lx)
let start_id = get_id(start_mask)

var head = 0
while head < masks.len:
    let m = masks[head]
    let m0 = step_mask(m, '0')
    let m1 = step_mask(m, '1')
    nxt0[head] = get_id(m0)
    nxt1[head] = get_id(m1)
    head += 1

let num_states = masks.len
var dp: [0..num_states]Z = 0
var ndp: [0..num_states]Z = 0
dp[start_id] = 1

for i in 0..s.len:
    ndp.fill(0)
    let allow0 = s[i] != '1'
    let allow1 = s[i] != '0'
    for st in 0..num_states:
        let cur = dp[st]
        if cur != 0:
            if allow0:
                ndp[nxt0[st]] += cur
            if allow1:
                ndp[nxt1[st]] += cur
    swap(mut dp, mut ndp)

var ans: Z = 0
for st in 0..num_states:
    if (masks[st] & 1) != 0:
        ans += dp[st]

print(ans)
```
**Verification:** 3/3 AtCoder samples pass (including the 63-char sample returning `992356689`); 150 random brute-force cases (25 in `cpl run`, 150 in ASan/UBSan `g++`) pass; max-N (`|S| = 2000, |X| = 8, |Y| = 8`) runs in **2.3 ms**, emitted C++ is **11,567 bytes**.
**Cleaner than C++?** **Strictly cleaner.** Zero modint boilerplate, clean nested `fn get_id` with no lambda capture list, niche-packed `?Int` instead of `-1` sentinel, and 2.3 ms runtime on max constraints.

---

### Problem 6: ARC231 B — Three Mex (Hard ARC Math / XOR Constructive + `@multitest` + `@checker`)
**URL:** `https://atcoder.jp/contests/arc231/tasks/arc231_b`
**Idea:** Necessary condition: since `0..A ⊆ X` and `0..B ⊆ Y`, there must be no `x in 0..A` with `(x ^ C) < B` (otherwise `C ∈ X ^ Y`). This condition is also sufficient: let `M = 1024 > max(A, B, C)` (so `M + 999 = 2023 <= 2026`). Set `X = {0..A-1}` and `Y = {0..B-1}`; if `C > 0`, add `{M ^ z | z in 0..C}` to `X` and `M` to `Y`. Every `z in 0..C` is realized by `(M ^ z) ^ M = z`, while any cross-term with bit `1024` is `>= 1024 > C`, so `mex(X) = A`, `mex(Y) = B`, and `mex(X ^ Y) = C`.

**(a) First-draft compile errors / type fights:**
None on the solution itself — **0 errors** across `cpl check`, `cpl run`, and `cpl build` + ASan/UBSan `g++` on the first draft (`@multitest`, top-level `return`, `any((x ^ c) < b for x in 0..a)`, list comprehensions, and `print(xs.len, xs)` all worked cleanly).

However, adding an in-file `@checker fn check():` per cheat sheet line 167 failed immediately:
```text
error[E0201]: no `read_output` in scope
  --> /tmp/sim1/p6_with_checker.cp:29:24
   |
29 |     let verdict: Str = read_output()
   |                        ^^^^^^^^^^^
```
And `cpl stress` on that file returned:
```text
error: /tmp/sim1/p6_with_checker.cp declares the checker `check` (line 28), which cpl can't build yet; pass it built with `--checker <program>`
```

**(b) Moments of hesitation where C++ would have been faster:**
1. Formatting `0` vs `N X_1 ... X_N` cleanly: I hesitated on whether `print(xs.len, xs)` prints an extra trailing space when `xs` is empty, so I guarded it with `if xs.is_empty: print(0) else: print(xs.len, xs)`.
2. Attempting to use in-file `@checker` + `cpl stress` (which cost two dead ends: `read_output()` missing, then `cpl stress`'s `@multitest` generator bug).

**(c) Discrepancies between `cpl check`, `cpl run`, `cpl build`, and `cpl stress`:**
- `cpl check`, `cpl run`, and `cpl build` agreed 100% on the solution.
- **Critical `cpl stress` bug:** Running `cpl stress /tmp/sim1/p6_arc231_b.cp --checker /tmp/sim1/p6_checker.py` failed immediately on seed 1 because the automatic generator forgot to emit `T` for `@multitest`:
```text
  FAIL on case 1 (seed 1): WA
  shrunk from 3 numbers to 3:
    1 0 0
  that is: a = 1  b = 0  c = 0
  got:
    Yes
    0
    0
```
The compiled binary read `t = 1`, consumed `0 0` for `a, b`, silently read `0` on EOF for `c`, and printed `Yes / 0 / 0`. Supplying `--gen` emitting `1` before `a b c` passed 200/200 `cpl stress` cases cleanly.

**(d) Final CP Lang 0.2.2 solution & verdict:**
```cp
@multitest
let (a, b, c): (0..=1000, 0..=1000, 0..=1000) = read()

if any((x ^ c) < b for x in 0..a):
    print("No")
    return

var xs: List<Int> = [x for x in 0..a]
var ys: List<Int> = [y for y in 0..b]

if c > 0:
    let m = 1024
    for z in 0..c:
        xs.push(m ^ z)
    ys.push(m)

print("Yes")
if xs.is_empty:
    print(0)
else:
    print(xs.len, xs)
if ys.is_empty:
    print(0)
else:
    print(ys.len, ys)
```
**Verification:** Sample passes; exhaustive verification of all `9^3 = 729` triples in `0..=8` plus 107 random/boundary triples up to `1000` (836 total) passes full MEX/XOR/bounds validation in both `cpl run` and ASan/UBSan `g++`; 200/200 `cpl stress --gen --checker` cases pass; max-N (`T = 10, A = C = 1000`) runs in **3.2 ms**, emitted C++ is **4,388 bytes**.
**Cleaner than C++?** **Strictly cleaner** — 25 lines total, `@multitest` + top-level `return` + `any(...)` + `print(xs.len, xs)` reads like executable math.

---

## Closing Takeaway

CP Lang 0.2.2 already beats C++17 on clarity, brevity, and zero-UB safety across sequence, greedy, custom-monoid segment tree, hard automaton/modular DP, and multitest constructive math — four of six blind contest problems compiled cleanly on the very first draft with zero borrow-checker fights and 2–172 ms max-N runtimes. Closing the **false-green `cpl check` vs `cpl run`/`cpl build` gap**, fixing **`Graph`/`Rooted`/`max_by`/`[1..=n]T`**, aligning the **prelude with the one-page cheat sheet**, and shipping **in-file `@brute`/`@checker` + `@multitest` support in `cpl stress`** will eliminate virtually every contest-speed bottleneck surfaced in this simulation.
