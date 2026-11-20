# Friction Hunter — Wave 14 (ports 391–420)

Base: `main` at `b2c22c2`, which includes `30e427e` ("Batch 7 part 2: graph
vertices from a lambda and as tuples, getters with (), std.rand, Dir8,
windows, zip, digit") and ecnerwala corpus wave 7. This is a read-only audit
of 30 compiling ports that no earlier wave covered. It also runs an edge-case
sweep of every Batch 7 feature through `cpl run`, `cpl run --release` and
`cpl build --checked` (ASan, UBSan, `_GLIBCXX_ASSERTIONS`), plus a smoke
test of the CLI commands on a fresh temp project. Every candidate gap and trap
was reproduced on HEAD before it was counted.

Labels as in earlier waves: **[WA-TRAP]**, **[REAL GAP]**, **[STALE PORT]**,
**[REJECTED]**.

## Ports (30)

**ecnerwala corpus wave 7 (16):** `cf2152h2`, `cf2152f`, `cf2157f`,
`cf2159f`, `cf2180g`, `cf2150d`, `cf2268d`, `cf2255d`, `cf2178e`,
`cf2152e`, `cf2255c`, `cf2159a`, `cf2152h1`, `cf2150b`, `cf2157c`,
`cf2152c`.

**canonical (14):** `scout_batch_6/nccc8s3`, `scout_batch_38/dpf`,
`scout_batch_35/coci21c6p2`, `scout_batch_33/btoi16p1`, `cco24p1`,
`scout_batch_7/sac22cc5p4`, `scout_batch_27/acc7p3`,
`scout_batch_2/cco20p2`, `scout_batch_16/{noip07p2, cco05p4}`,
`blind_critic_2/{sac22cc2p4, noi15p4, cco07p2}`,
`scout_batch_47/cco04p1`. Apart from `_draft` copies, these are the last
unaudited canonical ports with at least 16 lines, so the 30th port is a 16th
ecnerwala one.

## Averages

- **Real gaps:** 2 (1 from a port, 1 from a probe), so `real_gaps / 30` =
  **0.07** (wave 13: 0.10).
- **Awkward lines:** 77 across the 30 ports, so `total_awkward_lines / 30` =
  **2.57** (wave 13: 1.13). The new wave-7 ecnerwala ports are longer (median
  27 lines) and account for 49 of them; the canonical ports account for 28.
  9 ports have no awkward line at all.
- **WA traps:** no new kind. One real port, `cf2152e`, hits the wave-12
  release-build array trap (below).
- **Cumulative, 420 ports:** 178 real gaps and 2726 awkward lines, so
  **0.42** real gaps and **6.49** awkward lines per problem.

Per-port awkward lines / real gaps: cf2152h2 9/1, cf2152f 8/0, cf2157f 2/0,
cf2159f 3/0, cf2180g 4/0, cf2150d 5/0, cf2268d 2/0, cf2255d 1/0, cf2178e
2/0, cf2152e 2/0, cf2255c 0/0, cf2159a 4/0, cf2152h1 4/0, cf2150b 0/0,
cf2157c 1/0, cf2152c 2/0, nccc8s3 0/0, dpf 0/0, coci21c6p2 0/0, btoi16p1
3/0, cco24p1 7/0, sac22cc5p4 3/0, acc7p3 2/0, cco20p2 0/0, noip07p2 2/0,
cco05p4 2/0, sac22cc2p4 6/0, noi15p4 0/0, cco07p2 0/0, cco04p1 3/0.

## Differential runs: `cpl run` vs sanitized C++17

All 16 ecnerwala ports were run on 15 inputs each from
`gen/NAME.py SEED SEED%8+1`, through `cpl run` and through the
`cpl build --checked` binary. That is **240 runs**. 229 agree. The **11
divergences are all in `cf2152e`**, and all have the same cause:

- **`cf2152e` hits the release-build array trap.** `cf2152e` is interactive,
  and its generator feeds a fixed transcript, so on most seeds the replies do
  not match the queries. The port then walks `v` down to `-1`:
  - `cpl run` stops with "index -1 is outside an array dimension of length
    2" at `while lcp[v] != j: v -= 1`.
  - The checked binary stops with no output after `? 2 1 2`.
  - `cpl run --release` prints `! -1 1` and exits 0.

  The input is invalid for an interactive problem, so this is not a port
  bug. It is the wave-12 trap (fixed-size arrays have no bounds checks in
  release builds) showing up in a real port: on the judge, a wrong
  interaction becomes a silent wrong answer instead of a runtime error.
  Separately, `gate.sh`-style differential runs of interactive ports need an
  interactor, not a static generator.

## Batch 7 edge-case sweep (36 probes, about 90 runs)

Every probe agrees across `cpl run` and `cpl run --release`. A 12-probe
subset also agrees with `cpl build --checked`, with no sanitizer report.

**Clean, with a clear message where it traps:**

- **`xs.windows(k)`:** `k = 0` and `k = -2` trap with "a window holds at
  least one element". `k > len` and an empty list give no windows, as
  documented.
- **`zip`:** lists of unequal length give as many pairs as the shorter one,
  as documented. This is silent truncation by design.
- **`Char.digit`:** a non-digit traps with "`digit` needs one of '0' to
  '9'".
- **`x.inv()`:** works the same as `x.inv`. `Z(0).inv()` and a non-coprime
  inverse trap with "0 never has one, and no value that shares a factor with
  the modulus". `Dir8(1).cw()` works the same as `.cw`.
- **`std.rand`:** `Rng.int(lo, hi)` with `lo > hi` traps. `lo == hi`, and
  `shuffle` of an empty list, work.
- **`Dir8` and `step`:** from a corner, every off-grid direction gives
  `none`.
- **`find_all`:** overlapping matches work (`find_all("aaaa", "aa")` →
  `0 1 2`). A pattern longer than the text gives nothing. An empty pattern
  matches at all 4 boundaries of `"abc"`, which is consistent, though worth
  documenting.
- **`bfs` / `dijkstra` on `(R, C)` vertices:** unreachable cells, duplicate
  sources and an empty source list are handled. `dijkstra` with a negative
  edge weight still returned the right distances on a re-relaxation case,
  because it is the lazy variant. The docs say "non-negative" and nothing
  checks it, but the risk is time, not correctness.
- **`peel`, `topo` and `scc` with lambda inference:** `topo` on a cycle
  gives `none`, and `peel` on a cycle gives `[]`. `scc` with `n = 0`, and
  with `n = 1` and a self-loop, works.

**Found:**

- **[REAL GAP] `Rng.int` cannot cover a full-width range.**
  `rand.cp:18` computes `hi - lo + 1`, which overflows. `r.int(0, Int.MAX)`
  stops `cpl run` with "integer overflow", reported at line 1:1 inside the
  stdlib, but works in release, so the two backends disagree.
  `r.int(Int.MIN, Int.MAX)` stops `cpl run` the same way and crashes the
  release build ("the program crashed"). A fix is to compute the span in
  `U64`.

## CLI smoke test (temp project)

Run in a fresh directory against `main`. All times are wall-clock on a warm
binary.

- **`new`:** `cpl new sum --constraints "n: 1..=2e5; a: [n]-1e9..=1e9"`
  takes 6 ms and writes typed reads and an empty `sum.tests/`.
  `new --companion comp.json` takes 4 ms and writes the title, URL,
  `// limits:` line and the sample.
- **`check`, `run`, `build`:** `check` 5 ms; `run` 5 ms; `run --release`
  364 ms on first use, including g++, which is under the 500 ms target;
  `build` 5 ms (2,716 B); `build --checked` 552 ms.
- **`test`:** 9 ms, 2/2 passed.
- **`stress`:** 951 ms for 1000 cases against `@brute(main)`.
- **`fmt`, `lint`, `doc`:** `fmt --check` 4 ms, and it flags and rewrites
  an unformatted file. `lint` 4 ms; `lint --list` works. `doc --out` 4 ms
  writes `index.html`, `style.css` and a page per file.
- **`bench`:** 441 ms for 6 maximum-size shapes, each within its limits.
- **`lsp`:** answers `initialize` with its capabilities.
- **Errors:** a missing file gives a clean error from `run` and `test`.
  `--version` prints `cpl 0.2.2`.
- **Diagnostic gap:** `cpl new bad --constraints "n: 1..=; a: [n]"` accepts
  the malformed `n: 1..=`. It then fails on `a: [n]` with "can't generate a
  `` yet", naming an empty type.

## Real gap from ports (1)

- **[REAL GAP] No meldable (mergeable) heap** (`cf2152h2`, slope trick). The
  port hand-writes a skew-heap `meld` over `?Int` node ids in a
  `List<HeapNode>`, plus a hand-rolled DSU with negative-size parents. The
  DSU part is stale; the heap part is not.

## Awkward lines with a clean form on HEAD (selected)

- **[STALE PORT] Code Batch 7 now replaces.**
  - `sac22cc2p4` runs a hand-written 4-direction Dijkstra over
    `(Row, Col)` with a `(dr, dc)` list. It becomes
    `dijkstra(p => [(to: q, w: cost(q)) for q in p.adj4], ...)`.
  - `cco24p1` hand-rolls a multi-source Dijkstra with negative start costs.
    It becomes `dijkstra(v => adj[v], [(v, -a[v]) for v in V], e => e.w)`.
- **[STALE PORT] Interactive helpers.** `print("?", ..); flush(); read()`
  query helpers, instead of `ask("?", ..)`: `cf2159f`, `cf2178e`,
  `cf2152e` and `cf2159a`.
- **[STALE PORT] `let _ = xs.pop()` and `let _ = dfs(..)`** to discard a
  result (`cf2159a`, `cf2152h1`, `cf2157f`). These look like workarounds
  for the wave-11 discarded-result issue, but `xs.pop()` alone also compiles.
  Once discarded pure results become an error, these should stay legal.
- **[STALE PORT] `Comb` by hand.** `place_ways` written as
  `fact * inv_fact * inv_fact` instead of `comb.multichoose` (`cf2150d`).
  Typed literals `let two: Z = 2` (`cf2180g`).
- **[STALE PORT] Small idioms.** `best = min(best ?? pay, pay)` instead of
  `best min= pay` (`btoi16p1`). `ch - '\0'` to index by character, five
  times (`cco04p1`). Hand binary search instead of `bsearch_first`
  (`sac22cc5p4`).

## Suggested fixes, by leverage

1. **Release-build bounds checks for fixed-size arrays** (carried over from
   wave 12). `cf2152e` shows the trap on a real port.
2. **`Rng.int`:** compute the span in `U64` so full-width ranges work and
   both backends agree.
3. **`cpl new --constraints`:** reject a malformed range such as `1..=`
   where it is written, and never print an empty type name.
4. **Docs:** mention the empty-pattern behaviour of `find_all`, and that
   `dijkstra`'s non-negative-weight rule is unchecked.
