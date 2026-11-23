# Friction Backlog — Scout Batch 41: Hard Olympiad & NCCC Classics (`oly*`, `nccc*`)

Ten previously unported corpus problems from the strict `oly*`, `cerc*`, `ecna*`, `icpc*`, `naq*`, `nac*`, `waterloo*`, `mcipc*`, `utso*`, `nccc*` prefix bucket were rethought in CP Lang under `proto/ports/scout_batch_41/` against the locked **CP Lang v2** spec (`docs/one-page-cheat-sheet.md`, commit `3723dca`): `oly20practice139`, `oly23practice5`, `nccc9s4`, `nccc9s1`, `nccc9s2`, `nccc9s3`, `nccc10s1`, `nccc8s1`, `nccc8s2`, and `nccc8s4`.

`corpus/submissions/info.json` contains zero problems for `cerc*`, `ecna*`, `icpc*`, `naq*`, `nac*`, `waterloo*`, and `mcipc*`, and its sole `utso*` problem (`utso15p5`) is already ported. Across `oly*` and `nccc*`, these ten are the complete set of unported accepted submissions in the corpus (seven C++17 and three Java).

Each problem has a v2-spec file (`<problem>.cp`) and a runnable file (`today/<problem>.cp`). The eight `proto/cpc` runnable files compile to C++17 with ASan + UBSan and agree with the original submissions on 300 random inputs each (200 small, 100 medium; 2,400 total). The two unknown-length `Str` problems (`nccc8s1`, `nccc8s2`) run under `proto/cp.py lib-run` and agree with the original Java submissions on 60 random inputs each (40 small, 20 medium; 120 total, with line-by-line comparison preserving empty lines for `nccc8s1`). All 2,520 cases pass. All ten were written from the accepted corpus code rather than statements.

## Dual LOC

Counted by `proto/ports/scout_batch_41/loc.py` (non-blank, non-comment lines). "Stripped" removes the C++ contest template through `#define sz(x)`, stream setup lines, and the `namespace io` fast-`fread` reader in `nccc9s4`, and removes imports, `static class Input`, and `main` boilerplate in the three Java originals.

```
problem              raw stripped   v2 today
oly20practice139     167      142   55    91
oly23practice5       134      109   23    42
nccc9s4              166      123   53    95
nccc9s1               63       32    4     8
nccc9s2               58       27    4     9
nccc9s3               56       25    6     9
nccc10s1              56       28    9    25
nccc8s1               55       32   11    32
nccc8s2               37       13    7     9
nccc8s4               68       44   13    26
total                860      575  185   346
```

The v2 ports cut raw LOC by 78% and template-stripped LOC by 68%. For extra fairness, `6415646.cpp` (`oly23practice5`) also leaves behind 41 lines of unused debug `operator<<` and a dead brute-force `solve(xa, xb, arr)` helper; discounting those 41 lines leaves 534 live stripped lines across the batch, against which v2's 185 lines is still a 65% reduction. The runnable `today/` ports are 60% shorter than raw and 40% shorter than stripped — the 161-line gap between v2 (185) and `today/` (346) is overwhelmingly driven by missing `Str`, `compress.at`, `Subset`, `struct` defaults, and `for ... if` support in `proto/cpc`.

On `oly23practice5`, rethinking rather than translating saved the biggest chunk: sorting `0..=n` once by `-odd[y]` and finding the first non-conflicting `y` via `first(y for y in by_odd if y != x and (x, y) not in pen)` eliminates the original's conflict adjacency list, `multiset`, and per-vertex erase/reinsert loop while keeping `O(n log n)` total work.

## 1. Remaining friction and multi-feature collisions in the locked v2 spec

These issues remain even if every construct on `docs/one-page-cheat-sheet.md` is implemented verbatim.

### 1.1 No mutable iteration (`for mut x in xs`) for containers of self-adjusting structs

**Hit in:** `oly20practice139.cp`.

This is a three-way collision between **lexical `mut` borrows**, **amortized data structures whose query mutates internal state**, and **collection iteration / generator reductions**. In `oly20practice139`, each sqrt block owns a monotonic Convex Hull Trick whose `query(mut self)` advances `self.ptr`. Because `for b in blocks` binds `b` immutably and v2 has no `for mut b in blocks` syntax, neither a range-for loop nor a generator `max(b.query() for b in blocks)` can call `b.query()`. The port has to fall back to a raw index loop:

```cp
fn max_b() -> Int:
    var best = 0
    for i in 0..blocks.len: best max= blocks[i].query()
    best
```

The same trap hits any array or list of `Dsu`s (path compression), monotonic queues, splay trees, or treaps. Adding `for mut b in blocks:` (reusing the existing `mut` keyword without adding a new concept) fixes the loop case cleanly while keeping generator expressions pure.

### 1.2 Partitioning `Compressed<T>` or a domain into non-overlapping sqrt blocks

**Hit in:** `oly20practice139.cp`, `nccc9s4.cp`.

`docs/one-page-cheat-sheet.md` provides overlapping `D.windows(k)` and strided `(0..n).step(B)`, but no non-overlapping `.chunks(B)`. Furthermore, `Compressed<T>` exposes `xs.D`, `xs[v]`, and `xs.at(val)`, but no slice of its underlying sorted values. Splitting compressed `y`-coordinates into blocks of size `B = 400` in `oly20practice139` forces an awkward round-trip through `from_idx` and a manual `min` clamp:

```cp
let ys = compress(p.y for p in pts)
var blocks: List<Block> = []
for ql in (0..ys.D.K).step(B):
    let qr = min(ys.D.K, ql + B)
    var b = Block(ys: [ys[ys.D.from_idx(i)!] for i in ql..qr], v: List.filled(qr - ql, 0))
    b.rebuild()
    blocks.push(b)
```

Adding `D.chunks(k)` (yielding `Range<D>` sub-ranges, with the final chunk clamped to `D.last`) and allowing `ys[range]` to slice `Compressed<T>` turns that entire setup into `for r in ys.D.chunks(B):` with zero `.from_idx` or `min` clamping.

### 1.3 Empty-window cursors (`cr = mid - 1`) in Rollback Mo's collide with nominal domains

**Hit in:** `nccc9s4.cp`.

Rollback Mo's starts each bucket `bi` with an empty right window `[mid, mid - 1]` where `mid = bi * B` and grows `cr` rightward. For bucket `bi == 0`, `mid == 0`, so `cr == -1` sits one position before `N.first`. Under strict nominal `domain N = 1..=n`, `cr` cannot hold `-1`, and mixing `l: N` with `mid: Int` in `for j in l..mid` fails to typecheck. The port abandons `domain N` completely and drops to plain `0..n` `Int` indices everywhere, forfeiting zero-check array indexing on `a` and `id`:

```cp
let mid = bi * B
var cr = mid - 1
for (l, r, qi) in buckets[bi]:
    while cr < r:
        cr += 1
        let v = id[cr]
        ...
    for j in l..mid:
        ...
```

Boundaries (`N.bounds`) can represent the empty prefix cleanly (`var cr: N.bounds = ...`, stepping via `cr.right`), but `N.bounds` has no chunking or block-midpoint constructor, so contestants will keep dropping back to `Int` indices for Mo's and two-pointers unless `D.chunks(B)` yields boundary endpoints alongside ranges.

### 1.4 `defer` scope inside loops vs functions is unspecified

**Hit in:** `nccc9s4.cp`.

Rollback Mo's wants to snapshot and roll back temporary left-extension increments once **per query** inside the inner `for` loop:

```cp
for (l, r, qi) in buckets[bi]:
    ...
    for j in l..mid:
        let v = id[j]
        cnt[v] += 1
        cur max= (cnt[v], v)
    ans[qi] = vals[cur.1]
    for j in l..mid: cnt[id[j]] -= 1
```

`docs/one-page-cheat-sheet.md` §5 names `defer` as the sole scope-cleanup primitive (`let snap = dsu.mark(); defer dsu.undo_to(snap)`), but never says whether `defer` runs at **block exit** (end of the enclosing `for` iteration, like C++ RAII / Zig `defer`) or at **function exit** (like Go `defer`). If it is function-scoped, putting `defer` inside a loop silently accumulates `O(Q)` deferred rollbacks and returns wrong answers on query 2; if it is block-scoped, Rollback Mo's and backtracking can use `defer` directly inside loops. The spec should lock **block-scoped** `defer` explicitly on the cheat sheet.

### 1.5 Symmetric signed domains (`-26..=26`) and `topo` inference on neighbor closures

**Hit in:** `nccc8s4.cp`.

Two collisions appeared in the same 13-line tile-cycle problem:

First, tile labels naturally live in `-26..=26`, where `0` is a flat side and `x != 0` connects to `-x`. A nominal `domain Side = -26..=26` has no unary negation `-v` and does not compare against plain `0`, forcing the port to shift manually into `0..=52`:

```cp
domain Side = 0..=52
...
adj[Side.from_idx(tile[i] + 26)!, Side.from_idx(26 - tile[j])!] = true
```

Second, §11 states that unweighted graph algorithms accept either `Graph<V, E>` or a plain `V -> Iter<V>` closure, inferring `V` from `from:`, `root:`, or `g`. `topo` has none of those arguments, so passing a closure `u => (v for v in Side if adj[u, v])` gives inference no anchor for `V`. The port therefore materializes an intermediate edge list and `Graph.directed(edges)` just to ask whether a 53-vertex adjacency matrix has a cycle:

```cp
var edges: List<(Side, Side)> = []
for u in Side:
    for v in Side if adj[u, v]: edges.push((u, v))
let g = Graph.directed(edges)
print(if topo(g) == none: 1 else: 0)
```

Documenting `topo::<Side>(u => ...)` (or `topo(Side, u => ...)`) for graph algorithms that have no starting vertex (`topo`, `scc`, `two_color`, `articulation_points`) removes the throwaway edge list whenever the graph is already an adjacency matrix or implicit transition function.

### 1.6 Cheat-sheet omissions for everyday `Str`, `List`, and comprehension spellings

**Hit in:** `nccc8s1.cp`, `nccc9s2.cp`, `oly20practice139.cp`, `nccc9s4.cp`.

Four small everyday spellings had to be guessed or worked around because `docs/one-page-cheat-sheet.md` omits them:

- **`Str + Char`:** `nccc8s1` builds edit-distance-1 words by inserting or replacing a single `c: Char`. §4 lists `s + t` and `out.push(c)`, but not `Str + Char`, pushing the port to interpolation `"{s[..i]}{c}{s[i + 1..]}"` instead of `s[..i] + c + s[i + 1..]`.
- **List comprehensions:** `nccc9s2` writes `let pos = [i for (i, c) in s.items if c == 'I']`, and `oly20practice139` writes `[ys[ys.D.from_idx(i)!] for i in ql..qr]`. §3 documents array comprehensions `[V] v => expr` and §4/§10 show generator expressions inside calls, but the cheat sheet never states whether `[expr for x in it if cond]` is valid syntax for constructing a `List<T>`.
- **`List.filled(len, val)`:** Used inside `stdlib/bitset.cp` and needed by `oly20practice139` to initialize a dynamic `v: List<Int>` field inside `struct Block`, yet absent from §9.
- **Array of independent empty lists (`var buckets: [0..=nb]List<...> = []`):** `nccc9s4` relies on value semantics cloning a fresh empty `List` per slot; confirming that `var a: [D]List<T> = []` constructs `D.K` independent lists (not aliased handles) deserves one explicit mention in §3 or §9.

### 1.7 `Char`-range array dimensions and missing `prod(it)`

**Hit in:** `nccc8s2.cp`.

`nccc8s2` counts lowercase letters and multiplies `(cnt[c] + 1)` modulo `10^9 + 7`:

```cp
type Z = Mod<1_000_000_007>
let s: Str = read()
var cnt: ['a'..='z']Int = 0
for c in s: cnt[c] += 1
var ans: Z = 1
for x in cnt: ans *= x + 1
print(ans)
```

Two minor frictions stand out. First, §3 lists `lo..=hi` as an algebraic `Index` and §8 uses `'1'..='9'` in `@tags`, but §3 never clarifies whether `'a'..='z'` works directly as an array dimension indexed by `c: Char` (checked in debug, like `[0..n]Int` indexed by `Int`). Second, §10 lists `sum(it)`, `any(it)`, `all(it)`, and `count(it)` without `prod(it)` — even though modular product reductions are routine in combinatorics and `stdlib/algebra.cp` already defines `Prod`.

### 1.8 Iterating `Subset<D>` directly and `sorted` on a `Range`

**Hit in:** `nccc10s1.cp`, `oly23practice5.cp`.

In `nccc10s1`, every candidate answer key is a `Subset<Q>`, and Hamming distance is `(cand ^ m).len`:

```cp
print(max(min((cand ^ m).len for m in masks)! for cand in Subset<Q>)!)
```

§3 calls `Subset<D>` a finite `Index` type (which implies `for cand in Subset<Q>` should iterate all `2^K` masks, just like `for b in Bool`), yet §9 only lists `for g in s.subsets` and never names `Subset<D>.full` (or `.all`). Similarly, `oly23practice5` writes `let by_odd = sorted(0..=n, by: y => -odd[y])`, assuming `sorted` accepts any iterable including a `Range<Int>`. Both should be confirmed explicitly so contestants know they work without an intermediate `[0..=n] i => i` materialization.

## 2. Prototype gaps (`proto/cpc` and `proto/cpi` vs. v2 spec)

Running `proto/cp.py build` on copies of the ten v2 files (and writing the runnable `today/` ports) hit these exact gaps in the current prototypes:

1. **`Str` is opaque in `proto/cpc`** (`nccc9s2`: `Str has no .items`; `nccc9s3`: `` `+` needs Int operands, got Str and Str ``; `nccc8s2`: `cannot iterate Str`). `nccc9s2`, `nccc9s3`, and `nccc10s1` only built under `proto/cpc` because their string lengths were known in advance (`[0..2 * n]Char`, `[0..n, 0..k]Char`). `nccc8s1` and `nccc8s2` read unknown-length strings and had to fall back to `proto/cp.py lib-run`, where `nccc8s1` also lacked `s[l..r]` slicing and `Set<Str>`, bloating it from 11 v2 lines to 32 `today/` lines.
2. **Filtered `for ... if` loops are rejected by `proto/cpc`'s parser** (`oly23practice5`, `nccc9s4`, `nccc10s1`, `nccc8s4`: `expected ':' (at 'if')`).
3. **Struct field defaults fail to parse in `proto/cpc`** (`oly20practice139`: `expected end of line (at '=')`), and `today/oly20practice139.cp` expanded from 55 to 91 lines using parallel per-block arrays.
4. **Tuple `read()` and top-level `@multitest` fail in `proto/cpc`** (`nccc9s1`, `nccc8s1`: `unknown name read`), requiring `fn solve():` wrappers and separate `let` lines per scalar.
5. **`Map.items` and `popcount` are missing in `proto/cpc`** (`oly23practice5`: `Map<(f0: Int, f1: Int), Int> has no .items`; `nccc10s1`: `unknown name popcount`), forcing `for (k, v) in pen:` and a manual 7-line `popcnt` loop.
6. **`compress(a)` still uses the old `(D, vals)` tuple API in `proto/cpc`** instead of v2's `Compressed<T>` with `.at(x)` and `.first_ge(x)`. Both `today/oly20practice139.cp` and `today/nccc9s4.cp` had to write manual binary searches over `vals`.
7. **`Mod<P>` is absent from `proto/cpc`**, and in `stdlib/mod.cp` (`proto/cpi`) it has no one-way `Int` reduction (`ans = ans * Mod<1_000_000_007>(cnt[i] + 1)` instead of `ans *= x + 1`).

What worked cleanly on the first try in `proto/cpc`: Euclidean floor division `(a + b - 1) / b` for signed CHT intersections (`oly20practice139` agreed on all 300 cases including negative intercept differences and `10^9` coordinates), `[Bk]List<Int> = []` arrays of independent lists, `Map<(Int, Int), Int>` with tuple keys, nested closures mutating outer arrays, and `[0..n, 0..k]Char = read()` packed row reads.

## 3. Ranked recommendations

1. **Lock `defer` as block-scoped (per loop iteration) and add `for mut x in xs:`** (1.1, 1.4). Both eliminate manual boilerplate in Rollback Mo's, backtracking, and arrays of amortized/self-adjusting structs (`Block`, `Dsu`, `MonoDeque`) with zero new keywords.
2. **Add `D.chunks(k)` returning clamped `Range<D>` sub-ranges (and `Compressed<T>` range slicing)** (1.2, 1.3). Sqrt-decomposition and block-based offline algorithms appear constantly in Olympiad and ICPC tasks; `chunks(k)` is the natural sibling to `D.windows(k)`.
3. **Clarify closure-based graph inference for vertexless algorithms (`topo`, `scc`, `two_color`)** (1.5). Document `topo::<V>(neighbors)` or `topo(V, neighbors)` so adjacency-matrix and implicit graphs do not need throwaway edge lists.
4. **Tighten the one-page cheat sheet on five everyday items:** `Str + Char`, list comprehensions `[expr for ...]`, `List.filled`, iterating `Subset<D>` directly (plus `Subset<D>.full`), and `['a'..='z']Int` + `prod(it)` (1.6, 1.7, 1.8).
5. **Prioritize the top `proto/cpc` gaps driving the 161-line v2-to-today gap:** flat ASCII `Str` indexing/slicing/iteration, v2 `compress` (`.at`/`.first_ge`), `for ... if`, tuple `read()`, `popcount`, and `Mod<P>` (section 2).
