# Stage 4: Head-to-Head Syntax Bakeoff #1

This bakeoff tests the seven highest-stakes syntax and API forks from
`research/language-architecture-synthesis.md` and `decisions.md` against a
fixed benchmark suite of **25 hard, multi-topic problems** from `corpus/`.
Every problem in the suite forces at least three language mechanisms to collide
in the same file (for example: domains + lazy segment trees + sliding windows,
or product domains + 0-1 BFS + enum tags, or strings + DP + lexicographic
reconstruction).

Line count alone hides real ergonomic costs. A four-line chained call full of
nested lambdas, optional-chaining operators, and named closures can cost *more*
keystrokes and *more* symbol density than a nine-line imperative loop while
being harder to debug at 2 AM. Every variant below is written on real logic
from the 25-problem suite and scored on five axes:

1. **Readability at 2 AM (1–5):** How fast a tired contestant can scan the
   snippet and verify every boundary case.
2. **Keystroke count (1–5, plus exact count):** Non-whitespace code characters
   measured directly on the printed snippet (comments excluded).
3. **Punctuation / symbol load (1–5, plus exact count):** Non-alphanumeric,
   non-underscore, non-whitespace characters (`::<>{}[]()!?.=>:+-*/`, etc.).
4. **Mental overhead (1–5):** How many invisible rules (parameter order,
   sequential closure borrows, trait bounds, optional-vs-sentinel conventions,
   direction-flip invariants) must stay in your head to write it cleanly on the
   first try.
5. **Bug-resistance (1–5):** Whether the syntax prevents or invites real
   contest bugs observed in corpus scouts and blind-critic audits.

Higher is better on every 1–5 score (5 = best / lowest friction). Many ports in
`proto/ports/` are design sketches targeting proposed features not yet in
`proto/cpc` or `crates/`; the suite list below states honestly which ports run
today versus which are sketches. No files in `crates/` or `proto/cpc/` were
modified for this bakeoff.

---

## Summary of Verdicts

- **Fork 1 (Sliding windows & two-pointers):** **Winner:** Imperative `Deque`
  and `MonoDeque<T>` value structs + explicit `while` loops for stateful
  two-pointers + optional stateless `slide_until` (with non-optional
  `Range1.first`/`last`). **Loser:** Callback-heavy `slide_until(add:, drop:,
  until:)` and `run.slide(k, MonoDeque(Greater, p => ...))`. The bakeoff caught
  a real logic bug in the synthesis's `noi05p1` example (`p.to_idx` breaks on
  vertical and reversed tilts) and showed that imperative `MonoDeque` uses
  *fewer* keystrokes (172 vs. 175) and 11 fewer symbols than the 4-line
  callback version.
- **Fork 2 (Explicit generic function type arguments):** **Winner:**
  Bidirectional inference from concrete arguments (`from:`, `root:`, `g`) first,
  with `f::<T>()` as the unambiguous fallback. **Losers:** Requiring `f::<T>()`
  everywhere, `f<T>()` lookahead (already rejected in `decisions.md` §15.7 and
  diagnosed by `cpl-check`), and `f[T]()` (collides with indexing). Reordering
  or bidirectionally unifying `from:`/`root:` removes explicit type arguments on
  roughly 90% of real graph and tree calls in the suite.
- **Fork 3 (Product-domain `(Row, Col)` & grid BFS):** **Winner:** General
  `type Cell = (Row, Col)` deriving `Index` + userland `std.grid` with
  explicit `p.adj4` / `p.adj8` (not an unlabelled `p.around()`). **Losers:**
  Separate `(r, c)` tuples with manual `Deque` BFS (2× the keystrokes and
  2.3× the symbols on `ccc18s3`) and a dedicated `Grid<T>` type (fails to
  compose across dual grids, `Dsu<Cell>`, and `@memo`).
- **Fork 4 (`Str` operations):** **Winner (revises `decisions.md` §1.6 and
  `synthesis.md` §2.3B):** Immutable `0..len`-indexed ASCII byte `Str`
  (`[0..len]Char`) with `s[i]`, `s[l..r]`, `s.starts_with(x, at: i)`, `for c in
  s`, and `s + t`, keeping `[domain S]Char = read()` as an explicit opt-in for
  character-domain DP. **Losers:** Current opaque `Str` (593 keystrokes on
  `4331904`) and auto-opening nominal `s.Idx` as the default `Str` (403 vs. 325
  keystrokes on `4331904` because string algorithms constantly add lengths like
  `i + x.len`, `2 * w`, and `q + w`, hitting `?s.Idx` overflow at `s.len`).
- **Fork 5 (Binary search & linear search):** **Winner:** Distinct monotone
  names `bsearch_first`, `bsearch_last`, and `bsearch_first_some` usable as
  free functions on range literals (`bsearch_first(1..=t, done_in)`) and as
  methods on named domains/collections (`xs.first_ge(x)`), paired with
  `first(...)` / `a.find(x)` for O(n) scans. **Losers:** `first_where` (proven
  silent-WA footgun on non-monotone predicates), method-only
  `(1..=t).bsearch_first(...)` (mandatory parens and precedence trap), and
  `partition_point`-only (forces duplicate work on `coci21c4p4` and off-by-one
  `- 1` arithmetic on `bsearch_last`).
- **Fork 6 (Non-typeable stress hooks):** **Winner:** Statement-level `assume`
  + typed `read(gen:)` with `witness` + whole-input `@gen` fallback for coupled
  multi-read constraints, *without* a separate `stress x in ... by ...`
  sublanguage. **Losers:** Adding a dedicated `stress ... by ...` grammar DSL,
  raw-text `@gen`-only (format desync risk), and `where`-only (cannot express
  cross-read invariants). The bakeoff caught a real coupled-read bug in
  `ioi07p2.cp` where `m` is read before `gen_walls` runs.
- **Fork 7 (Tree & HLD path/subtree queries):** **Winner:** `Rooted<V>`
  exposing `chains(a, b, edges?: Bool)` plus generic `tree.fold_path(a, b, ds)`
  / `tree.apply_path(a, b, mut ds, f)` over any `SegTree`/`LazySegTree`/`Fenwick`
  using a `Reversible` monoid (1× memory, O(1) flip). **Losers:** Bundled
  `Hld` / `LazyHld` / `FenwickHld` wrapper proliferation (2× memory in current
  `stdlib/tree.cp` and cannot fit custom DS combinations) and raw `tree.chains`
  alone (invites silent 2 AM `rising`-segment merge bugs on `noi21p1`).

---

## Fixed 25-Problem Benchmark Suite

Future syntax bakeoffs should evaluate competing proposals on this exact
25-problem list so results remain directly comparable across iterations.
Every entry points to a real C++ submission in `corpus/submissions/` and its
current CP Lang port or design sketch in `proto/ports/`.

### 1. DP + Data Structures (4 problems)

1. **`4811954` — IOI 2009 Salesman**
   - Corpus: `corpus/submissions/4811954.cpp` (147 LOC) · Port: `proto/ports/blind_critic_1/4811954.cp` (101 LOC, runnable prototype port)
   - Collisions: Day-grouped DP + two range-max segment trees + coordinate indexing + two-pass same-day sweep.
2. **`noi16p4` — NOI 2016 Intervals**
   - Corpus: `corpus/submissions/4791678.cpp` (111 LOC) · Port: `proto/ports/noi16p4.cp` (36 LOC, runnable prototype port)
   - Collisions: Coordinate compression (`Compress`) + lazy range-add/range-max segment tree (`AddMax<X>`) + variable-width sliding window.
3. **`noi19p1` — NOI 2019 Route**
   - Corpus: `corpus/submissions/3790564.cpp` (172 LOC) · Port: `proto/ports/scout_batch_14/noi19p1.cp` (42 LOC, design sketch)
   - Collisions: Time-indexed event buckets + per-station convex hull / deque DP + domain-indexed station state.
4. **`noi15p3` — NOI 2015 Sushi Dinner**
   - Corpus: `corpus/submissions/3782724.cpp` (202 LOC) · Port: `proto/ports/scout_batch_14/noi15p3.cp` (56 LOC, design sketch)
   - Collisions: Runtime modulus (`ModR`) + ternary `[Small]Side` state index + `Subset<Small>` inclusion-exclusion + filtered loops.

### 2. Tree + Queries (4 problems)

5. **`noi21p1` — NOI 2021 Heavy and Light Edges**
   - Corpus: `corpus/submissions/3859087.cpp` (311 LOC) · Port: `proto/ports/scout_batch_9/noi21p1.cp` (48 LOC, design sketch)
   - Collisions: `Rooted<V>` + `tree.Pre` domain + non-commutative HLD path queries + lazy range assignment + boundary updates.
6. **`noi15p2` — NOI 2015 Package Manager**
   - Corpus: `corpus/submissions/3775038.cpp` (208 LOC) · Port: `proto/ports/scout_batch_2/noi15p2.cp` (35 LOC, design sketch)
   - Collisions: `Rooted<Pkg>` + path-to-root `chains` + `tree.sub(x)` subtree range + lazy assign-sum delta queries.
7. **`sdoi17r2p4` — SDOI 2017 Genius Hacker**
   - Corpus: `corpus/submissions/5515890.cpp` (298 LOC) · Port: `proto/ports/scout_batch_9/sdoi17r2p4.cp` (74 LOC, design sketch)
   - Collisions: Dictionary-tree `Rooted` + LCA + preorder-sorted prefix/suffix virtual graph + weighted Dijkstra.
8. **`noip18p3` — NOIP 2018 Track Construction**
   - Corpus: `corpus/submissions/4002963.cpp` (122 LOC) · Port: `proto/ports/scout_batch_2/noip18p3.cp` (66 LOC, design sketch)
   - Collisions: Binary search on answer (`bsearch_last`) + bottom-up tree DP + ordered multiset pairing (`first_ge`).

### 3. Grid + BFS + Bitmask / State (4 problems)

9. **`noi05p1` — NOI 2005 P1 Gorgeous Waltz**
   - Corpus: `corpus/submissions/5032738.cpp` (156 LOC) · Port: `proto/ports/scout_batch_12/noi05p1.cp` (45 LOC, runnable prototype port)
   - Collisions: Product-domain `(Row, Col)` grid + directional line traversal + obstacle splits + fixed-width monotone-deque DP.
10. **`coci10c7p4` — COCI 2010 Contest 7 #4 Poštar**
    - Corpus: `corpus/submissions/5245379.cpp` (120 LOC) · Port: `proto/ports/scout_batch_12/coci10c7p4.cp` (45 LOC, runnable prototype port)
    - Collisions: Dual grids (`town`, `high`) over `(Row, Col)` + 8-neighbor BFS + sorted unique levels + variable-width two-pointer search.
11. **`ccc18s3` — CCC 2018 S3 RoboThieves**
    - Corpus: `corpus/submissions/4329580.cpp` (141 LOC) · Port: `proto/ports/scout_batch_12/ccc18s3.cp` (65 LOC, runnable prototype port)
    - Collisions: `(Row, Col)` grid + `Dir4` camera rays + conveyor char-to-direction lookup + 0-1 BFS (`shortest`).
12. **`cco19p1` — CCO 2019 P1 Human Error**
    - Corpus: `corpus/submissions/3613457.cpp` (113 LOC) · Port: `proto/ports/scout_batch_12/cco19p1.cp` (32 LOC, design sketch — prototype lacks floats, enum-valued grids, and whole-board memo keys)
    - Collisions: `[Row, Col]?Player` compact board state + 4-neighbor moves + `@memo` over whole board + floating-point expectation.

### 4. String + Hashing (4 problems)

13. **`noi16p1` — NOI 2016 Excellent Splits**
    - Corpus: `corpus/submissions/4797772.cpp` (157 LOC) · Port: `proto/ports/scout_batch_10/noi16p1.cp` (33 LOC, design sketch)
    - Collisions: `Str` + `StrHash` + LCP/LCS binary search + stride multiples `q = k * w` + difference-array range increments.
14. **`noip20p2` — NOIP 2020 String Matching**
    - Corpus: `corpus/submissions/5445673.cpp` (190 LOC) · Port: `proto/ports/scout_batch_10/noip20p2.cp` (46 LOC, design sketch)
    - Collisions: `Str` + `z_function` + `Subset<Letter>` parity toggle + `Int`-indexed prefix/suffix tables (`2 * w`).
15. **`sac22cc3p5` — SAC 2022 CC3 P5 Unscrambled**
    - Corpus: `corpus/submissions/4340958.cpp` (124 LOC) · Port: `proto/ports/scout_batch_10/sac22cc3p5.cp` (44 LOC, design sketch)
    - Collisions: `[W]Str` word table + arena/trie `List<Node>` + `Letter` domain + substring traversal + `Subset` bitmask DP.
16. **`4331904` — Minimum-Token String Segmentation**
    - Corpus: `corpus/submissions/4331904.cpp` (78 LOC) · Port: `proto/ports/blind_critic_1/4331904.cp` (53 LOC, runnable prototype port)
    - Collisions: `Str` + `[D](c: Char, x: Str)` dictionary + suffix DP over `0..=s.len` + lexicographically smallest path reconstruction.

### 5. Flow / Graph + Binary Search (3 problems)

17. **`noi06p4` — NOI 2006 Profit**
    - Corpus: `corpus/submissions/5032812.cpp` (131 LOC) · Port: `proto/ports/scout_batch_5/noi06p4.cp` (25 LOC, design sketch)
    - Collisions: Two domains (`Station`, `User`) + sum-type `enum Item` index + generator `yields` + `max_closure` flow solver.
18. **`coci21c4p4` — COCI 2021/22 #4 Parkovi**
    - Corpus: `corpus/submissions/5120081.cpp` (112 LOC) · Port: `proto/ports/scout_batch_13/coci21c4p4.cp` (37 LOC, design sketch)
    - Collisions: Binary search returning a constructive witness (`bsearch_first_some`) + weighted tree DFS + greedy distance propagation.
19. **`ioi13p5` — IOI 2013 Robots**
    - Corpus: `corpus/submissions/4688418.cpp` (80 LOC) · Port: `proto/ports/scout_batch_5/ioi13p5.cp` (25 LOC, design sketch)
    - Collisions: Binary search on answer (`bsearch_first`) + `MaxHeap<Int>` + streaming sorted cursor (`take_while`) + descending sort.

### 6. Messy Simulation (3 problems)

20. **`ccc19s3` — CCC 2019 S3 Arithmetic Square**
    - Corpus: `corpus/submissions/4325116.cpp` (150 LOC) · Port: `proto/ports/scout_batch_16/ccc19s3.cp` (52 LOC, runnable prototype port)
    - Collisions: `?Int` grid with `"X"` sentinel parsing + randomized restart loop + `witness` + constructive `checker`.
21. **`noi03p1` — NOI 2003 Matchstick Equation**
    - Corpus: `corpus/submissions/3738358.cpp` (151 LOC) · Port: `proto/ports/scout_batch_16/noi03p1.cp` (67 LOC, design sketch)
    - Collisions: Raw-line string parsing + bitmask digit transitions + multi-branch expression evaluation + rollback simulation.
22. **`noip09p4` — NOIP 2009 P4 Target Sudoku**
    - Corpus: `corpus/submissions/5185258.cpp` (102 LOC) · Port: `proto/ports/scout_batch_12/noip09p4.cp` (62 LOC, runnable prototype port)
    - Collisions: `[Line, Line]` 9×9 board + `Subset<Digit>` bitmasks + 3×3 box mapping + MRV backtracking search.

### 7. Geometry (3 problems)

23. **`ioi07p2` — IOI 2007 Flood**
    - Corpus: `corpus/submissions/5133897.cpp` (175 LOC) · Port: `proto/ports/scout_batch_16/ioi07p2.cp` (85 LOC, ideal sketch — not run)
    - Collisions: Planar half-edge face tracing + `Dsu` + BFS over dual faces + coupled non-crossing wall `gen:`/`assume` constraints.
24. **`cco16p2` — CCO 2016 Splitting Hares**
    - Corpus: `corpus/submissions/5566900.cpp` (129 LOC) · Port: `proto/ports/cco16p2.cp` (46 LOC, runnable prototype port)
    - Collisions: Domain `Hare` + 2D vector cross-product comparator + radial sweep + general-position `assume` / `gen:`.
25. **`ccc00s5` — CCC 2000 S5 Sheep and Coyotes**
    - Corpus: `corpus/submissions/4584542.cpp` (106 LOC) · Port: `proto/ports/scout_batch_16/ccc00s5.cp` (42 LOC, design sketch — depends on float/fixed formatting)
    - Collisions: Floating-point input scaling + exact `Frac` half-plane intersection + two-decimal output formatting + tie-rich stress generation.

---

## Fork 1: Sliding Windows & Two-Pointers

`research/language-architecture-synthesis.md` §2.2 proposes a unified `Window`
protocol (`slide`, `slide_until`, `slide_while`, `MonoDeque` as a `Window`
adapter) to eliminate off-by-one two-pointer bugs and hand-rolled deques. We
tested it head-to-head on three distinct sliding-window problems from the suite:
**`noi16p4`** (stateful two-pointer with lazy segtree), **`coci10c7p4`**
(stateless two-pointer where two scout sessions hit `hi < lo`), and
**`noi05p1`** (fixed-width monotone-deque DP along grid lines with obstacles).

### Problem 1A: `noi16p4` (stateful two-pointer with `AddMax<X>`)

**Variant A — Callback `slide_until(add:, drop:, until:)` (synthesis §2.2):**

```cp
for w in I.slide_until(
    add: i => cover(spans[i], 1),
    drop: i => cover(spans[i], -1),
    until: _ => depth.max >= m,
):
    best min= span_len(spans[w.last]) - span_len(spans[w.first])
```

**Variant B — Current explicit `while` loop with `Int` right cursor (`proto/ports/noi16p4.cp`):**

```cp
var right = 0
for left in I:
    while right < n and depth.max < m:
        cover(spans[I.at(right)], 1)
        right += 1
    if depth.max < m: break
    best min= span_len(spans[I.at(right - 1)]) - span_len(spans[left])
    cover(spans[left], -1)
```

**Variant C — Explicit `while` loop using typed `?I` cursor (zero raw `Int` conversions):**

```cp
var r: ?I = I.first
var last = I.first
for l in I:
    while depth.max < m:
        let i = r else: break
        cover(spans[i], 1)
        last = i
        r = i + 1
    if depth.max < m: break
    best min= span_len(spans[last]) - span_len(spans[l])
    cover(spans[l], -1)
```

Notice what happens on `noi16p4` when we compare Variant A and Variant C
honestly. Variant A saves 32 keystrokes (**152 vs. 184**), yet still has
*more* symbols (**42 vs. 39**) and forces you to remember three named closure
parameters (`add:`, `drop:`, `until:`), whether `drop:` fires before or after
the loop body, and a subtle sequential mutable-borrow rule so both `add:` and
`drop:` can mutate `depth` inside the same call. Variant C is three lines
longer, completely eliminates `I.at(right - 1)` and out-of-bounds bugs, and
reads verbatim like the mental model of a two-pointer sweep.

### Problem 1B: `coci10c7p4` (stateless two-pointer over sorted altitudes)

**Variant A — Stateless `slide_until`:**

```cp
for w in levels.slide_until(until: w => delivers(levels[w.first], levels[w.last])):
    best min= levels[w.last] - levels[w.first]
```

**Variant B — Explicit `while` loop (`top max= floor` guard):**

```cp
var top = 0
for floor in 0..levels.len:
    top max= floor
    while top < levels.len and not delivers(levels[floor], levels[top]): top += 1
    if top == levels.len: break
    best min= levels[top] - levels[floor]
```

On a *stateless* two-pointer like `coci10c7p4`, Variant A shines: **114
keystrokes / 24 symbols** vs. **167 keystrokes / 28 symbols**, and it makes the
`hi < lo` empty-window bug structurally impossible — *provided* `w` is a
non-empty `Range1` whose `w.first` and `w.last` are plain indices without `!`
(note that `synthesis.md` wrote `w.first!` and `w.last!`, which adds two
unnecessary panic operators).

### Problem 1C: `noi05p1` (monotone-deque DP along obstacle-split grid lines)

**Variant A — `slide` + `MonoDeque` callback (`synthesis.md` §2.2, corrected):**

```cp
for run in line.split(p => hall[p] == 'x'):
    let idx = [0..run.len] i => (p: run[i], i: i)
    for (cur, best_prev) in idx.slide(seconds + 1, MonoDeque(max, x => far[x.p]? - x.i)):
        next[cur.p] max= best_prev + cur.i
```

**Variant B — Explicit `Deque`:**

```cp
var q: Deque<(val: Int, at: Int)> = []
for i in 0..line.len:
    let p = line[i]
    if hall[p] == 'x':
        q.clear()
        continue
    if let slid = far[p]:
        let v = slid - i
        while q.last?.val <= v: q.pop_back()
        q.push_back((v, i))
    while q.first?.at < i - seconds: q.pop_front()
    if let top = q.first: next[p] = top.val + i
```

**Variant C — Imperative `MonoDeque<Int>(max)` value struct:**

```cp
var q = MonoDeque<Int>(max)
for i in 0..line.len:
    let p = line[i]
    if hall[p] == 'x':
        q.clear()
        continue
    if let slid = far[p]: q.push(i, slid - i)
    q.drop_before(i - seconds)
    if let top = q.best: next[p] = top + i
```

Writing `noi05p1` out for real exposed **two serious flaws** in Variant A from
`synthesis.md` §2.2:

1. The synthesis wrote `MonoDeque(Greater, p => far[p]? - p.to_idx)`. That is a
   **real algorithmic bug**: `p.to_idx` is the global 2-D row-major index
   (`r * m + c`), so on a vertical tilt it jumps by `m` per step instead of `1`,
   and on an up/left tilt it *decreases*. To fix Variant A you have to zip `run`
   with step index `i` (`idx`), inflating Variant A to **175 keystrokes and 53
   symbols**.
2. `far[x.p]? - x.i` relies on magical `?`-skipping inside a `MonoDeque` lambda
   (which `decisions.md` §13 forbids for ordinary lambdas — `?` exits the
   enclosing function, not the element!). What happens if `best_prev` is `none`
   when every cell in the window was unreachable? You have to guess whether
   `slide` skips yielding or yields `?Int`.

Now look at **Variant C** (imperative `MonoDeque<Int>(max)` with `push(at,
val)`, `drop_before(min_at)`, `clear()`, `best -> ?Int`): **9 LOC, 172
keystrokes, 42 symbols** — *fewer* keystrokes and **11 fewer symbols** than
Variant A's 4-line functional chain, zero optional-in-lambda mystery, zero
obstacle-split allocation, and impossible to write the deque popping inequality
backwards at 2 AM.

### Fork 1 Scores

| Variant | Readability 2 AM | Keystrokes | Symbol load | Mental overhead | Bug-resistance |
| --- | --- | --- | --- | --- | --- |
| **A: `Window` protocol + callback `slide` / `slide_until`** | 2 / 5 | 4 / 5 (`152` / `114` / `175`) | 2 / 5 (`42` / `24` / `53`) | 2 / 5 | 3 / 5 |
| **B: Explicit `while` + `Deque`** | 4 / 5 | 3 / 5 (`183` / `167` / `245`) | 3 / 5 (`39` / `28` / `66`) | 4 / 5 | 3 / 5 |
| **C: Imperative `MonoDeque` + `Deque` + explicit stateful `while` + stateless `slide_until`** | **5 / 5** | **4 / 5 (`184` / `114` / `172`)** | **5 / 5 (`39` / `24` / `42`)** | **5 / 5** | **5 / 5** |

**Winner:** **Variant C** — ship `Deque<T>` and imperative `MonoDeque<T>`
(keyed by `Int` position with `push`, `drop_before`, `clear`, `best`) as small
plain stdlib structs, keep explicit `while` loops for stateful two-pointers,
and keep `slide_until` *only* for stateless range windows (yielding non-empty
`Range1` with non-optional `.first`/`.last`), dropping the heavy `Window` trait
and three-closure `slide_until(add:, drop:, until:)` overload.  
**Loser:** **Variant A** (full `Window` trait + multi-closure `slide_until` +
callback `MonoDeque`). It adds a new concept, requires sequential multi-closure
borrows, and had higher symbol density than the imperative alternative.

---

## Fork 2: Explicit Type Arguments on Generic Functions

`decisions.md` §2.13 and §15.7 settled on Rust-style `f::<T>()` for explicit
type arguments on generic functions (and commit `9df888a` already added a
`cpl-check` diagnostic rejecting `f<T>()`). We tested how often explicit type
arguments actually appear across our 25-problem suite — specifically on
**`coci10c7p4`**, **`ccc18s3`**, **`sdoi17r2p4`**, **`noi15p2`**, and
zero-argument helpers like `parse::<Int>`.

### Side-by-side across five real call sites

**Variant A — Turbofish `f::<T>()` required whenever a lambda precedes concrete args:**

```cp
let dist = bfs::<Cell>(p => p.adj8.filter(q => high[q] in lo..=hi), from: [office])
let steps = shortest::<Cell>(from: start, next: moves)
let best = dijkstra::<Node>(g, from: src)
let tree = Rooted::<Pkg>(u => needed_by[u], root: Pkg.first)
let n = parse::<Int>(tok)
```

**Variant B — C++/TS angle brackets `f<T>()` with parser lookahead:**

```cp
let dist = bfs<Cell>(p => p.adj8.filter(q => high[q] in lo..=hi), from: [office])
let steps = shortest<Cell>(from: start, next: moves)
let best = dijkstra<Node>(g, from: src)
let tree = Rooted<Pkg>(u => needed_by[u], root: Pkg.first)
let n = parse<Int>(tok)
```

**Variant C — Bidirectional arg unification (`from:`/`root:`/`g` first) + `f::<T>()` fallback:**

```cp
let dist = bfs(from: [office], next: p => p.adj8.filter(q => high[q] in lo..=hi))
let steps = shortest(from: start, next: moves)
let best = dijkstra(g, from: src)
let tree = Rooted(root: Pkg.first, adj: u => needed_by[u])
let n = parse::<Int>(tok)
```

*(Note: `f[T]()` — Scala/Go bracket syntax — is disqualified outright in CP
Lang because `a[i]` is indexing and `[I]T` is an array type; `f[Cell](x)` is
indistinguishable from calling a function element stored in `[Cell]Fn`.)*

Inspecting all 25 problems reveals the real surprise: **in roughly 90% of
generic function calls (`bfs`, `shortest`, `dijkstra`, `Rooted`, `max_closure`,
`scc`), the type parameter `V` is already determined by a concrete argument
(`from: start`, `root: Pkg.first`, `g: Graph<Node, E>`, `gain: Item -> Int`)!**
The only reason callers were writing `bfs::<Cell>(...)` or `Rooted::<Pkg>(...)`
was left-to-right parameter order putting an unannotated closure `u => ...`
before `from:` / `root:`.

Once the type checker unifies non-closure arguments before checking closure
parameters (and stdlib puts `from:` / `root:` first in signatures), explicit
type arguments disappear almost everywhere, leaving `f::<T>()` only for genuine
return-type-polymorphic calls like `parse::<Int>(tok)` or `zero::<Z>()`.
Variant C beats *even* `f<T>()` lookahead on keystrokes (**209 vs. 221**) and
symbols (**50 vs. 54**) while keeping the parser LL(1) and avoiding the
`a < b and c > d` ambiguity that makes `f<T>()` fragile at 2 AM.

### Fork 2 Scores

| Variant | Readability 2 AM | Keystrokes | Symbol load | Mental overhead | Bug-resistance |
| --- | --- | --- | --- | --- | --- |
| **A: Turbofish `f::<T>()` on graph/tree calls** | 4 / 5 | 3 / 5 (`231`) | 3 / 5 (`64`) | 4 / 5 | 5 / 5 |
| **B: Angle brackets `f<T>()` with lookahead** | 3 / 5 | 4 / 5 (`221`) | 4 / 5 (`54`) | 3 / 5 | 2 / 5 |
| **C: Bidirectional inference + `f::<T>()` fallback** | **5 / 5** | **5 / 5 (`209`)** | **5 / 5 (`50`)** | **5 / 5** | **5 / 5** |
| **D: Brackets `f[T]()`** | 1 / 5 | — | — | 1 / 5 | 1 / 5 (grammar collision) |

**Winner:** **Variant C** — keep `f::<T>()` as the explicit syntax (matching
`decisions.md` §2.13 and `cpl-check`), make bidirectional argument unification
infer `V` from non-closure arguments before checking lambdas, and standardize
stdlib parameter order with `from:` / `root:` before `next:` / `adj:`.  
**Losers:** **Variant B** (`f<T>()` lookahead), **Variant D** (`f[T]()`), and
forcing turbofish on every `bfs`/`Rooted` call.

---

## Fork 3: Product-Domain `(Row, Col)` Indexing & Grid BFS

`synthesis.md` §2.1A & §2.3B unify 2-D grids under the general product domain
`type Cell = (Row, Col)` deriving `Index` (automatically flattening to
`Row.len * Col.len`), plus a tiny userland `std.grid` module providing `Dir4`,
`Dir8`, `p.step(d)`, `p.ray(d)`, and neighbor helpers. We tested it against
today's separate `(r, c)` style and against a dedicated `Grid<T>` type on
**`ccc18s3`** (camera rays + conveyors + 0-1 BFS), **`coci10c7p4`** (dual grids
+ 8-neighbor BFS), and **`cco19p1`** (board-state `@memo`).

### Problem 3: `ccc18s3` (camera visibility + conveyors + 0-1 BFS)

**Variant A — Product-domain `type Cell = (Row, Col)` + `std.grid` + `shortest`:**

```cp
type Cell = (Row, Col)
var blocked: [Cell]Bool = false
for p in Cell if grid[p] == 'C':
    blocked[p] = true
    for d in Dir4:
        for q in p.ray(d).take_while(q => grid[q] != 'W'):
            if grid[q] in ".S": blocked[q] = true

let dist = shortest(from: start, next: p =>
    if let d = Dir4.from("UDLR", grid[p]):
        if let q = p.step(d) if grid[q] != 'W' and not blocked[q]: [(q, 0)] else: []
    else:
        [(q, 1) for q in p.adj4 if grid[q] != 'W' and not blocked[q]]
)
```

**Variant B — Separate `(r, c)` indices + manual `Deque` 0-1 BFS (current working style):**

```cp
const DIRS = [(-1, 0), (1, 0), (0, -1), (0, 1)]
var blocked: [Row, Col]Bool = false
for r in Row:
    for c in Col:
        if grid[r, c] == 'C':
            blocked[r, c] = true
            for (dr, dc) in DIRS:
                var (nr, nc) = (r, c)
                while let (rr, cc) = (nr + dr, nc + dc):
                    if grid[rr, cc] == 'W': break
                    if grid[rr, cc] == '.' or grid[rr, cc] == 'S': blocked[rr, cc] = true
                    (nr, nc) = (rr, cc)
var dist: [Row, Col]?Int = none
var q: Deque<(Row, Col)> = []
if not blocked[sr, sc]:
    dist[sr, sc] = 0
    q.push_front((sr, sc))
while let (r, c) = q.pop_front():
    let d = dist[r, c]!
    if let (dr, dc) = conveyor(grid[r, c]):
        if let (nr, nc) = (r + dr, c + dc) if grid[nr, nc] != 'W' and not blocked[nr, nc]:
            if d < dist[nr, nc] ?? INF:
                dist[nr, nc] = d
                q.push_front((nr, nc))
    else:
        for (dr, dc) in DIRS:
            if let (nr, nc) = (r + dr, c + dc) if grid[nr, nc] != 'W' and not blocked[nr, nc]:
                if d + 1 < dist[nr, nc] ?? INF:
                    dist[nr, nc] = d + 1
                    q.push_back((nr, nc))
```

**Variant C — Dedicated `Grid<T>` container (`grid.cells()`, `grid.ray(p, d)`, `grid.shortest(...)`):**

```cp
var blocked = grid.map(_ => false)
for p in grid.cells() if grid[p] == 'C':
    blocked[p] = true
    for d in Dir4:
        for q in grid.ray(p, d).take_while(q => grid[q] != 'W'):
            if grid[q] in ".S": blocked[q] = true

let dist = grid.shortest(from: start, next: p =>
    if let d = Dir4.from("UDLR", grid[p]):
        if let q = grid.step(p, d) if grid[q] != 'W' and not blocked[q]: [(q, 0)] else: []
    else:
        [(q, 1) for q in grid.adj4(p) if grid[q] != 'W' and not blocked[q]]
)
```

The numbers on `ccc18s3` are a landslide. Variant A cuts keystrokes in half
(**355 vs. 720**) and cuts symbol load by more than half (**97 vs. 228**)
compared to Variant B, while eliminating three classic contest bugs: swapped
`(r, c)` arguments, out-of-bounds ray steps, and the copy-pasted 0-1 BFS
`push_front`/`push_back` bug.

Variant C (`Grid<T>`) looks competitive on `ccc18s3` (**368 keystrokes, 103
symbols**), but it falls apart as soon as a problem has **two grids over the
same cells** (`town` and `high` in `coci10c7p4`), feeds cells into `Dsu<Cell>`
or `SegTree<Cell>`, or uses `[Cell]?Player` as a `@memo` key (`cco19p1`).
Because `(Row, Col)` is just a regular `Index`, Variant A reuses the exact same
`[Cell]T`, `bfs`, `shortest`, `Dsu`, and `@memo` with zero grid-specific
duplication.

### Critical Sub-Fork: `p.around()` vs. `p.adj4` / `p.adj8`

`synthesis.md` §2.3B proposed `p.around()` defaulting to 4-neighbors and
`p.around(Dir8)` for 8-neighbors. On our suite, **`coci10c7p4` uses 8-neighbor
moves while `ccc18s3` and `cco19p1` use 4-neighbor moves**. At 2 AM, writing
`p.around()` in `coci10c7p4` compiles silently and produces a wrong answer
because "around" in English sounds like all 8 surrounding squares! Naming them
explicitly **`p.adj4`** and **`p.adj8`** (with `p.step(d)` and `p.ray(d)`
taking `Dir4` or `Dir8`) saves 2 keystrokes over `p.around()` *and* makes the
4-vs-8 choice impossible to miss when scanning code.

### Fork 3 Scores

| Variant | Readability 2 AM | Keystrokes | Symbol load | Mental overhead | Bug-resistance |
| --- | --- | --- | --- | --- | --- |
| **A: `type Cell = (Row, Col): Index` + `std.grid` (`p.adj4`/`p.adj8`)** | **5 / 5** | **5 / 5 (`355`)** | **5 / 5 (`97`)** | **5 / 5** | **5 / 5** |
| **B: Separate `(r, c)` + manual `Deque` BFS** | 2 / 5 | 1 / 5 (`720`) | 1 / 5 (`228`) | 3 / 5 | 2 / 5 |
| **C: Dedicated `Grid<T>` type** | 4 / 5 | 4 / 5 (`368`) | 4 / 5 (`103`) | 3 / 5 | 4 / 5 |

**Winner:** **Variant A** (`(Row, Col): Index` + userland `std.grid` with
explicit `p.adj4` / `p.adj8` instead of default-4 `p.around()`).  
**Losers:** **Variant B** (manual `(r, c)` + `Deque` boilerplate) and
**Variant C** (bespoke `Grid<T>` that breaks composition with `Dsu`, `shortest`,
and dual grids).

---

## Fork 4: `Str` Operations (Nominal Auto-Opening `[domain]Char` vs. `0..len`-Indexed `Str`)

This fork produced the **biggest design reversal of the bakeoff**.
`decisions.md` §1.6 and `synthesis.md` §2.3B model `Str` as an existential
`[domain]Char` package that opens a fresh nominal index domain `s.Idx` wherever
a `Str` is named (including inside structs and arrays). We tested that design
against a plain immutable `0..len`-indexed ASCII byte `Str` (`[0..len]Char`,
retaining `[domain S]Char = read()` as an explicit opt-in) and against today's
opaque `Str` across all four string problems in the suite (**`4331904`**,
**`noip20p2`**, **`noi16p1`**, **`sac22cc3p5`**).

### Problem 4A: `4331904` (dictionary matching DP + lexicographic reconstruction)

**Variant A — Auto-opening nominal `[domain]Char` (`s.Idx` and `s.Idx.bounds`):**

```cp
let s: Str = read()
let dict: [D](c: Char, x: Str) = read()
var dp: [s.Idx.bounds]?Int = none
dp[s.Idx.bounds.last] = 0
for i in s.Idx.rev:
    for (c, x) in dict:
        if let end = i.r(x.len) if s[i.l..end] == x:
            if let nxt = dp[end]: dp[i.l] min= 1 + nxt
var out: List<Char> = []
var pos = s.Idx.bounds.first
while pos != s.Idx.bounds.last:
    let (c, end) = min_by(
        [(c, end) for (c, x) in dict
            if let end = pos.after(x.len) if s[pos..end] == x and dp[end] == dp[pos]! - 1],
        e => e.0
    )!
    out.push(c)
    pos = end
```

**Variant B — Immutable `0..len`-indexed ASCII `Str` (`[0..len]Char`, with opt-in `[domain S]Char`):**

```cp
let s: Str = read()
let dict: [D](c: Char, x: Str) = read()
var dp: [0..=s.len]?Int = none
dp[s.len] = 0
for i in (0..s.len).rev:
    for (c, x) in dict:
        if s.starts_with(x, at: i):
            if let nxt = dp[i + x.len]: dp[i] min= 1 + nxt
var out = ""
var i = 0
while i < s.len:
    let (c, step) = min_by(
        [(c, x.len) for (c, x) in dict
            if s.starts_with(x, at: i) and dp[i + x.len] == dp[i]! - 1],
        e => e.0
    )!
    out.push(c)
    i += step
```

**Variant C — Today's opaque `Str` requiring manual `[domain S]Char` + `List<Char>` copy:**

```cp
let s: [domain S]Char = read()
var chars: [D]Char = 'a'
var words: [D]List<Char> = []
for d in D:
    let c: Char = read()
    let w: [domain W]Char = read()
    chars[d] = c
    for k in W: words[d].push(w[k])
var dp: [0..=S.len]?Int = none
dp[S.len] = 0
for i in (0..S.len).rev:
    for d in D:
        let w = words[d]
        if i + w.len <= S.len and all(s[S.at(i + j)] == w[j] for j in 0..w.len):
            if let nxt = dp[i + w.len]: dp[i] min= 1 + nxt
var out: List<Char> = []
var i = 0
while i < S.len:
    var best_c: ?Char = none
    var best_len = 0
    for d in D:
        let w = words[d]
        if i + w.len <= S.len and all(s[S.at(i + j)] == w[j] for j in 0..w.len):
            if dp[i + w.len] == dp[i]! - 1 and (best_c == none or chars[d] < best_c!):
                best_c = chars[d]
                best_len = w.len
    out.push(best_c!)
    i += best_len
```

### Problem 4B: `noip20p2` (`z_function` + parity prefix table)

**Variant A — Nominal `s.Idx` colliding with `z_function` (`Int` lengths):**

```cp
let s: Str = read()
let z = z_function(s)
var odd_from: [0..=s.len]Int = 0
var seen: Subset<Letter> = Subset.empty
for i in s.Idx.rev:
    seen.toggle(Letter(s[i]))
    odd_from[i.idx] = seen.len
for w in 2..s.len:
    let copies = min(1 + z[w] / w, (s.len - 1) / w)
    let even = copies / 2
    ans += (copies - even) * by_odd[odd_from[w]] + even * by_odd[odd_from[2 * w]]
```

**Variant B — `0..len`-indexed `Str`:**

```cp
let s: Str = read()
let z = z_function(s)
var odd_from: [0..=s.len]Int = 0
var seen: Subset<Letter> = Subset.empty
for i in (0..s.len).rev:
    seen.toggle(Letter(s[i]))
    odd_from[i] = seen.len
for w in 2..s.len:
    let copies = min(1 + z[w] / w, (s.len - 1) / w)
    let even = copies / 2
    ans += (copies - even) * by_odd[odd_from[w]] + even * by_odd[odd_from[2 * w]]
```

Look closely at what happened across **all four** string problems in the suite:

1. In `4331904`, a match ending at the final byte of `s` lands at boundary
   `s.len` — which is *not* a valid character position in `s.Idx` (`0..s.len`).
   So Variant A forces you to switch back and forth between `s.Idx` (characters)
   and `s.Idx.bounds` (boundaries), using `i.l`, `i.r(x.len)`, `pos.after(x.len)`,
   and `s.Idx.bounds.last` (**403 keystrokes, 115 symbols** vs. Variant B's
   **325 keystrokes, 102 symbols**).
2. In `noip20p2`, `z_function(s)` returns `Int` lengths (`decisions.md` §10
   explicitly notes that Z/KMP/SA outputs are lengths and offsets!), and the
   algorithm indexes `odd_from[w]` and `odd_from[2 * w]`, forcing `odd_from[i.idx]`
   under Variant A.
3. In `noi16p1` (`q = k * w`, `lcp(q, q + w)`) and `sac22cc3p5` (`for start in
   0..s.len: for i in start..s.len: s[i]`), every index is arithmetic on lengths
   and offsets too!
4. Variant B stays **100% inside the `plan/19` kernel**: `Str` is simply an
   ASCII byte sequence indexed by `0..len` (with bounds-checked `s[i]`, slicing
   `s[l..r]`, `s.starts_with(x, at: i)`, `for c in s`, `s + t`, `sep.join`, and
   mutable `out.push(c)` or `List<Char>` builder). Whenever a problem genuinely
   wants a nominal character domain (e.g., two strings `A` and `B` in edit
   distance / LCS DP where swapping `i` and `j` is a real bug), `let a: [domain
   A]Char = read()` remains a one-line explicit opt-in!

### Fork 4 Scores

| Variant | Readability 2 AM | Keystrokes | Symbol load | Mental overhead | Bug-resistance |
| --- | --- | --- | --- | --- | --- |
| **A: Auto-opening nominal `s.Idx` as default `Str`** | 2 / 5 | 3 / 5 (`403` / `292`) | 3 / 5 (`115` / `71`) | 2 / 5 | 4 / 5 |
| **B: `0..len`-indexed ASCII `Str` + opt-in `[domain S]Char`** | **5 / 5** | **5 / 5 (`325` / `293`)** | **5 / 5 (`102` / `74`)** | **5 / 5** | **4 / 5** |
| **C: Today's opaque `Str` + manual `List<Char>` copy** | 1 / 5 | 1 / 5 (`593`) | 1 / 5 (`166`) | 2 / 5 | 3 / 5 |

**Winner:** **Variant B** — make `Str` an immutable `0..len`-indexed ASCII byte
sequence (with `s[i]`, `s[l..r]`, `s.starts_with(x, at: i)`, `for c in s`,
`s + t`, `sep.join`, and `Str.from(chars)` / `List<Char>`), while keeping
`[domain S]Char = read()` opt-in for multi-string DP.  
**Losers:** **Variant C** (today's opaque `Str`, unworkable) and **Variant A**
(nominal `s.Idx` by default — fights every real string algorithm in the suite).

---

## Fork 5: Binary Search & Linear Search API

In `blind-critic-audit-1.md` and `decisions.md` §15, calling binary search
`first_where` caused a **silent wrong answer** on a non-monotone predicate
(`9-T12`, `ioi19p2`) because contestants read `first_where` as linear search.
`synthesis.md` §2.3A proposes renaming binary search to `bsearch_first` /
`bsearch_last` / `bsearch_first_some` (and `first_ge` on sorted containers)
while reserving `first(...)` and `a.find(x)` for O(n) linear scans. We tested
four API shapes across **`coci21c4p4`**, **`noip18p3`**, **`ioi13p5`**, and
linear search.

### Side-by-side across four real search patterns

**Variant A — Free functions on ranges + methods on named domains/collections:**

```cp
let (reach, built) = bsearch_first_some(0..=2e14, parks)!
let track = bsearch_last(0..=total, k => built_tracks(k) >= m)!
let mins = bsearch_first(1..=t, done_in) ?? -1
let partner = bag.first_ge(k - w)
let bad = first(i for i in 0..n if a[i] != want[i])
```

**Variant B — Method-only on `Range` (`synthesis.md` §2.3A literal syntax):**

```cp
let (reach, built) = (0..=2e14).bsearch_first_some(parks)!
let track = (0..=total).bsearch_last(k => built_tracks(k) >= m)!
let mins = (1..=t).bsearch_first(done_in) ?? -1
let partner = bag.first_ge(k - w)
let bad = first(i for i in 0..n if a[i] != want[i])
```

**Variant C — Current prototype names (`first_where` / `last_where` / `first_some`):**

```cp
let (reach, built) = (0..=2e14).first_some(parks)!
let track = (0..=total).last_where(k => built_tracks(k) >= m)!
let mins = (1..=t).first_where(done_in) ?? -1
let partner = bag.first_ge(k - w)
let bad = (0..n).first_where(i => a[i] != want[i])
```

**Variant D — Single `partition_point` (Rust-style):**

```cp
let reach = (0..=2e14).partition_point(r => parks(r) == none)
let built = parks(reach)!
let track = (0..=total).partition_point(k => built_tracks(k) >= m) - 1
let r = (1..=t).partition_point(m => not done_in(m))
let mins = if r <= t: r else: -1
let partner = bag.first_ge(k - w)
let bad = first(i for i in 0..n if a[i] != want[i])
```

Three findings stand out immediately:

1. **Variant C (`first_where`) is a proven contest hazard.** Notice the last
   line of Variant C: `(0..n).first_where(i => a[i] != want[i])` reads like
   plain English for "first index where `a[i]` differs from `want[i]`", compiles
   with zero warning, and silently binary-searches a non-monotone predicate.
2. **Variant D (`partition_point` only) has severe 2 AM friction.** It jumps to
   **274 keystrokes and 70 symbols** (60 more keystrokes than Variant A),
   re-runs the O(n) `parks(reach)` construction a second time on `coci21c4p4`,
   forces you to invert `not done_in(m)` on `ioi13p5`, and introduces the
   classic `- 1` off-by-one trap on `bsearch_last` (`noip18p3`).
3. **Variant A beats Variant B on range literals.** Because `..` / `..=` binds
   looser than `.`, method-only `(1..=t).bsearch_first(done_in)` requires outer
   parentheses on every range literal (6 extra symbols across 3 calls, and
   `1..=t.bsearch_first(...)` is a classic first-draft parse error). Providing
   `bsearch_first(range, ok)` / `bsearch_last(range, ok)` /
   `bsearch_first_some(range, f)` as free functions (alongside method syntax on
   named domains and sorted containers like `bag.first_ge(x)`) removes the parens
   completely and makes range-based binary search read left-to-right like `sum`,
   `min`, and `first`.

### Fork 5 Scores

| Variant | Readability 2 AM | Keystrokes | Symbol load | Mental overhead | Bug-resistance |
| --- | --- | --- | --- | --- | --- |
| **A: Free-fn `bsearch_first`/`last`/`first_some` + container `.first_ge` + `first(...)`** | **5 / 5** | **5 / 5 (`215`)** | **5 / 5 (`51`)** | **5 / 5** | **5 / 5** |
| **B: Method-only `(lo..=hi).bsearch_first(...)`** | 4 / 5 | 4 / 5 (`221`) | 4 / 5 (`57`) | 4 / 5 | 5 / 5 |
| **C: `first_where` / `last_where` / `first_some`** | 2 / 5 | 5 / 5 (`212`) | 3 / 5 (`62`) | 2 / 5 | 1 / 5 (silent WA) |
| **D: `partition_point` only** | 2 / 5 | 2 / 5 (`274`) | 2 / 5 (`70`) | 2 / 5 | 2 / 5 (off-by-one) |

**Winner:** **Variant A** — `bsearch_first`, `bsearch_last`, and
`bsearch_first_some` (debug-checked monotone on boundaries and sampled points)
callable as free functions on ranges and as methods on domains, plus
`xs.first_ge(x)` on sorted collections and `first(...)` / `a.find(x)` for linear
scans.  
**Losers:** **Variant C** (`first_where` — retire completely) and **Variant D**
(`partition_point` alone).

---

## Fork 6: Non-Typeable Stress Hooks (`assume`, `gen:`, `witness`, `@gen`, `stress`)

`research/friction-backlog-batch-16.md` and `synthesis.md` §4 address inputs
whose constraints cannot be expressed as per-element types: non-crossing planar
walls (**`ioi07p2`**), guaranteed-solvable puzzles (**`ccc19s3`**), points in
general position (**`cco16p2`**), and tie-rich floating-point coordinates
(**`ccc00s5`**). We tested three designs side-by-side.

### Side-by-side on `ioi07p2` and `ccc00s5`

**Variant A — `assume` + per-read `gen:` + `stress <field> in <range> by <step>` DSL (`synthesis.md` §4):**

```cp
let pts: [P]Pt = read(gen: gen_pts)
let walls: [W](u: P, v: P) = read(gen: gen_walls)
assume all(pts[w.u].x == pts[w.v].x or pts[w.u].y == pts[w.v].y for w in walls)
assume no_two_cross(pts, walls)

let sheep: [S](x: Float, y: Float) = read()
    stress x in 0..=3 by 0.25, y in 0.25..=3 by 0.25
```

**Variant B — `assume` + typed `gen:` (with `witness` and coupled tuple/whole-input `@gen`) and NO `stress ... by ...` DSL:**

```cp
let pts: [P]Pt = read(gen: gen_pts)
let (m, walls) = read(gen: () => gen_walls(pts))
assume all(pts[w.u].x == pts[w.v].x or pts[w.u].y == pts[w.v].y for w in walls)
assume no_two_cross(pts, walls)

let sheep: [S](x: Fixed<2>, y: Fixed<2>) = read(gen: () =>
    [S] _ => (Fixed(rand(0, 12) * 25), Fixed(rand(1, 12) * 25))
)
```

**Variant C — Whole-input raw-text `@gen` printing stdin manually:**

```cp
@gen fn make_flood(size: Int):
    let side = rand(2, max(2, isqrt(size)))
    let n = side * side
    print(n)
    for y in 0..side:
        for x in 0..side: print("{x} {y}")
    let near = grid_edges(side)
    let m = rand(1, near.len)
    print(m)
    for (u, v) in sample(near, m): print("{u} {v}")
```

**Constructive witness companion (`ccc19s3`, shared by Variants A & B):**

```cp
let given: [Line, Line]?Int = read(gen: gen_square)

fn gen_square() -> [Line, Line]?Int:
    let base = random_ap_square()
    let mask = sample(cells, rand(0, 9))
    let g = [Line, Line] (r, c) => if (r, c) in mask: none else: base[r, c]
    witness base
    g

checker(out: [Line, Line]Int):
    for (r, c) in Line * Line:
        if let v = given[r, c]: assert out[r, c] == v
    for r in Line: assert out[r, 0] + out[r, 2] == 2 * out[r, 1]
    for c in Line: assert out[0, c] + out[2, c] == 2 * out[1, c]
```

Inspecting `proto/ports/scout_batch_16/ioi07p2.cp` line by line caught a **real
bug in per-read `gen:`**: in `ioi07p2.cp`, `let m: 1..=2e5 = read()` is drawn
*before* `gen_walls()` runs, so `sample(near, m)` crashes whenever `m >
near.len`! A per-read `gen:` only works when the read itself owns all coupled
fields (such as `let (m, walls) = read(gen: ...)` in Variant B, or a typed
whole-input `@gen` when `n`, `pts`, `m`, and `walls` are mutually constrained).

Meanwhile, comparing Variant A and Variant B on `ccc00s5` shows that the
`stress x in 0..=3 by 0.25, y in 0.25..=3 by 0.25` syntax saves only **25
keystrokes** (242 vs. 267) at the cost of introducing a brand-new grammar
sublanguage (`stress <field> in <lo>..=<hi> by <step>`) with its own scoping
and comma-list rules. Ordinary `gen:` closures already handle tie-rich narrowing
using the exact same language constructs you already know — and `assume` checks
both real inputs and `gen:` outputs without a second syntax. Finally, Variant C
(raw-text `@gen` printing stdin) is short on `ioi07p2` (**219 keystrokes**) but
loses hard on bug-resistance: one missing newline or swapped `(u, v)` format
silently breaks the generator without the type checker noticing, whereas typed
`gen:` reuses `read()`'s exact codec and pairs naturally with `witness` and
`checker` on `ccc19s3`.

### Fork 6 Scores

| Variant | Readability 2 AM | Keystrokes | Symbol load | Mental overhead | Bug-resistance |
| --- | --- | --- | --- | --- | --- |
| **A: `assume` + per-read `gen:` + `stress ... by ...` DSL** | 4 / 5 | 5 / 5 (`242`) | 4 / 5 (`65`) | 3 / 5 (extra DSL) | 4 / 5 |
| **B: `assume` + typed `gen:`/`witness` + coupled `@gen` fallback (no DSL)** | **5 / 5** | **4 / 5 (`267`)** | **4 / 5 (`84`)** | **5 / 5** | **5 / 5** |
| **C: Whole-input raw-text `@gen` only** | 3 / 5 | 5 / 5 (`219`) | 5 / 5 (`57`) | 4 / 5 | 2 / 5 (codec desync) |

**Winner:** **Variant B** — keep the concept budget lean with just three
orthogonal stress primitives: `assume` (checks real inputs and validates
`gen:`), typed `read(gen:)` (with `witness` for constructive problems and
coupled tuple reads), and typed whole-input `@gen` when earlier sizes depend on
later structures; drop the `stress ... by ...` sublanguage.  
**Losers:** **Variant C** (raw-text `@gen`-only) and the `stress ... by ...`
sub-grammar in **Variant A**.

---

## Fork 7: Tree & HLD Path / Subtree Queries

`stdlib/tree.cp` currently ships `Rooted<V>` (with `sub`, `lca`, `dist`,
`chains`) *and* a bundled `Hld<V, T>` struct that allocates **two** `SegTree`
instances (`down` and `rise`) to handle non-commutative monoids. `synthesis.md`
§2.1D proposes adding `LazyHld` alongside it. We tested three designs on
**`noi21p1`** (non-commutative heavy-edge count with lazy path color
assignment) and **`noi15p2`** (path-to-root and subtree assign-sum returning
changed counts).

### Problem 7A: `noi21p1` (non-commutative path fold + lazy range assignment)

**Variant A — Raw `tree.chains(a, b)` over `LazySegTree<tree.Pre>`:**

```cp
var left_acc = EdgeRun.id
var right_acc = EdgeRun.id
for c in tree.chains(a, b):
    let seg = tree_seg.fold(c.range)
    if c.rising: left_acc = EdgeRun.op(left_acc, seg.rev())
    else: right_acc = EdgeRun.op(right_acc, seg)
let ans = EdgeRun.op(left_acc, right_acc).heavy
```

**Variant B — Bundled `LazyHld<V, T, F>` wrapper class:**

```cp
var hld = LazyHld(tree, EdgeRun.monoid, AssignColor.action, init)
hld.apply_path(a, b, now)
let ans = hld.path(a, b).heavy
```

**Variant C — `Rooted` + generic `tree.fold_path` / `tree.apply_path` over any DS + `Reversible` monoid:**

```cp
var seg = LazySegTree<tree.Pre>(EdgeRun.monoid, AssignColor.action, init)
tree.apply_path(a, b, mut seg, now)
let ans = tree.fold_path(a, b, seg).heavy
```

### Problem 7B: `noi15p2` (path-to-root & subtree lazy assign-sum delta)

**Variant A — Raw `tree.chains` + helper `switch`:**

```cp
fn switch(r: Range<tree.Pre>, on: Bool) -> Int:
    let before = seg.fold(r)
    seg.apply(r, on)
    abs(seg.fold(r) - before)

if op == "install": print(sum(switch(c.range, true) for c in tree.chains(Pkg.first, x)))
else: print(switch(tree.sub(x), false))
```

**Variant B — Bundled `LazyHld`:**

```cp
if op == "install":
    let before = hld.path(Pkg.first, x)
    hld.apply_path(Pkg.first, x, true)
    print(hld.path(Pkg.first, x) - before)
else:
    let before = hld.sub(x)
    hld.apply_sub(x, false)
    print(before - hld.sub(x))
```

**Variant C — `Rooted` + generic `tree.fold_path` / `tree.apply_path` + direct `seg` on `tree.sub(x)`:**

```cp
if op == "install":
    let before = tree.fold_path(Pkg.first, x, seg)
    tree.apply_path(Pkg.first, x, mut seg, true)
    print(tree.fold_path(Pkg.first, x, seg) - before)
else:
    let before = seg.fold(tree.sub(x))
    seg.apply(tree.sub(x), false)
    print(before - seg.fold(tree.sub(x)))
```

Look closely at what happens across both problems:

1. **Variant A (raw `tree.chains` alone) is a 2 AM minefield for
   non-commutative path folds.** In `noi21p1`, `c.rising` segments are yielded
   from `a` up to `lca`, and falling segments from `lca` down to `b`. Even when
   `chains` orders them carefully, forgetting `seg.rev()` on `c.rising` or
   combining `EdgeRun.op(seg, left_acc)` in the wrong order causes a silent WA.
2. **Variant B (bundled `Hld` / `LazyHld`) looks short per call (108 keystrokes
   on `noi21p1`), but fails the architecture and max-N test:**
   - Today's `Hld` in `stdlib/tree.cp` allocates **two** segment trees (`down`
     and `rise`), doubling memory (8N nodes) and halving update throughput even
     for commutative monoids like `Sum` or `Max`!
   - Every new range data structure forces a parallel wrapper (`Hld`, `LazyHld`,
     `FenwickHld`, `AssignSumHld`, `LiChaoHld`), and none of them fit custom
     two-tree tricks like `noi21p1`'s `stamp` + `tied` boundary update.
3. **Variant C gets 95% of Variant B's conciseness with none of its cost.**
   `tree.fold_path(a, b, seg)` and `tree.apply_path(a, b, mut seg, tag)` are
   ~10-line generic methods on `Rooted<V>` taking any range-fold / range-apply
   structure indexed by `tree.Pre`. When the monoid implements `Reversible`
   (trivial O(1) `rev()` swapping `(left, right)` on `EdgeRun`, or identity on
   commutative monoids), `fold_path` flips `c.rising` automatically using **one
   segment tree (1× memory)** instead of two — and `tree.chains(a, b)` stays
   public right underneath for problems like `noi15p2`'s `switch` or `noi21p1`'s
   chain-boundary updates!

### Fork 7 Scores

| Variant | Readability 2 AM | Keystrokes | Symbol load | Mental overhead | Bug-resistance |
| --- | --- | --- | --- | --- | --- |
| **A: Raw `tree.chains(a, b)` only** | 3 / 5 | 3 / 5 (`230` / `215`) | 4 / 5 (`36` / `52`) | 2 / 5 (flip order) | 2 / 5 (silent flip WA) |
| **B: Bundled `Hld` / `LazyHld` wrappers** | 4 / 5 | 5 / 5 (`108` / `185`) | 5 / 5 (`19` / `40`) | 3 / 5 (wrapper matrix) | 4 / 5 (2× memory cost) |
| **C: `Rooted.chains` + generic `fold_path`/`apply_path` + `Reversible`** | **5 / 5** | **4 / 5 (`135` / `241`)** | **5 / 5 (`23` / `52`)** | **5 / 5** | **5 / 5** |

**Winner:** **Variant C** — keep `Rooted<V>` as the single tree abstraction
with `chains(a, b, edges?: Bool)` plus generic `fold_path` / `apply_path`
methods working over any `tree.Pre`-indexed DS (using `Reversible` for O(1)
single-tree non-commutative path folds). Retire the dual-tree `Hld` wrapper and
do not add `LazyHld`.  
**Losers:** **Variant B** (wrapper proliferation + 2× memory) and **Variant A**
(raw `chains` with no `fold_path` helper).

---

## Concrete Recommendations for `decisions.md` and the Synthesis

1. **Revise `decisions.md` §1.6 & `synthesis.md` §2.3B (`Str`):** Change
   default `Str` from nominal auto-opening `[domain]Char` to immutable
   `0..len`-indexed ASCII `[0..len]Char` with `s[i]`, `s[l..r]`,
   `s.starts_with(x, at: i)`, `for c in s`, and `s + t`, keeping `[domain
   S]Char = read()` as an explicit opt-in for multi-string DP.
2. **Prune `synthesis.md` §2.2 (Sliding windows):** Replace the `Window` trait,
   three-closure `slide_until(add:, drop:, until:)`, and callback `MonoDeque`
   with plain imperative `Deque<T>` and `MonoDeque<T>` structs plus stateless
   `slide_until` yielding non-empty `Range1` (with non-optional `.first`/`.last`).
3. **Sharpen `synthesis.md` §2.3B (`std.grid`):** Name 4- and 8-neighbor
   iterators `p.adj4` and `p.adj8` explicitly instead of an unlabelled
   `p.around()` that silently defaults to 4.
4. **Upgrade type inference before adding turbofish noise (Fork 2):** Unify
   non-closure arguments (`from:`, `root:`, `g`) before checking closure
   parameters and put `from:`/`root:` first in `std.graph`/`std.tree`, reserving
   `f::<T>()` for the ~10% of calls where no argument determines `T`.
5. **Offer free-function `bsearch_first` / `bsearch_last` / `bsearch_first_some`
   (Fork 5):** Eliminates `(lo..=hi).bsearch_first(...)` parens and precedence
   mistakes while retiring `first_where` completely.
6. **Drop the `stress ... by ...` sublanguage (Fork 6):** Keep `assume` +
   typed `read(gen:)` (with `witness` and coupled tuple reads) + whole-input
   `@gen` fallback, and fix `ioi07p2.cp`'s `m` / `walls` coupling.
7. **Replace dual-tree `Hld` / `LazyHld` with `Rooted.fold_path` / `apply_path`
   + `Reversible` (Fork 7):** Cuts memory and update time in half (1× instead of
   2×) and works uniformly across `SegTree`, `LazySegTree`, and `Fenwick`.
