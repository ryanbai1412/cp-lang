# Scout Batch 52: Friction Backlog & Dual LOC Report (15 Hard Olympiad Problems in Locked CP Lang v2.2)

## Overview

Scout Batch 52 ported 15 previously unported Olympiad problems (6 IOI, 6 NOI, 1 CTSC, 1 NOIP, 1 IATI) into the locked 34-keyword CP Lang v2.2 surface under `proto/ports/scout_batch_52/`, paired with runnable `today/` ports compiled through the real Rust `cpl build` backend and verified differentially under ASan/UBSan against the original C++17 submissions (`proto/ports/scout_batch_52/test_batch_52.py`).

Every ideal v2.2 port passes the real Rust frontend typechecker (`cpl check`: **15/15**). None of the 15 ideal ports yet lowers cleanly through `cpl build` (**0/15**), exposing a crisp set of compiler/backend gaps. All 15 runnable `today/` ports compile cleanly with `cpl build` -> C++17 -> `g++ -O2 -fsanitize=address,undefined` and match their original C++ submissions (or validate constructively, for `noip20p3` and interactive `noi02p2`) across **750/750** random and edge test cases (50 per problem).

---

## Dual LOC Reductions (Raw C++ vs Library-Stripped C++ vs Ideal v2.2)

Across the 15 problems, non-blank/non-comment code shrinks from **2,531 raw C++ LOC** and **1,999 library-stripped C++ LOC** (excluding `#include`, `using`, `#define`, debug helpers, fast-I/O boilerplate, and reusable `mint`/`BIT`/`SegTree` templates) down to **871 ideal v2.2 LOC** — a **-65.6% raw LOC reduction** and **-56.4% library-stripped LOC reduction**. Even the hand-lowered `today/` ports (which spell out binary heaps, implicit treaps, and 2D Fenwick trees manually to work around `cpl build` gaps) weigh in at **1,492 LOC** (-41.1% raw, -25.4% stripped).

Per-problem breakdown (raw C++ -> stripped C++ -> ideal v2.2, plus `today/`):

- **IOI '14 P6 Holiday** (`ioi14p6io`, `4691224.cpp`): 134 raw / 109 stripped -> **67 v2.2** (-50.0% raw, -38.5% stripped); `today/` 93.
- **IOI '13 P3 Wombats** (`ioi13p3`, `4722414.cpp`): 184 raw / 144 stripped -> **66 v2.2** (-64.1% raw, -54.2% stripped); `today/` 96.
- **IOI '14 P1 Rail** (`ioi14p1`, `4691718.cpp`): 92 raw / 66 stripped -> **41 v2.2** (-55.4% raw, -37.9% stripped); `today/` 89.
- **IOI '22 P5 Insects** (`ioi22p5`, `4786085.cpp`): 91 raw / 66 stripped -> **40 v2.2** (-56.0% raw, -39.4% stripped); `today/` 72.
- **IOI '17 P4 The Big Prize** (`ioi17p4`, `5659145.cpp`): 86 raw / 62 stripped -> **37 v2.2** (-57.0% raw, -40.3% stripped); `today/` 49.
- **IOI '20 P5 Mushrooms** (`ioi20p5`, `4736004.cpp`): 210 raw / 172 stripped -> **85 v2.2** (-59.5% raw, -50.6% stripped); `today/` 166.
- **NOI '23 P4 Trade** (`noi23p4`, `5723063.cpp`): 168 raw / 98 stripped -> **58 v2.2** (-65.5% raw, -40.8% stripped); `today/` 124.
- **NOI '19 P3 Sequence** (`noi19p3`, `5669058.cpp`): 245 raw / 209 stripped -> **73 v2.2** (-70.2% raw, -65.1% stripped); `today/` 153.
- **NOI '21 P5 Code** (`noi21p5`, `3927923.cpp`): 233 raw / 198 stripped -> **102 v2.2** (-56.2% raw, -48.5% stripped); `today/` 141.
- **NOI '08 P5 Clouds** (`noi08p5`, `6067573.cpp`): 232 raw / 196 stripped -> **40 v2.2** (-82.8% raw, -79.6% stripped); `today/` 82.
- **NOI '02 P6 Robots** (`noi02p6`, `3754486.cpp`): 74 raw / 48 stripped -> **15 v2.2** (-79.7% raw, -68.8% stripped); `today/` 28.
- **NOI '02 P2 Ring Interactive** (`noi02p2`, `4793917.cpp`): 74 raw / 47 stripped -> **18 v2.2** (-75.7% raw, -61.7% stripped); `today/` 37.
- **CTSC '04 P4 Station** (`ctsc04p4`, `4529641.cpp`): 198 raw / 149 stripped -> **73 v2.2** (-63.1% raw, -51.0% stripped); `today/` 111.
- **NOIP '20 P3 Ball Move** (`noip20p3`, `5445674.cpp`): 216 raw / 175 stripped -> **71 v2.2** (-67.1% raw, -59.4% stripped); `today/` 109.
- **IATI '21 A13 Tree Activation** (`iati21a13`, `5052303.cpp`): 294 raw / 260 stripped -> **85 v2.2** (-71.1% raw, -67.3% stripped); `today/` 142.

Where the biggest wins came from:

- **NOI '08 P5 Clouds (-79.6% stripped)**: `Fenwick2D` + `Range.clamp` + Euclidean `%` collapsed 196 lines of reflected-coordinate case splits and custom 2D BITs into 40 lines.
- **IATI '21 A13 (-67.3% stripped)** and **NOI '19 P3 (-65.1% stripped)**: `SortedBag`, `SegTree`, `Rooted`, and `next_permutation` replaced hundreds of lines of custom PBDS wrappers, lazy-deletion priority queues, and manual Euler-tour arrays.
- **IOI '13 P3 Wombats (-54.2% stripped)**: Tropical `?Int` arithmetic (`out[i][k] min= out[i][k - 1] + h[row][k - 1]`) and `(0..self.c).rev` eliminated sentinel `INF` checks and reduced the Monge min-plus product to a single clean nested loop.

---

## Real Rust `cpl build` Audit & Runtime Findings

Running `cpl check` and `cpl build` directly on the 15 ideal v2.2 ports plus compiling and ASan-running all 15 `today/` ports exposed two distinct tiers of issues: **severe runtime/codegen bugs in `cpl build` today** and **remaining language/ergonomic friction points in v2.2**.

### 1. Critical `cpl build` Codegen Bug: Nested Recursive Functions Segfault Under ASan

When a nested `fn` inside another function calls itself recursively (or references another nested helper), `cpl build` lowers it to a C++ lambda stored in `std::function` or captures an uninitialized self-binding, causing an immediate **AddressSanitizer null-pointer/stack-use-after-scope segfault** at runtime. We hit this exact crash on 7 separate problems (`ioi14p6io`, `ioi13p3`, `ioi17p4`, `noi23p4`, `noi21p5`, `ctsc04p4`, `noip20p3`) before lifting every recursive helper to top-level functions with global `var` state in `today/`.

Concrete pattern that crashes in `cpl build` today:

```cp
fn solve():
    var g = ...
    fn dfs(u: Int, p: Int) -> Int:
        for v in g[u]:
            if v != p: let _ = dfs(v, u)  // ASan crash in emitted C++17
        return 0
    let _ = dfs(0, -1)
```

Workaround forced into `today/`: hoist `g` and `dfs` to module scope. Fixing `cpl build` to emit a C++17 `auto dfs = [&](auto&& self, ...) -> ...` (Y-combinator lambda) would immediately unblock idiomatic nested recursion without global mutable state.

### 2. Recursive Unit-Returning Functions Fail `E0204` in `cpl build`

Even at top level, any recursive function with no explicit return type (implicitly returning unit) is rejected by `cpl build` with `error[E0204]: 'walk' calls itself, so it needs its return type written`, and there is no accepted unit return annotation (`-> ()` / `-> Unit`) in `cpl build` today.

```cp
fn clear_sub(u: Int):
    g_d[u] = INF
    if u * 2 < g_n:
        clear_sub(u * 2)      // E0204 in cpl build
        clear_sub(u * 2 + 1)
```

Workaround forced into `today/noi23p4.cp` and `today/iati21a13.cp`: give every recursive procedure a dummy `-> Int`, append `return 0`, and write `let _ = clear_sub(...)` at every call site. Fix: allow self-recursive procedures without an explicit `-> T` to default to unit (or recognize `-> ()` cleanly).

### 3. Top-Level `let` vs `const`/`var` Visibility in `cpl build`

In `cpl build`, top-level `let MODV = 998244353` is emitted inside `main()`, so top-level `fn` definitions cannot see it (`error[E0201]: no MODV in scope`), whereas top-level `const` and `var` are emitted at file scope. In a contest, a user naturally writes `let MOD = 998244353` at file top before helper functions. Either emit top-level compile-time `let` bindings at file scope or emit an actionable hint (`use const for top-level constants referenced by functions`).

### 4. Member-Domain Paths Rejected in Generic Type Position (`E0105`)

In `iati21a13.cp`, the v2.2 cheat sheet encourages `tree.Pre` (the preorder domain of a `Rooted` tree). Passing `tree.Pre` as a type argument fails the parser in `cpl check`:

```cp
let tree = g.root(V.first)
var bit: Fenwick<tree.Pre, Int> = Fenwick()
// error[E0105]: expected a type (`types are capitalized`)
```

Even binding `let Pre = tree.Pre` still fails `cpl check` (`E0204: no type or domain named Pre`), forcing `domain Pre = 0..n` plus manual `.idx` wrapping (`Pre(tree.pos(u).idx)`). Allowing `ident.CapIdent` (path-qualified associated domains like `tree.Pre`) in type position and `[tree.Pre]T` array heads would remove this entire class of `.idx` casts on Euler-tour tree problems.

### 5. `??=` As an Expression vs Statement (`ioi20p5.cp`)

Memoized map lookup-and-initialize is one of the highest-frequency CP patterns, and users naturally try to chain a method or copy onto `??=`:

```cp
// Rejected by cpl check:
let dt = (cache[(k, x, y)] ??= compute_dt(k, x, y)).copy()

// Forced two-line workaround:
cache[(k, x, y)] ??= compute_dt(k, x, y)
let dt = cache[(k, x, y)]!.copy()
```

Making `map[k] ??= expr` evaluate to a reference/value of `V` (just like Ruby/JS/Kotlin `getOrPut`) eliminates the redundant lookup and the scary `!` unwrap on the next line.

### 6. `cpl build` Lowering Gap Summary Across All 15 Ideal Ports

Auditing all 15 ideal v2.2 ports against the real Rust `cpl build` backend produced six distinct, high-leverage lowering blockers:

- **`@grader` extern declarations (`E0501`, 4/15 ports: `ioi14p1`, `ioi22p5`, `ioi17p4`, `ioi20p5`)**: `cpl build` rejects bodyless `@grader fn` declarations as non-runnable (`there is nothing to run here`). For IOI-style problems, `cpl build --grader` (emitting a header-only/linkable C++ translation unit with `extern` signatures matching the judge grader, no `main()`) is essential so IOI solutions compile directly without a wrapper rewrite.
- **`Mod<P>` (`E0204`, 3/15 ports: `noi23p4`, `noi21p5`, `noi02p6`)**: `Mod<998244353>` and `Mod<10000>` pass `cpl check` cleanly, but `cpl build` does not yet know `Mod`.
- **Empty list / `none` inference across typed sinks (`E0302`, 3/15 ports: `ioi13p3`, `noip20p3`, `iati21a13`)**: `List.filled(c, none)` and `var q: List<(ty: Int, u: V, k: Int)> = []` pass `cpl check`, yet `cpl build`'s lowering pass still complains `can't tell which optional this none is` or `expected List<T>, found List<_>`.
- **`Range<Int>` methods `.step(k)` and `.clamp(lo..hi)` (`E0203`, 2/15 ports: `ioi14p6io`, `noi08p5`)**: Both parse and typecheck in `cpl check`, but `cpl build` lacks lowering for `Range.step` and `Range.clamp`.
- **2D / domain-indexed `SortedBag` (`E0302`, 1/15 ports: `noi19p3`)**: `var bags: [0..4, 0..4]SortedBag<(Int, Int)> = SortedBag()` passes `cpl check` cleanly, while `cpl build` confuses the element monomorphization (`expected [Int, Int]SortedBag_Tup2_Int_P0_0, found SortedBag_Int_1`).
- **`Bool` as a domain key and `Graph` stdlib (`E0201`, 2/15 ports: `noi02p2`, `ctsc04p4`)**: `[Bool]Int` (indexing a 2-element array by `Bool`, used naturally in `noi02p2` and `ioi20p5`) and `Graph<V, Int>` pass `cpl check` but are not yet lowered in `cpl build`.

### 7. Additional Ergonomic Friction Points Spotted During Porting

- **Mutating `read::<T>()` in generic expression positions**: In `cpl build`, `read::<Int>()` emits invalid C++ template syntax (`read<int64_t>()`) when called inside expressions like `a.push(read::<Int>())`. In every `today/` port we had to define a 3-line `fn ri() -> Int` helper. Supporting `read::<Int>()` (or `read(Int)`) directly in `cpl build` eliminates boilerplate across every single program.
- **No `List.pop()` return discard without `let _ =`**: Procedures discarding the popped element (`stk.pop()`) had to write `let _ = stk.pop()` in `today/` ports. Statement-position calls to pure-or-mutating value-returning methods like `.pop()` should not require `let _ =` unless marked `@must_use`.
- **Return types referencing local domains (`ioi14p1.cp`)**: An IOI `@grader` function whose signature receives `n: Int` cannot return `[Station]Int` when `domain Station = 0..n` is declared inside the function body, forcing a fallback to `(List<Int>, List<Int>)`. Dependent return types like `fn findLocation(n: Int) -> ([0..n]Int, [0..n]Int)` or allowing `domain Station = 0..n` in parameter position would keep bounded domains intact across grader boundaries.
- **Ordered-set `.rank(x)` / order-statistic tree (`iati21a13.cp`)**: `Set<T>` / `SortedBag<T>` in v2.2 cleanly expressed `noi19p3` (min/max pop across 16 bags), and `.rank((lim, n))` in `iati21a13.cp` replaced 50 lines of GNU PBDS `tree<..., rb_tree_tag, tree_order_statistics_node_update>` boilerplate. Ensuring `SortedBag.rank(x)` and `SortedBag.kth(k)` are first-class in `std.bag` and lowered in `cpl build` directly covers PBDS order-statistic trees.
- **Explicit `List.copy()` vs value semantics on `.copy()`**: In `ioi20p5.cp` and `noi19p3.cp`, cloning a `List` inside a cache hit or snapshot required explicit manual element loops in `today/` because `cpl build` lacks `.copy()` on `List<T>`.
- **Multi-array parallel sorting / custom comparator**: In `ioi14p6io` and `noip20p3`, `sorted(items, by: desc)` and `by: x => x.key` made ideal v2.2 read like pseudocode, while `today/` had to manually pack `(key, val)` tuples and negate keys for descending order. Lowering `sorted(..., by:)` and `sort(by:)` in `cpl build` is a top contest-ergonomics win.

---

## Verification Evidence

Command run:

```bash
python3 proto/ports/scout_batch_52/test_batch_52.py --cases 50
```

Summary of observed results:

- Ideal v2.2 `cpl check`: **15/15 PASS** (0 errors across all 15 files).
- Ideal v2.2 `cpl build`: **0/15 PASS** (exact first diagnostic per file logged by `test_batch_52.py`).
- Runnable `today/` differential suite (`cpl build` -> `g++ -O2 -fsanitize=address,undefined` vs original C++17 submissions): **15/15 PASS**, **750/750 cases**, zero ASan/UBSan errors.
- Note on constructive / interactive verification: `noip20p3` uses an independent move-sequence validator (`validate_noip20p3`) checking legality, stack height limits, move count `<= 820000`, and monochromatic final stacks in addition to confirming the original C++ passes the same validator; `noi02p2` runs an interactive protocol pipe against the original C++ binary on the identical hidden ring and compares the recovered truth/liar classification; the 5 IOI `@grader` problems (`ioi13p3`, `ioi14p1`, `ioi22p5`, `ioi17p4`, `ioi20p5`) link the original C++ submissions against deterministic local grader wrappers reading the identical hidden test input and compare outputs exactly.
