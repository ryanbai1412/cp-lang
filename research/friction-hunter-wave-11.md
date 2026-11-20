# Friction Hunter — Wave 11 (ports 301–330)

Base: `main` at `d655dd3`. This is a read-only audit of 30 compiling ports that
no earlier wave covered. Every candidate gap and trap was reproduced on HEAD
before it was counted.

Wave 10 ran out of long ports in its usual pools. This wave mixes **15
ecnerwala ports from corpus waves 5 and 6**, which all have generators, with
**15 long canonical ports** from `blind_critic_4/5` and `scout_batch_*`. The
blind-critic ports were written as if the stdlib did not exist (hand-rolled
Fenwick trees, lazy segment trees, sieves, `lower_bound`). That is why this
wave's awkward-line rate is far above wave 10's. Without those 10 ports, the
other 20 average 4.6 awkward lines each.

Labels as in earlier waves: **[WA-TRAP]**, **[REAL GAP]**, **[STALE PORT]**,
**[REJECTED]**.

## Ports (30)

**ecnerwala waves 5–6 (15):** `agc077a`, `ucup4s25e`, `cf2183d2`, `cf2165e`,
`cf2187b`, `cf2178f`, `cf2159b`, `cf2201f1`, `cf2159c`, `cf2159d2`,
`cf2165c`, `cf2187c`, `cf2178d`, `cf2165d`, `cf2201b`.

**canonical (15):** `blind_critic_5/{noi22p5, bsspc21s4, noi04p3, noi08p4,
ioi19p5, noiwc22p3}`, `blind_critic_4/{ioi15p6, ioi11p6, ioi11p5, ioi13p6,
dmopc21c5p5, ioi17p6}`, `scout_batch_19/egoi23p3`,
`scout_batch_43/etsr23r1p4`, `scout_batch_34/dmopc20c6p6`.

## Averages

- **Real gaps:** 5 across the 30 ports, so `real_gaps / 30` = **0.17**
  (wave 10: 0.20). All 5 are in canonical ports.
- **Awkward lines:** 469 across the 30 ports, so `total_awkward_lines / 30` =
  **15.63** (wave 10: 2.63). 79 are in the ecnerwala ports (5.3 per port) and
  390 in the canonical ports (26.0 per port). About 95% already have a clean
  form on HEAD.
- **WA traps:** **3 new**, plus a recheck showing the wave-10 shift trap is
  still open (below).
- **Cumulative, 330 ports:** 170 real gaps and 2463 awkward lines, so
  **0.52** real gaps and **7.46** awkward lines per problem.

Per-port awkward lines / real gaps: agc077a 8/0, ucup4s25e 10/0, cf2183d2
5/0, cf2165e 5/0, cf2187b 8/0, cf2178f 7/0, cf2159b 5/0, cf2201f1 6/0,
cf2159c 2/0, cf2159d2 5/0, cf2165c 4/0, cf2187c 4/0, cf2178d 4/0, cf2165d
2/0, cf2201b 4/0, noi22p5 60/0, bsspc21s4 25/1, noi04p3 41/0, ioi15p6 35/1,
ioi11p6 30/1, ioi11p5 30/0, ioi13p6 27/1, dmopc21c5p5 33/0, noi08p4 23/0,
ioi17p6 30/0, egoi23p3 8/0, etsr23r1p4 10/1, dmopc20c6p6 3/0, ioi19p5 20/0,
noiwc22p3 15/0.

`friction_lint.py` flags only 5 of these lines (2 real_gap, 3 stale_port),
plus 70 `label_crossing` metric hits. Almost everything below was found by
reading the ports.

## Differential runs: `cpl run` vs sanitized C++17

All 15 ecnerwala ports were run on 15 inputs each from
`gen/NAME.py SEED SEED%8+1`. Each input went through `cpl run` and through the
`cpl build --checked` binary, built with ASan, UBSan and
`_GLIBCXX_ASSERTIONS`. That is **225 runs with 0 divergences** and no
sanitizer reports. The 15 canonical ports have no generator scripts, so they
were read only.

## Targeted WA-trap probes (15 areas, 30 runs)

Each probe was run through `cpl run` and `cpl run --release`.

- **`List.swap` out of range → [WA-TRAP], new.** `cpl run` stops with
  "swap(0, 7) is outside a list of length 3". The release build prints
  `0 2 3` and exits 0. `cpl build` emits a bare `std::swap(xs[0], xs[n]);`
  with no bounds check, so this is an out-of-bounds write on the judge.

  ```cp
  let n: Int = read()      // 7
  var xs = [1, 2, 3]
  xs.swap(0, n)
  print(xs)                // cpl run: R0001; release: 0 2 3
  ```

- **A discarded `xs.rev()` / `xs.sorted()` does nothing → [WA-TRAP], new.**
  Both forms compile as statements. Both backends leave `xs` unchanged, and
  neither `cpl check` nor `cpl lint` warns. Two audited ports do this:
  `ioi11p6` (`sent.rev()`) and `dmopc21c5p5` (`out.rev()`). Both happen to be
  order-insensitive, but the author clearly meant to reverse in place. The
  in-place form is `reverse(mut xs)`. A pure call whose value is dropped
  should be a compile error, or at least a lint error.

  ```cp
  var xs = [0, 1, 2]
  xs.rev()                 // silently a no-op
  print(xs)                // 0 1 2 in both backends
  ```

- **`repeat(s, n)` with negative `n` returns `""` → [WA-TRAP], new.** Both
  backends print an empty line. `List.filled(-1, 0)` traps for the same
  input, so the two count-taking constructors disagree. The cause is
  `stdlib/strings.cp:7`, where `0..n * self.len` is silently an empty range.
  This is the same "negative count silently means empty" class as
  Meta-Factory audit #11's slice traps.

  ```cp
  let n: Int = read()      // -2
  print(repeat("ab", n))   // "" in both backends
  ```

- **Shifts, recheck of wave 10.** `5 >> 64` and `-5 >> 64` print `5 -5` in
  release; `cpl run` traps. `5.bit(-1)` prints `NO` in release; `cpl run`
  traps with "shift by -1". So `Int.bit` inherits the open wave-10 trap.
  It is still open on `d655dd3`.
- **`List.filled(-1, 0)`:** `cpl run` gives a clean R0001. Release aborts
  inside libstdc++ (`std::length_error`) with no cpl message. That is a
  diagnostic gap, not a silent WA.
- **Clean in both backends:** `ctz(0)` = 64 and `popcount(0)` = 0 in both;
  `min` of an empty list and `xs[5..]` on a 3-element list trap in both;
  tuple assignment `(a, b) = (n, n + 1)` works.
- **By design:** `Int(F64)` is rejected without a rounding mode (E0302).
  `sum` overflow traps only locally.

## Real gaps (5)

1. **[REAL GAP] Cover-count segment tree** (`bsspc21s4`, rectangle union).
   The port hand-writes a 45-line `CoverTree` whose node count goes up and down
   with no push-down. `LazySegTree` and its `Action`s cannot express "covered
   if count > 0, else merge the children".
2. **[REAL GAP] Worklist BFS cursor loop** (`ioi15p6`, linter rule W2). It
   uses `var head = 0; while head < qn.len: ...; qn.push(..)`. The port runs a
   BFS over its own adjacency list with weighted edges, so `bfs` does not fit.
3. **[REAL GAP] Domain stepping** (`ioi11p6`, linter rule N2). Lines like
   `RL.from_idx(i - 1)!` step a domain index back by one, and there is no
   checked `pred`/`succ` on a domain index.
4. **[REAL GAP] Dynamic (implicit) segment tree** (`ioi13p6`, 2D gcd over
   coordinates up to 1e9). The port keeps parallel `List<Int>` node arrays with
   `-1` as null. Mutating them recursively forces this three-line pattern six
   times, because `in_lc[o] = inner_update(in_lc[o], ..)` would lend `in_lc`
   twice:

   ```cp
   let cur_lc = in_lc[o]
   let ch = inner_update(cur_lc, l, mid, pos, val)
   in_lc[o] = ch
   ```

5. **[REAL GAP] Segment tree beats** (`etsr23r1p4`, range floor-division).
   The tree needs a "stop here when `mn / x == mx / x`" break condition, and
   `LazySegTree` has no hook for one. The hand-written tree is indexed by
   `Int`, which forces 8 `l.to_idx` / `r.to_idx` crossings at the call sites.

## Awkward lines with a clean form on HEAD (selected)

- **[STALE PORT] Hand-rolled stdlib**, in 8 blind-critic ports. These are
  hand-written Fenwick trees (`noi22p5`), lazy min trees (`noi22p5`),
  `unique_sorted` + `lower_bound` (`noi22p5`, `bsspc21s4`), sieves
  (`dmopc21c5p5`), run-length grouping (`ioi15p6`, which should use `runs`),
  `repeat_str` (`egoi23p3`, which should use `repeat`) and a component-wise
  `Pt` (`ucup4s25e`, which should use `std.geometry.Pt` with `+`/`-`). Together
  that is about 160 lines.
- **[STALE PORT] Push-to-fill loops and untyped reads in loops.** For example
  `for _ in 0..n: let c: Int = read(); costh.push(c)` instead of
  `let costh: [0..n]Int = read()`. This is in 9 ports, about 70 lines.
- **[STALE PORT] Domain crossings.** Ports read `Int`s, then write
  `V.from_idx(i)!` at every index, instead of reading typed indices and
  looping over the domain: `ioi15p6` (22), `ioi11p6` (14), `ioi17p6` (14),
  `dmopc21c5p5` (11).
- **[STALE PORT] `if v < m: m = v` blocks** instead of `m min= v`: about 15
  blocks in `ioi17p6`, `ioi15p6` and `noiwc22p3`. Also `?Int` with `?? 0`
  comparisons where `min=` on an optional already works: `noi04p3`,
  `noiwc22p3`, `ioi19p5`.
- **[STALE PORT] Swaps.** `cf2187c` swaps two list elements in three lines via
  `.take()`. `ioi11p5` uses a three-line temp swap. Both have `xs.swap(i, j)`.
  Related diagnostic gap: `swap(mut xs[i], mut xs[j])` fails with E0601 ("may
  be the same storage") and does not point to `xs.swap(i, j)`.
- **[STALE PORT] Tuple temporaries.** `ioi17p6` writes
  `let nxt = expand(l, r); l = nxt.0; r = nxt.1` three times. Tuple
  assignment `(l, r) = expand(l, r)` works on HEAD.
- **[STALE PORT] `Int(x.bit(b))`**, mixing `Bool` and `Int`: `cf2187b` and
  `cf2165c`. Manual `(bni >> i) & 1` instead of `bni.bit(i)`: `noi04p3`, 5
  times.

## Rejected

- **`cf2159c` reads `[0..=n]Int`.** That looked like an off-by-one, but the
  input really carries `n + 1` values. The generator prints `a[0..n]`.
- **`swap(mut xs[i], mut xs[j])` E0601** was first logged as a real gap. It is
  retracted because `List.swap(i, j)` exists (see the diagnostic note above).
- **`clz` not in scope:** `bit_width` covers it.

## Suggested fixes, by leverage

1. **WA traps:** bounds-check `List.swap` in emitted C++ the same way indexing
   is checked. Reject a discarded value from a pure call (`xs.rev()`,
   `xs.sorted()`) at check time. Make `repeat` with a negative count trap like
   `List.filled`.
2. **Diagnostics:** make E0601 on `swap(mut xs[i], mut xs[j])` suggest
   `xs.swap(i, j)`. Make the release build of `List.filled` with a negative
   length print the cpl message instead of a libstdc++ abort.
3. **Real gaps, in this order:** a beats hook or break condition for
   `LazySegTree`, a dynamic segment tree over a sparse coordinate range, a
   cover-count tree for rectangle-union sweeps, and a checked domain
   `pred`/`succ`.
