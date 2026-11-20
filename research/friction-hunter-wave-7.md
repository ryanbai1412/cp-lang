# Friction Hunter — Wave 7 (30 hard ports, run-vs-C++ divergence hunt)

Base: `main` at `ef6036e`. Read-only audit. Every finding was checked on HEAD
with `target/release/cpl run`. On top of the line-by-line read, every port with
a scout generator was run differentially: the same random inputs through
`cpl run` (debug interpreter) and `cpl run --release` (the emitted C++17),
comparing output and exit status. None of these 30 ports appear in waves 1–6.

Labels as in earlier waves: **[WA-TRAP]**, **[REAL GAP]**, **[STALE PORT]**,
**[REJECTED]**.

## Ports (30)

- **Heavy data structures:** `scout_batch_43/etsr23r1p4` (segment tree beats),
  `scout_batch_21/ioi08p6` (two sweep trees), `scout_batch_30/noi19p4`
  (merge-sort tree + Dijkstra), `scout_batch_7/cco17p3` (2D binary trie),
  `scout_batch_7/noi04p1` (Fenwick k-th), `scout_batch_7/coci20c6p5` (Mo),
  `scout_batch_41/nccc9s4` (sqrt mode queries), `scout_batch_15/btoi20p3`
  (rollback DSU divide and conquer).
- **Strings / automata:** `scout_batch_22/noi18p3` (suffix automaton +
  persistent segtree), `scout_batch_30/ioi99p2`, `scout_batch_7/noi03p2`
  (rope editor), `scout_batch_23/noi09p5`.
- **Graphs / trees:** `scout_batch_27/ioi18p3` (Kruskal reconstruction trees),
  `scout_batch_2/noip18p3`, `scout_batch_26/noi20p2` (long-path merging),
  `scout_batch_30/cco22p3`, `scout_batch_23/noi07p1`, `scout_batch_27/cco19p2`,
  `scout_batch_12/wossoly21s3`.
- **Math / matrices / DP:** `scout_batch_23/noi19p5`, `scout_batch_11/noi20p1`
  (max-plus matrix power), `scout_batch_11/noi07p5` (determinant +
  recurrence guessing), `scout_batch_20/noi02p4`, `scout_batch_30/noi06p1`,
  `scout_batch_30/noi06p3`.
- **Geometry / bitsets / search:** `scout_batch_21/nccc10s4`,
  `scout_batch_29/ioi09p2`, `scout_batch_20/btoi19p2` (word bitsets),
  `scout_batch_12/noip09p4` (sudoku with `Subset`).
- **Interactive:** `scout_batch_34/dmopc20c6p6`.

## Averages

- **Real gaps:** 20 across the 30 ports, so `real_gaps / 30` = **0.67**
  (wave 6: 0.43).
- **Awkward lines:** 369 across the 30 ports, so `total_awkward_lines / 30` =
  **12.30** (wave 6: 9.60). This wave picked the longest unaudited ports
  (49–136 lines); about 80% of the awkward lines already have a clean form.
- **WA traps:** **1 new** (in the toolchain: `cpl run --release` skips
  `@interactor`). Both wave 6 traps still reproduce on this HEAD.
- **Cumulative, 210 ports:** 132 real gaps and 1492 awkward lines, so
  **0.63** real gaps and **7.10** awkward lines per problem.

Per-port awkward lines / real gaps: noi19p4 30/1, noi18p3 22/1, ioi08p6 30/2,
etsr23r1p4 12/1, dmopc20c6p6 6/0, nccc10s4 9/0, cco22p3 16/0, noi19p5 8/0,
ioi18p3 12/1, noi06p3 14/2, noip18p3 16/0, nccc9s4 5/0, noip09p4 8/1,
btoi20p3 7/1, coci20c6p5 12/1, btoi19p2 20/0, cco17p3 8/1, noi20p2 8/0,
noi04p1 8/1, ioi99p2 16/1, noi07p1 14/0, noi20p1 14/1, ioi09p2 4/0,
noi06p1 8/0, noi02p4 16/0, noi03p2 6/1, wossoly21s3 10/2, noi07p5 6/0,
cco19p2 10/1, noi09p5 14/1.

## Divergence hunt: `cpl run` vs emitted C++17

**Differential runs:** 15 of the 30 ports, 15 random inputs each from their
scout generators (225 runs). 14 ports agree on every input. One diverges on
13 of 15 inputs: the interactive `dmopc20c6p6`, which is WA trap 1 below. The other
15 ports were read but not run differentially: the debug interpreter was too
slow on their generators in the time box.

### [WA-TRAP] 1. `cpl run --release` ignores `@interactor`

`cpl run file.cp < hidden.txt` runs the solution against its `@interactor`
and prints `the interactor judge accepted the run`. `cpl run --release` on the
same file and input runs the C++ build with the hidden test piped straight in
as if it were the judge's replies. The solution then prints a wrong answer,
for example `A 14 2 3 0` instead of `A 2 5 3 4`, and exits 0 without warning.
**Fix:** `--release` uses the same interactor path, running the interactor in
the interpreter and the solution as the C++ binary, or refuses with a clear
error. Add `dmopc20c6p6` to the run-vs-release differential corpus.

**Targeted probes**, each run both ways, all agree:

```cp
print("{0.0625:.3f} {2.5:.0f} {0.125:.2f} {1.0005:.3f} {-0.0004:.3f}")  // 0.062 2 0.12 1.000 0.000
print(1.0 / 3.0)                  // 0.333333
print(1e300 * 1e10)               // inf
print(-7 / 2, -7 % 2, 7 / -2, 7 % -2)   // -4 1 -3 1  (Euclidean everywhere)
```

Float formatting matches, including round-half-even ties and `-0.000`
normalised to `0.000`.

**One difference that is by design:** `(1 << 63) - 1` stops with an overflow
trap in `cpl run`; the release C++ prints `9223372036854775807`. That matches
the approved rule that unproven overflow traps are local-only. It is listed
here so nobody files it as a regression. A `cpl lint` note on constant
expressions that overflow at check time would still be cheap and useful.

## [REAL GAP] findings

### New (not in waves 1–6 or `friction-queue.md`)

1. **`LazySegTree` has no "descend while" hook** for segment-tree-beats
   operations (range divide, chmin, chmax, sqrt). `etsr23r1p4` hand-writes a
   60-line `BeatsTree` because `apply` must be allowed to fail on a node and
   push down. Minimal shape: an optional `Action.try_apply(f, node, len) -> ?T`
   that returns `none` to make the tree recurse.
2. **`Char` is not an index domain and `Char` ranges don't work with `in`.**
   `var c: [Char]Int = 0` fails ("doesn't know the name `Char`"), so
   `ioi99p2` indexes `[0..256]` through `Int(w[W(j)])`; and
   `ch in 'a'..='z'` fails ("expected `Int`, found `Char`"), so
   `wossoly21s3` writes `'a' <= ch and ch <= 'z'`. A `domain Letter = 'a'..='z'`
   already works, so this is the same rule applied to `Char` itself.
3. **No `List.insert(i, x)` / `remove(i)`.** `noi03p2` splices with
   `pieces = pieces[0..i] + [text] + pieces[i..pieces.len]` three times.
4. **`Subset<D>` has no `set(d, on)`.** `noip09p4` writes
   `if on: s.with(d) else: s.without(d)` three times.

### Recurring (counted again)

- **Sub-domain / sum / product domain embedding** (the largest unowned
  item): `noi19p4` packs cities and portals into one `0..n + m` index;
  `noi20p1` packs `(City, wait)` as `State(c.to_idx * 5 + wait)`; `cco19p2`
  packs `(Row, Col)` plus an extra floor node into `Cell(r * m + c)`;
  `wossoly21s3` can't call `bfs` on `(Row, Col)` states, so it writes the
  worklist by hand.
- **Label crossings with `Int` cursors:** `ioi18p3` (`c.to_idx >= l`),
  `btoi20p3` (`edges[E(idx)]` in divide and conquer), `coci20c6p5` (Mo
  cursors through `Paper.nth(hi)`, `Count(h)`).
- **`D.window(lo, hi)` / scalar `D.clamp(x)`:** `ioi08p6`
  (`Y(max(b.y0 - k + 1, 1))` ×3), `noi06p3`
  (`max(0, i - 2)..=min(n - 1, i + 2)` + `Col(j)`), `noi09p5` (10 `I(..)` wraps
  because its loops run over `lo..=hi` ints).
- **Position-aware `Action.apply`:** `ioi08p6` hand-writes a 40-line
  max-free-run tree because each node needs its real width.
- **Node arena with returned ids** (wave 6's `Trie` item): `cco17p3` and
  `noi18p3` both write `push(..); xs.len - 1`.
- **`Fenwick.kth`:** `noi04p1` writes `lower_bound(k)!.left!` twice.
- **`scan` not in the prelude:** `noi06p3` writes prefix-min and suffix-min
  loops by hand.
- **`Char.lower`:** `wossoly21s3` writes `place[ch + 32]`.

## [STALE PORT] patterns (clean form verified on HEAD)

- **Sentinels instead of optionals:** `const NONE = -4e18` with `> NONE` ×6
  (`noi20p1`), `-1` in `ioi99p2` ×3, `Int.MAX` in `ioi08p6`. `?Int` with
  `max=`/`min=` covers all of them.
- **`prev == none or d < prev!`** before a push (`noi19p4`, `cco22p3`):
  `if dist[u] min= d:`.
- **`let d = top.0; let u = top.1; ...`** after `pq.pop()` (`noi19p4`,
  `cco22p3` ×4): `while let (d, u, loc, other) = pq.pop()`.
- **Hand binary searches** (`noi19p4`, `ioi08p6`, `cco22p3`, `noip18p3` ×2):
  `bsearch_first` / `bsearch_last`.
- **Field-by-field reads and read-and-push loops** (`noi19p4` 8 single reads,
  `ioi08p6`, `nccc10s4`, `ioi18p3`, `noi06p3`): `[n](x: Int, y: Int) = read()`.
- **Word bitsets by hand** (`btoi19p2`, 20 lines plus a hand `popcount`):
  `Bitset<D>` has `shifted`, `bitand` and `len`, and `popcount` is built in.
- **`row = g[h]; g[h] = []`** (`noi20p2`): `g[h].take()` works.
- **Single-field `(to: City)` tuples to feed `Rooted`** (`ioi18p3`): `Rooted`
  accepts a `List<V>` adjacency.
- **Hand `ext_gcd`** (`noi02p4`, 14 lines): `std.math.ext_gcd` and `crt`.
- **`let (_, _): (Int, Int) = read()`** is fine; `let _: (Int, Int) = read()`
  also works.
- **`Rooted(u => g.adj(u), r)`, `if tree.parent(u) == v`** (`noip18p3`,
  `noi20p2`): `Rooted(g, r)`, `tree.lower(u, v)`.
- **`Mod` literal wrapping** (`noi19p5`, `noi09p5`, `noi07p5`, `noi20p2`):
  typed declarations and `Mod * Int`.
- **`.bit(k)` over shift-and-mask** (`noi20p1`, `noi06p1`, `cco17p3`),
  **`.last!`** over `xs[xs.len - 1]` (`noi20p1`), **`??=`** over
  `if x == none: x = ..` (`cco17p3`), **`Int(cond)`**, **dedupe and Map
  id-assignment by hand** (`noi06p3`, `nccc10s4`, `noi04p1` → `compress`).
- **Redundant prelude imports** in 9 of the 30 files.

## [REJECTED]

- Mo's algorithm, sqrt decomposition, skip-pointer DSU over a sorted list,
  and suffix automaton helpers: algorithms, not language gaps.
- Reading an upper-triangular matrix (`noi06p1`): one port.
- A "max with tie-break" assign for `ioi09p2`'s rational comparison: a tuple
  key with `max=` covers it once `Ord` is implemented.

## Proposed Syntax Batch #7 additions

Wave 6's six items still stand, starting with its two WA traps.

0. **[WA-TRAP] `--release` with `@interactor`:** same interactor path as `cpl run`, or a hard error.
1. **`Char` as a domain:** `[Char]T` arrays and `ch in 'a'..='z'`, from the
   same `Index` rule that `domain Letter = 'a'..='z'` already uses.
2. **`List.insert(i, x)` / `remove(i) -> T`:** two userland methods in
   `std.seq`.
3. **`Subset.set(d, on)`:** one line in the `Subset` impl.
4. **`Action.try_apply` beats hook:** an optional field on `Action`; when it is
   set, `LazySegTree` recurses on `none`. One structure covers range
   chmin/chmax/div/sqrt.
5. **Sum and product domains** (carried since wave 4): `noi19p4`, `noi20p1`,
   `cco19p2` and `wossoly21s3` add four more ports to the case. This is now
   the biggest single source of label crossings in the audit.
6. **`cpl lint` for overflowing constant expressions** like `(1 << 63) - 1`,
   so the by-design run/release difference never surprises anyone.

## Proposed `friction_lint.py` rules

- `const \w+ = -?\d+e1[5-9]` used as a sentinel → `?T`.
- `== none or .* < \w+!` → `min=` / `max=`.
- `let \w+ = top\.0` → tuple destructuring in `while let`.
- `xs = xs\[0\.\.i\] \+ \[` → `insert`.
- `if \w+: \w+\.with\(.*\) else: \w+\.without\(` → `Subset.set`.
- `\[0\.\.256\]` or `Int\(\w+\[.*\]\)\]` indexing by a char → `[Char]`.
