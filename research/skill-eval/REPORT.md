# CP Lang Skill-v2 Cold Evaluation Report

Cold evaluation of `skills/cp-lang/` on the 20 dry-run contest problems
(`research/dryrun/p01_*` .. `p20_*`). Language knowledge came exclusively from
`skills/cp-lang/SKILL.md` and supplemental `.md` files read on demand; no
`spec/`, `crates/`, `web/`, or existing `.cp` solutions were consulted.

## 1. Per-Problem Results

All 20 solutions are in `research/skill-eval/p01.cp` .. `p20.cp`.
`cpl stress` was run for 200 cases against the compiled C++ reference
(`--brute` + `--gen research/dryrun/pNN_*_gen.py`; `--checker` on `p06`);
`p19` (interactive) was verified via `p19_guess_interact.py` on `1.in` and 50
generated seeds; `@alt(program)` was additionally verified on `p05` (100 cases).

| # | Problem | `check` | `run` (tests) | `stress` | `lint` | `build` + `g++` | Time | Notes |
|---|---|---|---|---|---|---|---|---|
| p01 | Ordinary Numbers | PASS | PASS | PASS (200) | PASS (0w) | PASS | ~1.5m | `@!multitest` worked cleanly |
| p02 | Movie Festival | PASS | PASS | PASS (200) | PASS (0w) | PASS | ~1.0m | `movies.sorted(by: m => m.b)` |
| p03 | Forest Queries | PASS* | PASS | PASS (200) | PASS (0w) | PASS | ~1.5m | *`Prefix(trees)` failed E0302; `Prefix<R, C, Int>(trees)` and `prefix(trees)` worked |
| p04 | Factory Machines | PASS | PASS | PASS (200) | WARN (1w) | PASS | ~1.0m | `L0506` on `1e9` in two unrelated `read` bounds |
| p05 | Subarray Sums I | PASS | PASS | PASS (200+100 `@alt`) | WARN (1w) | PASS | ~1.0m | `L0506` on `1e9` in two unrelated `read` bounds |
| p06 | Labyrinth | PASS | PASS | PASS (200, checker) | PASS (0w) | PASS | ~1.5m | `bfs` over `(R, C)` with `p.adj4` worked on first try |
| p07 | Shortest Routes I | PASS* | PASS* | PASS (200) | PASS (0w) | PASS* | ~2.0m | *`dijkstra(...)` hit `R0402 todo reached` + C++ codegen error; hand-wrote Dijkstra with `MinHeap` |
| p08 | Road Construction | PASS | PASS | PASS (200) | PASS (0w) | PASS | ~0.5m | `Dsu<V>` worked on first try |
| p09 | Course Schedule | PASS | PASS | PASS (200) | PASS (0w) | PASS | ~0.5m | `use std.graph.{smallest_topo}` worked on first try |
| p10 | Tree Matching | PASS | PASS | PASS (200) | PASS (0w) | PASS | ~1.0m | `Rooted` + `t.Pre.rev` + `t.at[p]` worked on first try |
| p11 | Tree Distances II | PASS | PASS | PASS (200) | PASS (0w) | PASS | ~1.0m | `Rooted.down` + `t.depth`/`t.size` worked on first try |
| p12 | Range Updates & Sums | PASS* | PASS | PASS (200) | PASS (0w) | PASS | ~3.0m | *`SegTree(+, a, act:)` failed E0302 + E0204 (`F` unknown); switched to `BeatsSegTree<I>` |
| p13 | Dynamic Range Sum | PASS | PASS | PASS (200) | PASS (0w) | PASS | ~0.5m | `Fenwick<I, Int>(a)` worked on first try |
| p14 | Elevator Rides | PASS* | PASS | PASS (200) | PASS (0w) | PASS | ~2.5m | *`dp[Subset<P>.empty]`/`full` failed E0302; worked around via `for mask in Subset<P>:` + `mask.len` |
| p15 | Book Shop | PASS | PASS | PASS (200) | WARN (1w) | PASS | ~0.5m | `L0506` on `1000` in three unrelated `read` bounds |
| p16 | Increasing Subseq | PASS | PASS | PASS (200) | PASS (0w) | PASS | ~0.5m | `a.lis().len` worked on first try |
| p17 | String Matching | PASS | PASS | PASS (200) | PASS (0w) | PASS | ~1.0m | Applied `L0504` (`sum(1 for _ in text.find_all(pat))`) |
| p18 | Binomial Coeffs | PASS | PASS | PASS (200) | PASS (0w) | PASS | ~0.5m | `Comb<Mod<1_000_000_007>>` worked on first try |
| p19 | Guess the Number | PASS | PASS | PASS (50 interact) | PASS (0w) | PASS | ~1.0m | `(1..=n).last_true(y => ask(y))!`; auto-flush worked |
| p20 | Point Location Test | PASS | PASS | PASS (200) | PASS (0w) | PASS | ~0.5m | `@!multitest` + `read<(Pt, Pt, Pt)>()` + `turn` |

**Totals:** 20/20 passing every stage (4 required workarounds for stdlib/compiler
bugs or skill discrepancies: `p03`, `p07`, `p12`, `p14`; 3 retained deliberate
`L0506` warnings: `p04`, `p05`, `p15`). Total wall time across all 20 problems:
~22 minutes (excluding `cargo build`).

---

## 2. Skill Gaps, Contradictions, and Errors

### 2.1 `skill-gen.py` truncates multi-line / default-value signatures

- **Skill quotes:**
  - `skills/cp-lang/stdlib/tree.md:31-32`:
    ``fn up<T, U, M>( self, op: M, lift: (T, V, V) -> U``
    ``fn reroot<T, U, M>( self, op: M, lift: (T, V, V) -> U``
  - `skills/cp-lang/stdlib/seq.md:268`:
    ``fn lis<D, T, K>(self: [D]T, strict: Bool = true, by: T -> K``
  - `skills/cp-lang/stdlib/algebra.md:26`:
    ``fn new<M>(id: F = F.zero, compose: (F, F) -> F``
  - Also truncated: `graph.md:50-51,58,137,147,150,179,185`,
    `heap.md:17-18,36,65,82,117`, `strings.md:86`, `tree.md:60,64`.
- **What was needed:** Full parameter lists (e.g. `finish` parameter on
  `Rooted.up` / `Rooted.reroot`), default values, and return types (e.g.
  whether `a.lis()` returns `List<D>` or `Int`).
- **What actually worked:** For `p11`, avoided `reroot` and used `Rooted.down`
  whose signature was intact; for `p16`, guessed `a.lis().len` and confirmed via
  `cpl check`.

### 2.2 `Prefix` 2D constructor vs `prefix(...)` prelude omission

- **Skill quotes:**
  - `skills/cp-lang/SKILL.md:33`: `let p = Prefix(a)`
  - `skills/cp-lang/stdlib/prefix.md:5`:
    `Call prefix(a) on a [D]T, [R, C]T, or List<T> to build static prefix sums`
  - `skills/cp-lang/stdlib/prefix.md:7`:
    `In prelude (no use needed): Diff, Prefix, scan.`
  - `skills/cp-lang/stdlib/prefix.md:48`:
    `fn new(a: [R, C]T) -> Prefix<R, C, T> — Build 2-D prefix sums over a: [R, C]T; prefix(a) on a grid calls it.`
- **What was needed:** Building 2D prefix sums over `trees: [R, C]Int` in `p03`.
- **What actually worked:** `Prefix(trees)` failed with
  `E0302: Prefix takes a [R]Int for a, but trees is a [R, C]Int` (type inference
  picked 1D `Prefix<R, Int>`). Both `Prefix<R, C, Int>(trees)` and `prefix(trees)`
  worked — meaning `prefix` IS in the prelude despite `prefix.md:7` omitting it,
  while `Prefix(a)` without type arguments only works for 1D.

### 2.3 `SegTree` constructor and lazy `act:` documentation vs reality

- **Skill quotes:**
  - `skills/cp-lang/stdlib/algebra.md:5`:
    `SegTree<V, Int>(max, a)` and `SegTree<V, Int>(+, a)`
  - `skills/cp-lang/stdlib/README.md:26`:
    `Range apply on a segment tree is SegTree(op, a, act:) (segtree.cp).`
  - `skills/cp-lang/stdlib/segtree.md:13-14`:
    `fn new<M, A>(op: ?M = None, a: ?[D]T = None, fill: ?T = None, act: ?A = None, over: ?Range<Int> = None) -> SegTree<D, T>`
    `fn apply(mut self, r: Range<D>, f: F)`
- **What was needed:** A lazy segment tree with range add, range assign, and
  range sum for `p12`.
- **What actually worked:** `SegTree<I, Int>(+, a)` and
  `SegTree<I, Int>(+, a, act: act)` both failed `E0302: expected ?Int, found (Int, Int) -> Int`.
  Passing `Monoid<Int>.of(+)` fixed construction, but `st.apply(...)` then failed
  `E0204: F is not a type cpl run knows`. Had to abandon `SegTree` and use
  `BeatsSegTree<I>(a)`.

### 2.4 `Action.compose` argument order is undocumented

- **Skill quote:** `skills/cp-lang/stdlib/algebra.md:22`:
  `struct Action<T, F>(id: F, compose: (F, F) -> F, apply: (F, T, Int) -> T, over: Monoid<T>)`
- **What was needed:** Whether `compose(f, g)` means "apply `f` after `g`"
  (new-on-old, standard function composition `f . g`) or "apply `f` then `g`"
  (old-on-new, like `Affine.then`). For non-commutative affine tags (add +
  assign), getting this backwards silently produces wrong answers.
- **What actually worked:** Could not even test it via `SegTree` because of the
  `SegTree.apply` `E0204` bug.

### 2.5 `Subset<B>` empty literal `{}` is wrong, and domain-indexing is undocumented

- **Skill quotes:**
  - `skills/cp-lang/stdlib/subset.md:13`:
    `@get fn empty() -> Subset<B> — O(1); The empty subset, also written {}.`
  - `skills/cp-lang/stdlib/subset.md:9-27`: Never mentions that `Subset<B>` can
    index an array (`var dp: [Subset<P>]T`) or be iterated (`for mask in Subset<P>:`).
- **What was needed:** Bitmask DP table indexed by `Subset<P>` in `p14`,
  initializing the empty subset and reading the full subset.
- **What actually worked:** `{}` failed `E0301: dp is indexed by Subset<P>, but {} is a ()`.
  `dp[Subset<P>.empty]` and `dp[Subset<P>.full]` failed
  `E0302: expected Subset<P>, found Subset<P>`. Iterating `for mask in Subset<P>:`
  and checking `if mask.len == 0:` / `if mask.len == n:` worked.

### 2.6 `cpl build` vs `cpl submit` contradiction

- **Skill quotes:**
  - `skills/cp-lang/SKILL.md:22`:
    ``cpl build sol.cp` writes the C++ to submit (`--judge <name>`, `cpl judges`).`
  - `skills/cp-lang/debugging-and-stress.md:15-16`:
    `cpl build sol.cp                 # release binary`
    `cpl submit sol.cp -o sub.cpp     # single-file C++17 for judge upload`
- **What was needed:** Knowing what `cpl build` produces.
- **What actually worked:** `cpl build sol.cp -o out.cpp` writes C++17 source
  (not a binary), matching `SKILL.md:22` and contradicting
  `debugging-and-stress.md:15`.

### 2.7 `p.adj4` vs `p.adj4()` contradiction

- **Skill quotes:**
  - `skills/cp-lang/stdlib/grid.md:5`:
    `bfs(p => p.adj4.filter(q => g[q] != '#'), from: [s])`
  - `skills/cp-lang/stdlib/graph.md:5`:
    `bfs(p => p.adj4().filter(q => grid[q] != '#'), from: [start])`
- **What was needed:** Calling `adj4` inside `bfs` in `p06`.
- **What actually worked:** `p.adj4` (without parens, since `grid.md:39`
  declares `@get fn adj4`).

### 2.8 `Paths` indexing: `p[v]` vs `p.dist`

- **Skill quotes:**
  - `skills/cp-lang/stdlib/graph.md:73`:
    `fn get(self, v: V) -> ?W — p[v]: the distance to v` (no `dist` field listed)
  - `skills/cp-lang/stdlib/graph.md:193`: `print(p.dist[V(3)]!)`
  - `skills/cp-lang/idioms.md:18`: `print(res.dist, none: -1)`
- **What was needed:** Reading shortest-path distances from `Paths` in `p07`.
- **What actually worked:** `p[v]!` type-checked (though `dijkstra` itself hit
  `R0402`). The struct docs omit `.dist` even though two examples use it.

### 2.9 `Rooted.at` is undocumented as a member

- **Skill quotes:**
  - `skills/cp-lang/stdlib/tree.md:18`: `the inverse of at.`
  - `skills/cp-lang/stdlib/tree.md:29`: `t.at[p] for p in t.sub(v)`
- **What was needed:** Mapping a preorder position `p: t.Pre` back to `V` in
  `p10`.
- **What actually worked:** `t.at[p]` worked, inferred only from the prose in
  `subtree`; `at` has no signature entry in `tree.md`.

### 2.10 Copy-pasted docstrings in `stdlib/grid.md`

- **Skill quotes:**
  - `skills/cp-lang/stdlib/grid.md:19-22`: `.cw`, `.ccw`, `.back`, `.delta` on
    `Dir4` all have the identical description:
    `A quarter turn clockwise: up to right, right to down, down to left, left to up.`
  - `skills/cp-lang/stdlib/grid.md:30-33`: Same copy-paste bug on `Dir8`.
- **What was needed:** Checking whether `Dir4.delta` or a letter helper existed
  for `p06` path reconstruction.

### 2.11 Missing details in `io.md` and `debugging-and-stress.md`

- `skills/cp-lang/io.md:25`: Says `2D grid of ints or chars: let g = read<[R, C]Int>()`
  without naming `Char` (had to guess `Char`, which worked).
- `skills/cp-lang/stdlib/graph.md:149`: Documents `dijkstra` without noting that
  its body is unimplemented (`R0402`).
- `skills/cp-lang/debugging-and-stress.md:69-71`: Lists `--brute`, `--gen`,
  `--checker`, but says nothing about how to test interactive problems like
  `p19`.

---

## 3. Compiler and Standard Library Bugs

All minimal repro files are under `research/skill-eval/repro/` and were verified
with `cpl check` / `cpl run`.

1. **`std.graph.dijkstra` is unimplemented (`R0402`) and breaks C++ codegen**
   - Repro: `research/skill-eval/repro/p07_dijkstra_todo.cp`
   - `cpl check`: passes with 0 errors.
   - `cpl run`: traps with `error[R0402]: todo reached` inside `dijkstra`.
   - `cpl build` + `g++ -O2 -std=c++17`: C++ compilation fails because codegen
     emits `return [&]() -> void { cpl::fail("todo reached"); }();` inside a
     function returning `Paths_...` (a `todo` expression in non-void return
     position produces invalid C++).

2. **`SegTree<D, T>(+, a)` fails type inference on bare operator**
   - Repro: `research/skill-eval/repro/p12_segtree_op_arg.cp`
   - `cpl check`: `error[E0302]: expected ?Int, found (Int, Int) -> Int` on `+`.
   - Passing `Monoid<Int>.of(+)` or using `SegTree<D, T>.of(+, ...)` avoids it.

3. **`SegTree.apply` has an unbound type parameter `F`**
   - Repro: `research/skill-eval/repro/p12_segtree_apply_F.cp`
   - `cpl check`: `error[E0204]: F is not a type cpl run knows` on
     `st.apply(I(1)..=I(3), (mul: 1, add: 5))`.
   - Makes lazy `SegTree(..., act:)` unusable.

4. **`Subset<P>` domain-index type vs `Subset<P>.empty` / `.full` mismatch**
   - Repro: `research/skill-eval/repro/p14_subset_index_type.cp`
   - `cpl check`: `error[E0302]: expected Subset<P>, found Subset<P>` on both
     `dp[Subset<P>.empty]` and `dp[Subset<P>.full]` (and `E0301: this is indexed by Subset<P>, but the index is a Subset<P>` for `Subset<P>(0)`).

5. **`{}` does not construct an empty `Subset<P>`**
   - Repro: `research/skill-eval/repro/p14_empty_subset_braces.cp`
   - `cpl check`: `error[E0301]: dp is indexed by Subset<P>, but {} is a ()`.
   - Either a parser/typechecker bug or a stale claim in `stdlib/subset.md:13`.

---

## 4. Idiom Judgement

### Where our solutions are not idiomatic per `SKILL.md` Practices

- **`p07.cp` (violates Practice #1 "Name the algorithm"):** Hand-rolled Dijkstra
  with `MinHeap<(Int, V)>` because `std.graph.dijkstra` hits `R0402`.
- **`p12.cp` (violates Practice #1 and #10 "Costs are visible"):** Used
  `BeatsSegTree<I>` (an O(log n) amortized Ji Driver tree with min/max/second
  tracking) for plain range add + assign + sum because `SegTree(..., act:)`
  cannot type-check `.apply(...)`.
- **`p14.cp` (awkward workaround for `Subset<P>` bug):** Used
  `if mask.len == 0:` and `if mask.len == n:` inside `for mask in Subset<P>:`
  plus a mutable `var ans = 0` instead of direct `dp[Subset<P>.empty] = (1, 0)`
  and `print(dp[Subset<P>.full]!.0)`.
- **`p06.cp` (violates Practice #5 "Absence is `?T`, never `-1`/`INF`"):**
  Initialized `var start = (R(0), C(0))` and `var goal = (R(0), C(0))` as dummy
  placeholders before scanning the grid, rather than `var start: ?(R, C) = None`
  and unwrapping with `!`. Also reconstructed `'U'/'D'/'L'/'R'` via raw
  `Int(r2) < Int(r1)` comparisons instead of a direction helper.
- **`p01..p20` (Workflow #4 / Practice #12):** Most files omit an inline
  `@alt(program)` brute and relied on `--brute` + `--gen` against the C++
  reference, because `@gen` is not yet implemented (`debugging-and-stress.md:74`)
  and type-driven generation cannot express problem invariants like trees,
  `a < b`, or grids with exactly one `'A'` and `'B'`.

### Where Practices conflicted

- **Practice #6 ("Constraints once, as types") vs Practice #7 ("One home per
  fact") vs `cpl lint` `L0506` (`p04`, `p05`, `p15`):**
  In `p04`, `t` (target products) is `1..=1e9` and `k[i]` (machine time) is
  `1..=1e9`; in `p05`, `x` (target sum) and `a[i]` are both `1..=1e9`; in `p15`,
  `n`, `h[b]`, and `s[b]` are all `1..=1000`. `cpl lint` warns `L0506` on these
  repeated literals and suggests naming a single `const P = 1e9` / `1000`.
  Applying that suggestion violates Practice #7 ("one home per fact") by merging
  unrelated problem constraints into one constant, so per Workflow #5 ("apply
  what it suggests unless clarity suffers") we left those 3 warnings unapplied.
  By contrast, in `p14` (`w[i] <= x`) and `p18` (`Comb<Z>(MAXA)` and
  `0..=MAXA`), the repeated bound genuinely was one fact, so fixing `L0506`
  improved clarity.
