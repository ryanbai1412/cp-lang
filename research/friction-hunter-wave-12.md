# Friction Hunter — Wave 12 (ports 331–360)

Base: `main` at `555b6c7`, which includes `1f32fec` ("Batch 7 traps: wide Mod
multiply, shift and power range in every build, tuple read of a new domain,
enum match arms"). This is a read-only audit of 30 compiling ports that no
earlier wave covered. It adds a systematic probe of container, string and
array access in `cpl run` versus `cpl run --release`. Every candidate gap and
trap was reproduced on HEAD before it was counted.

Labels as in earlier waves: **[WA-TRAP]**, **[REAL GAP]**, **[STALE PORT]**,
**[REJECTED]**.

## Ports (30)

**ecnerwala waves 5–6 (15):** `cf2187g`, `cf2207e1`, `cf2180h1`, `cf2201c`,
`cf2183d1`, `cf2180d`, `cf2187e`, `cf2180h2`, `cf2159d1`, `cf2201a2`,
`cf2201a1`, `cf2165b`, `cf2180c`, `cf2178b`, `cf2222d`.

**canonical (15):** `sim3/{p5_abc481_g, p6_abc478_e, p3_abc481_f,
p1_abc476_g, p2_abc479_e}`, `blind_critic_1/{3757433, 3767355}`,
`blind_critic_3/ioi09p6`, `blind_critic_4/ioi22p1`,
`scout_batch_51/egoi22p3`, `scout_batch_43/etsr23r4p2`,
`scout_batch_22/ioi16p4`, `scout_batch_20/ccc19s4`, `scout_batch_9/ioi19p2`,
`scout_batch_29/sdoi17r1p6`.

## Averages

- **Real gaps:** 3 (2 from ports, 1 from a probe), so `real_gaps / 30` =
  **0.10** (wave 11: 0.17).
- **Awkward lines:** 152 across the 30 ports, so `total_awkward_lines / 30` =
  **5.07** (wave 11: 15.63). 39 are in the ecnerwala ports and 113 in the
  canonical ports. Wave 11's rate was inflated by ten blind-critic ports
  written without the stdlib. This wave's canonical ports mostly use it
  (`Rooted`, `scc`, `Dsu`, `LazySegTree`, `bsearch_first`, `Graph`).
- **WA traps:** **4 new**. `1f32fec` closed the wave-10 shift and power trap.
  The 3 traps from wave 11 are still open (below).
- **Cumulative, 360 ports:** 173 real gaps and 2615 awkward lines, so
  **0.48** real gaps and **7.26** awkward lines per problem.

Per-port awkward lines / real gaps: cf2187g 6/0, cf2207e1 4/0, cf2180h1 2/0,
cf2201c 0/0, cf2183d1 4/0, cf2180d 2/0, cf2187e 4/0, cf2180h2 0/0, cf2159d1
4/0, cf2201a2 3/0, cf2201a1 3/0, cf2165b 1/0, cf2180c 1/0, cf2178b 3/0,
cf2222d 2/0, p5_abc481_g 19/0, 3757433 11/0, p6_abc478_e 5/0, ioi09p6 12/1,
p3_abc481_f 7/0, egoi22p3 6/0, etsr23r4p2 4/0, p1_abc476_g 6/0, ioi22p1 6/0,
p2_abc479_e 4/0, ioi16p4 8/0, 3767355 16/0, ccc19s4 5/0, ioi19p2 3/1,
sdoi17r1p6 1/0.

## Differential runs: `cpl run` vs sanitized C++17

All 15 ecnerwala ports were run on 15 inputs each from
`gen/NAME.py SEED SEED%8+1`. Each input went through `cpl run` and through the
`cpl build --checked` binary, built with ASan, UBSan and
`_GLIBCXX_ASSERTIONS`. That is **225 runs with 0 divergences** and no
sanitizer reports. The canonical ports have no generator scripts.

## Did `1f32fec` close the wave-10 trap? Yes

`5 >> 64`, `-5 >> 64`, `1 << 64`, `5.bit(-1)` and `2 ** -1` now stop in the
release build with the same message as `cpl run` (for example "a shift by 64
is outside 0..64"), with exit status 3.

## Access probes, `cpl run` vs `cpl run --release` (40 probes, 80 runs)

These probe every indexing and mutation path on `List`, fixed-size arrays,
domains, `Str`, `Map`, `Set`, `Deque`, `MinHeap`, `SortedBag` and `Bitset`
with out-of-range, negative and empty inputs.

### New WA traps (4)

1. **[WA-TRAP] Fixed-size arrays skip bounds checks in release.** This
   covers reads, writes and every dimension. `cpl run` stops with "index 5 is
   outside an array dimension of length 3". The release build exits 0. A write
   past the end of a row silently lands in the next row:

   ```cp
   let i: Int = read()      // 3
   var g: [0..3, 0..3]Int = 0
   g[0, i] = 7              // release: writes g[1, 0]
   print(g[1, 0])           // cpl run: R0001; release: 7
   ```

   `var a: [0..3]Int` with `a[5]` prints `0`, and `g[-1, 0]` prints `0`. In
   release, `List` indexing does check ("index outside the list"), so this is
   specific to arrays declared with a range or domain.

2. **[WA-TRAP] The domain constructor `D(i)` skips its range check in
   release.** `cpl run` stops with "5 is not in `D`, which is 0..=2". The
   release build prints `0` and exits 0. Domains are the main way the language
   proves an index safe, so an unchecked `D(i)` undoes that guarantee on the
   judge.

   ```cp
   let i: Int = read()      // 5
   domain D = 0..3
   var a: [D]Int = 0
   print(a[D(i)])           // cpl run: R0001; release: 0
   ```

3. **[WA-TRAP] `Str` indexing skips bounds checks in release.** `s[10]` and
   `s[-1]` on `"abc"` read `0`. `s[5] = c` is silently dropped, so the string
   is still `abc`. All of these exit 0, while `cpl run` stops with "index 10
   is outside a string of length 3". The emitted C++ is a raw `s[i]`. `Str`
   slices are checked in both backends.

4. **[WA-TRAP] `xs.truncate(-1)` empties the list silently** in both
   backends. This is the same "negative count means empty" class as wave 11's
   `repeat(s, -2)`. `xs.truncate(10)` on a 3-element list is a no-op, which is
   fine.

### Still open from wave 11

- `xs.swap(0, 7)` on a 3-element list: `cpl run` traps; release prints
  `0 2 3`.
- `xs.rev()` and `xs.sorted()` as statements do nothing. This wave adds the
  free-function form: `sorted(xs)` as a statement also does nothing. Neither
  `cpl check` nor `cpl lint` warns.
- `repeat("ab", -2)` returns `""` in both backends.

### Diagnostic gaps and a backend divergence

- **`m[k]` on a missing `Map` key:** `cpl run` prints "the key `5` is not in
  the map; `m.get(k)` gives an optional". Release aborts inside libstdc++
  (`std::out_of_range: map::at`) with no cpl message. `List.filled(-1, 0)`
  does the same with `std::length_error` (seen in wave 11, still open).
- **`xs.fill(4)` on a `List`:** `cpl run` fails with "internal: this value
  can't be changed that way". The release build fills the list. That is a
  divergence between the two backends, with the interpreter as the side that
  fails.
- **[REAL GAP] `Str` has no `.last` / `.first`** (E0203), although `List`
  has both.

### Clean in both backends

- **Lists:** `List` index, negative index and store; `insert`/`remove` out
  of range (assertion); `xs.last!` on empty; `xs.rev[i]` and `xs[1..2][i]`
  out of range.
- **Other containers:** `Deque.get` out of range (assertion); `pop`,
  `pop_front`, `MinHeap.pop` and `SortedBag.nth` on empty give `none`;
  `Map.remove`, `Set.remove` and `SortedBag.remove` of a missing key are
  no-ops; `Bitset` is indexed by a domain, so out-of-range values stop at
  `from_idx`.
- **Strings:** `Str` slices out of range or negative.

## Real gaps (3)

1. **[REAL GAP] Worklist BFS cursor loop** (`ioi09p6`, twice: a multi-source
   bee BFS and a grid BFS with state; `ioi19p2`, a BFS bounded by a size
   limit). Both are `var i = 0; while i < q.len: ...; q.push(..)`. `bfs` in
   `std.graph` takes a `Graph` and a single source, so neither fits.
2. **[REAL GAP] `Str` has no `.last` / `.first`** (from the probes).

## Awkward lines with a clean form on HEAD (selected)

- **[STALE PORT] Hand helpers the stdlib already has:**
  - `ioi09p6` hand-rolls 4-neighbours (`std.grid.adj4`) and answer
    bisection with `-1` / `1e9 + 1` sentinels (`bsearch_first`, which
    `ioi22p1` uses).
  - `p5_abc481_g` builds fact, pow2 and inv_fact tables by hand (`Comb`).
  - `3767355` has its own `popcnt` (`popcount`).
  - `p6_abc478_e` builds condensation arcs by hand (`condense`).
- **[STALE PORT] Last-match loops** that overwrite a variable instead of
  `find(..)` (`cf2187g`, 4 loops). Neighbour searches written twice
  (`3757433`).
- **[STALE PORT] Two-line state rotations** instead of tuple assignment, for
  example `let n_mi = d - r_ma; let n_ma = d - r_mi; r_mi = n_mi; r_ma = n_ma`
  in `cf2180d` and the same shape in `cf2178b`.
- **[STALE PORT] Descending sort by negation**, `sorted([(-a[i], i) ..])`, in
  `cf2201a1` and `cf2201a2`. An argsort by `(val, idx)` pair in `cf2222d`.
- **[STALE PORT] `Z(1)` / `Z(i)` / `Mod<65521>(x)` literal wrapping** in
  `p5_abc481_g` and `3767355`. `if odd: -1 else: 1` signs before wrapping.
- **[STALE PORT] Index crossings.** `ioi22p1` passes `Int` columns with
  `if col < 0 or col >= n` guards and rebuilds `Col(col)`, instead of stepping
  the domain index optionally. `3767355` indexes `Row.at(..)` and
  `Mask.at(..)` everywhere.
- **[STALE PORT] Copied relaxation code.** `@brute` and `@checker` repeat the
  same relaxation loop (`p6_abc478_e`).

## Rejected

- **`type Z = Mod<p>` with `p` read at runtime** (`etsr23r4p2`) looked like
  it should not compile. It does, and it agrees across backends.
- **`cpl build --checked` + ASan on the `Str` probe** did not flag the
  out-of-range read in a single run. That is not counted as separate
  evidence; the release-build result above is what counts.

## Suggested fixes, by leverage

1. **Release-build bounds checks** for fixed-size and multi-dimensional
   arrays, for `D(i)`, for `Str` indexing and for `List.swap`. These are the
   judge-facing equivalents of checks `cpl run` already does. `List` indexing
   in release shows the cost is acceptable.
2. **Reject a discarded value from a pure call** (`xs.rev()`, `xs.sorted()`,
   `sorted(xs)`) at check time.
3. **Make negative counts trap** in `repeat` and `truncate`, as `List.filled`
   does.
4. **Fix the interpreter's `xs.fill(v)`**, and print cpl messages for a
   missing `Map` key and for `List.filled` with a negative length in release.
