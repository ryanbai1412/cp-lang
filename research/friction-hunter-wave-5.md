# Friction Hunter, wave 5 (post-Syntax-batch-4)

Wave 5 audits 30 more solved ports that waves 1 to 4 did not cover, on HEAD
`812549e` (Syntax batch 4 `bcd873b`, plus `b59851a`). Every port checks clean
with `cpl check`. Every `[REAL GAP]` and `[WA-TRAP]` below was reproduced on
HEAD with a small `cpl run` program. Each `[STALE PORT]` was also checked: the
cleaner form compiles and runs on HEAD.

Tags:

- `[WA-TRAP]`: the program compiles, but it can silently print a wrong answer.
- `[REAL GAP]`: there is no clean way to write it on HEAD.
- `[STALE PORT]`: HEAD already has a clean form; only the port needs a rewrite.
- `[REJECTED]`: a one-off or the algorithm itself, not language friction.

## The 30 ports

The picks are biased toward hard, unaudited problems. They come from 23 scout
batches plus 3 ecnerwala library files, all between 25 and 90 lines.

- **Data structures:** `noi22p1`, `ioi21p1`, `shoi15p2`, `ioi22p4`,
  `sdoi17r1p6`, `ioi12p6`, `ioi14p6io`
- **Graphs and trees:** `ioi17p3`, `ioi11p1`, `noi10p3`, `noip19p6`, `ioi09p7`,
  `noi09p1`, `ioi13p1`, `noi22p4`
- **Geometry and sweeps:** `noi23p1`, `ioi07p5`, `ccc14s4`, `ioi04p2hard`
- **DP and greedy:** `rpeng3`, `noi20p4`, `cco22p2`, `noip22p2`
- **Constructive:** `noip20p3`, `egoi22p8`
- **Strings and parsing:** `noi03p1`, `ccc99s5`
- **Math libraries:** `ecnerwala/math/floor_sum`, `hom`, `frac`

## Averages

- **Real gaps:** 17 across the 30 ports, so `real_gaps / 30` = **0.57**.
  - Wave 4 was 0.67, and the 120-port cumulative was 0.68.
  - 7 of the 17 are new. The other 10 are known gaps that recur: worklist ×2,
    domain/`Int` label crossings ×5, `read()` of `?T` ×1, payload enum or sum
    domain ×1, and `Str` split ×1.
- **Awkward lines:** 329 across the 30 ports, so `total_awkward_lines / 30` =
  **10.97**.
  - Wave 4 was 3.47. The jump is mostly three ports that hand-roll a segment
    tree (`shoi15p2`, `ioi21p1`, `ioi22p4`, 75 lines between them), plus long
    read-and-push loops in older scout ports.
  - Without those three ports, it is 254 lines over 27 ports, or 9.4 per port.
- **WA traps:** 0 confirmed inside ports. There is 1 design-level candidate
  (below).
- **Cumulative over 150 ports:** 99 real gaps and 835 awkward lines, so **0.66**
  real gaps and **5.57** awkward lines per problem.

The real-gap rate is flat. Syntax batch 4 cleared the gaps it targeted, but
hard ports keep surfacing new library-shape gaps (position-aware actions,
product-state graph search, key-grouped runs). Most of the awkward lines are
stale: 84% of them already have a clean form on HEAD.

## [WA-TRAP] candidate

**`print(Bool)` writes `YES`/`NO`.** This is documented in
`site/content/io-and-formatting.md`, but AtCoder (and `noi22p4`'s own judge)
want `Yes`/`No`. A competitor who writes `print(ok)` on AtCoder gets a silent
WA on every case.

```cp
print(true, false)       // YES NO
print([true, false])     // YES NO
```

Proposal: keep the convention, and make `cpl lint` warn on `print` of a `Bool`
expression when the problem file's samples contain `Yes` or `No` (the
`cpl new` scaffold already knows the samples). Ports in this wave work around
it with `Int(cond)` (`ioi17p3`) or explicit strings (`noi22p4`, `cco22p2`).

## New real gaps (7)

### 1. The prelude leaves out names its own docs use

`desc`, `scan`, `fold` and `ceil_div` are not in the prelude. `stdlib/algebra.cp`
documents `sort(mut es, by: e => (e.w, desc(e.name)))`, and `decisions.md` §4.3
lists `sort(..., by: desc)`. The prelude imports `Desc`, but not `desc`.

```cp
print(sorted([3, 1, 2], by: desc))      // error: no `desc` in scope
print([0] + scan(xs, +))                // error: no `scan` in scope
print(ceil_div(7, 2))                   // error: no `ceil_div` in scope
```

`ioi14p6io` imports `desc`, `scan` and `reverse` by hand, and `frac` writes
`-((-num) / den)` instead of `ceil_div`.

**Correction to waves 3 and 4:** they listed `sorted(xs, by: desc)` as the
stale-port rewrite for `by: x => -x` (`ioi08p4`, `cco17p4`). That rewrite fails
without `use std.algebra.{desc}`.

### 2. `runs` can't group by a key

`runs(xs)` groups equal neighbours only. Grouping by a field takes a hand-written
`i`/`j` loop (`rpeng3`: fairs at the same time) or a hand merge (`noi23p1`:
strokes on the same line).

```cp
var i = 0
while i < fairs.len:
    var j = i
    while j < fairs.len and fairs[j].t == fairs[i].t: j += 1
    ...
    i = j
```

Repro: `runs(xs, by: p => p.0)` fails with "`runs` has no parameter named `by`".
The proposal is `runs(xs, by: key) -> List<(key: K, items: List<T>)>`, the same
`by:` that `sorted`, `max_by` and `compress` already take.

### 3. Exclusive folds work only on `Rooted`

`Rooted.excl_fold` gives every child the fold of its siblings. A plain list has
no such form, so `ioi22p4` builds prefix and suffix product arrays by hand (9
lines) to get "the product of everyone but me".

```cp
var after: List<Int> = []
for _ in 0..=c: after.push(1)
for i in (0..c).rev: after[i] = after[i + 1] * all[kids[u][i]] % MOD
var before = worth[u]
for i in 0..c:
    worth[kids[u][i]] = before * after[i + 1] % MOD
    before = before * all[kids[u][i]] % MOD
```

Repro: `excl(xs, *, 1)` is "no `excl` in scope". The proposal is
`xs.excl_fold(m) -> yields (x: T, rest: T)` in `std.seq`, with
`Rooted.excl_fold` rebuilt on top of it, so there is one name for one concept.

### 4. `Action.apply` doesn't know where the segment is

`apply` gets `(f, x, len)`, so an update that depends on position, such as
"set `x[i] = v + i`" or "add an arithmetic progression", has to carry the
segment's left end in the node. `sdoi17r1p6` adds a `lo` field and a `len` field
to its `Sums` record, and rebuilds the whole 6-field record in `join` just to
keep them.

Repro: `Action<Int, Int>(0, +, (f, x, len, at) => .., over: ..)` is rejected:
"expected `(Int, Int, Int) -> Int`". The proposal is to pass the segment's
`Range<D>` instead of `len` (`r.len` still gives the length). That is one general
signature, and no second kind of action.

### 5. No way to clamp to a domain

Clipping a query window to the domain's ends is spelled out with `max`/`min`
and the domain's literal bounds (`ioi07p5`, three times):

```cp
near.sum(V(max(a.v - d, 1 - m))..=V(min(a.v + d, m - 1)))
```

Repro: `V.clamp(10)` and `(-10..=1) & V` both fail. The proposal is
`D.clamp(x: Int) -> D` and `D.window(lo: Int, hi: Int) -> Range<D>`, an
intersection that can be empty and is never reversed. That also closes the
slice-bounds lesson from wave 2.

### 6. Graph search over product states doesn't work

`bfs` over a state like `(fountain, arrived_by_best)` fails inside the stdlib
generic, reported at 1:1:

```cp
var back: [F, Bool]List<(F, Bool)> = []
let d = bfs(s => back[s], [(F(1), true)])   // error: doesn't know the name `V` (1:1)
```

So `ioi11p1` packs the state into one domain by hand:
`State(2 * f.to_idx + (if by_best: 1 else: 0))`, `F(s.to_idx / 2)`,
`s.to_idx % 2 == 1`. That is 5 lines of crossings.

`[F, Bool]T` arrays already work (`seen[F(1), true]`), and so does indexing a
2D array by a tuple (`dist[s]` with `s: (A, B)`). The fix is vertex-domain
inference for tuple vertices in `bfs`, `dijkstra`, `topo` and `peel`, so the
`[A, B]` and `[(A, B)]` shapes unify.

### 7. `Fenwick.lower_bound` returns a bound, not an element

Finding the k-th live element (`ioi12p6`, twice) reads as a double unwrap:

```cp
let first = alive.lower_bound(s + 1)!.left!
```

The proposal is `Fenwick.kth(k: Int) -> ?D`, the element whose prefix first
reaches `k`. `lower_bound` stays for the bound form.

## Known gaps that came up again (10)

- **Worklist** (wave 3): `ioi17p3` has two `var head = 0; while head < q.len`
  loops, and `noi09p1` one. `for x in q` with `q.push` is still E0602.
  `while let x = q.get(i)` works, but still needs the counter.
- **Domain/`Int` label crossings** (wave 4, 477 lines): `cco22p2` crosses
  between `Mkt = 0..n-1` and `Bus = 0..n` with `Bus(i + 1)`, `Mkt(j)` 10 times.
  Also `noip19p6` (`Size(..)` ×8), `noip20p3` (`P(i)`, `u.to_idx + 1`) and
  `ioi07p5`.
- **`read()` of `?T`** (Batch #12): `noi22p4` reads a parent as `-1..=n` and
  branches on `-1`.
- **Payload enum or sum domain** (wave 1): `ioi12p6` packs leaves and rounds
  into `Node = 0..n + c` and tests `v.to_idx < n`.
- **`Str` split** (friction queue L16): `noi03p1` copies characters up to `#`
  with a push loop. `first(i for i in 0..s.len if s[i] == '#')` is the best
  form on HEAD.

## Stale-port findings (the clean form already works on HEAD)

Each of these was checked on HEAD.

- **Hand-rolled segment trees, 3 ports, about 75 lines.**
  - `shoi15p2`: an assign-lazy tree with a `tag = -1` sentinel. This is
    `LazySegTree` with `Action<Run, ?Bool>`.
  - `ioi22p4`: an XOR-toggle tree with three parallel arrays. This is
    `LazySegTree` with a `Bool` flip action.
  - `ioi21p1`: three parallel arrays and a stateful right-to-left descent. This
    is `SegTree.min_left(to, ok)`.
- **Hand-written binary search, 3 ports.** `noi23p1` and `ioi09p7` are
  `xs.first_ge(x)`; `ccc14s4` is `compress(..)`. `[1, 4, 9].first_ge(5)` gives
  `2`.
- **Read-and-push loops, 9 ports.**
  - Arrays of records read directly: `let ends: [Trail](a: F, b: F) = read()`.
  - Struct reads work too: `let panes: [n]Pane = read()` (checked with input).
  - `let xs: [n]Int = read()` replaces `for _ in 0..n: let x = read(); xs.push(x)`.
- **Fill-by-push loops, 6 ports.** `[0..n]List<Int> = []`, `[0..k]Int = 1` and
  `List.filled(n, false)` work. `List.filled(3, List<Int>())` does not alias
  (checked).
- **Redundant `use` lines, 9 ports.** `Fenwick`, `Rooted`, `Graph`, `bfs`,
  `LazySegTree`, `Action`, `Monoid`, `bsearch_first`, `gcd` and `Reversible`
  are all in the prelude.
- **Tuples:** `dist[s.0, s.1, s.2]` ×5 in `noi10p3` is `dist[s]`. Tuple-indexing
  a 3D array works.
- **Optionals:**
  - `if dist[s] == none or d < dist[s]!` is `if dist[s] min= d:`.
  - `best == none or v < best!` is `best min= v`.
  - `big[r1] == none: big[r1] = ..` is `big[r1] ??= ..`.
- **Sentinels:** `NEG = -1e18` in `rpeng3` is not needed. `?Int` arithmetic
  lifts (`none + 1` is `none`), and `max=` skips `none`.
- **Lists:**
  - Three parallel stacks in `cco22p2` are one `List<(at, cnt, pref)>` with
    `st.last!.cnt -= take`.
  - `pre[pre.len - 1]` is `pre.last!`.
  - `0..x + 1` is `0..=x`.
- **Maps:** `if names.get(k) == none: names[k] = names.len` is
  `names.get_or_insert(k, names.len)`.
- **Loops and printing:**
  - `for by_best in [false, true]` is `for by_best in Bool`.
  - `if r > inner[v]: 1 else: 0` is `Int(r > inner[v])`.
  - `noi09p1`'s 4-line string join is `print([ans[i]!.to_idx for i in Pos])`.
- **Characters:** `eq[at].ord - '0'.ord` is `eq[at] - '0'`, and
  `Char(Int('0') + d)` is `'0' + d`.
- **Domains:**
  - `[0..=n, O]Bool` with a runtime-negative `O = mn..=mx` works, so
    `noi20p4`'s flat `reach[i * W + o]` table is stale.
  - `List(read_rows)` turns a `[Row]Str` into a list (`ccc99s5`).
- **Library helpers:**
  - `Rooted.from_parents` replaces the hand-built `kids` lists in `ioi09p7`.
  - `Prefix2(tint).sum(..=i, ..=j)` replaces the 2D prefix-sum `if let up =
    i.prev` nest in `ccc14s4`.
  - `max_by(walks, w => w.dist)!.to` is the `// WISH` in `ioi13p1`.
  - `Mod<1000002022>` works for a non-prime modulus, so `ioi22p4`'s `% MOD` is
    stale.
  - `hom`'s `compose(one, zero)` can use `T.one` and `T.zero`, which work in
    generic code.
- **`floor_sum`:** the 3-line temp swap is `(den, slope) = (slope, den)`.

## Rejected

- **Last-tie `argmax`** (`noi20p4`, `ioi12p6`): `max_by((0..n).rev, ..)` already
  gives the last maximum.
- **Fixed-width most-significant-first digits** (`egoi22p8`): one problem.
  `x.digits(128)` exists.
- **28-bit rotation** (`egoi22p8`): one problem.
- **Destructuring a 2-element list** as `let (lo, hi) = sorted([u, v])`: a list
  is not a tuple.
- **`sign(x)`:** one use.
- **The `impossible`/`ok` flags** (`noi09p1`, `cco22p2`): top-level `return`
  works on HEAD (checked inside a `for`), so these are port style, not a gap.
- **The linked-list store** in `noi22p1`: that is the algorithm.

## Proposed Syntax Batch #6

These are ordered by how many ports they unblock. All of them are general rules
or extensions of existing stdlib names, with no new one-off helpers.

1. **`print(Bool)` lint.** In `cpl lint`, warn when a `Bool` is printed and the
   samples say `Yes`/`No`. Keep `YES`/`NO` as the default. Add a
   site/course callout.
2. **Prelude coherence gate.** Export `desc`, `scan`, `fold` and `ceil_div`.
   Add a `cpl test` that compiles every stdlib `///` example with no `use`
   lines, so a documented name can never be missing from the prelude again.
3. **Graph search over product states.** Tuple vertices of finite domains in
   `bfs`, `dijkstra`, `topo` and `peel`, with one array shape for `[A, B]T` and
   `[(A, B)]T`. Also fix the 1:1 diagnostic inside stdlib generics (waves 3 and
   4).
4. **`Action.apply(f, x, r: Range<D>)`.** Pass the segment's range instead of
   `len`, and migrate `AddSum`, `AssignSum` and the others to `r.len`.
5. **`runs(xs, by: key)`**, returning `(key, items)`.
6. **`xs.excl_fold(m)`** on every sequence, with `Rooted.excl_fold` built on it.
7. **`D.clamp(x)` and `D.window(lo, hi) -> Range<D>`**, empty-safe.
8. **`Fenwick.kth(k) -> ?D`.**

Carried from earlier waves and still the biggest unowned items: `Worklist<T>`
(wave 3) and sub-domain embedding for label crossings (wave 4).

## Proposed `scripts/friction_lint.py` rules

- `hand_segtree`: `\b2 \* o \+ 1\b` together with `fn \w+\(o: Int, l: Int, r: Int`.
  This is a stale port for `SegTree`/`LazySegTree`.
- `hand_bsearch`: `while lo < hi:` followed within 3 lines by `let mid = (lo + hi) / 2`
  on a list. Use `first_ge`, `bsearch_first` or `compress`.
- `read_push_loop`: `let (\w+): \S+ = read\(\)` followed by `\w+\.push\(\1\)` on
  the next line. Read the array directly.
- `fill_push_loop`: `for _ in \S+: \w+\.push\((0|1|-1|false|true|\[\])\)`. Use a
  filled array.
- `tuple_spelled_index`: `\[(\w+)\.0, \1\.1(, \1\.2)?\]`. Index with the tuple.
- `desc_by_negation`: `by: \w+ => -\w+`. Use `by: desc` (once it is in the
  prelude).
- `big_sentinel`: `= -?1(000){5,}\b|= -?1e1[5-8]`. Use `?Int` with `min=`/`max=`.
