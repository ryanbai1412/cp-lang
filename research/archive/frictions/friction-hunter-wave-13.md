# Friction Hunter — Wave 13 (ports 361–390)

Base: `main` at `f3454f4`. This is a read-only audit of 30 compiling ports that
no earlier wave covered. It adds a degenerate-input probe sweep over ten
stdlib structures, each run through `cpl run` and `cpl run --release`. Every
candidate gap and trap was reproduced on HEAD before it was counted.

**Pool note:** the unaudited ecnerwala ports are now tiny (4–12 lines). Of
the canonical pool, only scout-library files (`connectivity`, `flow_ext`,
`hashing`) are longer than 37 lines. This wave's 30 ports average 19 lines, so
the per-problem numbers drop again. For the same reason, the probe sweep
carries more of this wave's weight.

Labels as in earlier waves: **[WA-TRAP]**, **[REAL GAP]**, **[STALE PORT]**,
**[REJECTED]**.

## Ports (30)

**ecnerwala (15):** `cf2187a`, `cf2229c1`, `cf2207a`, `cf2180e`, `cf2180b`,
`cf2178c`, `cf2157a`, `cf2237c`, `cf2183c`, `cf2183b`, `cf2229b`, `cf2165a`,
`cf2178a`, `cf2237a`, `cf2229a`.

**canonical (15):** `scout_batch_9/ioi20p6`,
`blind_critic_1/{4342005, 4553296}`, `hillclimb_dp_graph/{apio07p3,
coi06p2}`, `blind_critic_3/{noip22p1, cheerio1s4, cco16p1}`,
`scout_batch_35/coci21c1p5`, `scout_batch_20/cco23p1`,
`blind_critic_2/ccc02s4`, `aac5p2`, `scout_batch_37/noip06p3`,
`scout_batch_28/noip05p2`, `scout_batch_27/coci20c6p2`.

## Averages

- **Real gaps:** 3 (2 from ports, 1 from a probe), so `real_gaps / 30` =
  **0.10** (wave 12: 0.10).
- **Awkward lines:** 34 across the 30 ports, so `total_awkward_lines / 30` =
  **1.13** (wave 12: 5.07). 6 are in the ecnerwala ports and 28 in the
  canonical ports. 11 of the 30 ports have no awkward line at all.
- **WA traps:** **2 new**, both in stdlib structures.
- **Cumulative, 390 ports:** 176 real gaps and 2649 awkward lines, so
  **0.45** real gaps and **6.79** awkward lines per problem.

Per-port awkward lines / real gaps: cf2187a 3/0, cf2180b 1/0, cf2178a 1/0,
cf2229a 1/0, and the other 11 ecnerwala ports 0/0; ioi20p6 1/0, 4342005 4/1,
4553296 2/1, apio07p3 3/0, coi06p2 0/0, noip22p1 2/0, cheerio1s4 3/0,
cco16p1 1/0, coci21c1p5 1/0, cco23p1 2/0, ccc02s4 3/0, aac5p2 4/0,
noip06p3 1/0, noip05p2 0/0, coci20c6p2 1/0.

## Differential runs: `cpl run` vs sanitized C++17

All 15 ecnerwala ports were run on 15 inputs each from
`gen/NAME.py SEED SEED%8+1`. Each input went through `cpl run` and through the
`cpl build --checked` binary, built with ASan, UBSan and
`_GLIBCXX_ASSERTIONS`. That is **225 runs with 0 divergences** and no
sanitizer reports.

## Stdlib structure probes (52 probes, 104 runs)

Each probe was run through `cpl run` and `cpl run --release`. It covers
`SegTree`, `LazySegTree`, `Fenwick`, `Dsu`, `Rooted`, `FlowGraph`,
`RollingHash`, `Comb`, `Bitset` and `@memo`, with empty and reversed ranges,
`n = 0` and `n = 1`, out-of-range `kth`, `choose` with `k < 0` or `k > n`,
self-loops, disconnected graphs and deep recursion.

### New WA traps (2)

1. **[WA-TRAP] `Fenwick.kth` / `lower_bound` with a negative entry give a
   wrong index silently** in both backends. `stdlib/fenwick.cp` documents
   "for non-negative values", but nothing checks it. A port that keeps a
   multiset count which goes negative after a bad update gets a plausible
   wrong index instead of an error.

   ```cp
   domain D = 0..3
   var f = Fenwick<D, Int>()
   f.add(D(0), 1)
   f.add(D(1), -1)
   f.add(D(2), 2)
   print(f.kth(1))          // 2 in both backends; the first prefix >= 1 is at 0
   ```

   `f.lower_bound(1)` returns boundary 3 instead of 1. Two fixes would work:
   a debug-build check that `add` keeps every entry non-negative, or a
   linear-scan fallback when the tree has seen a negative `add`.

2. **[WA-TRAP] `RollingHash.slice(l, r)` with `l > r` is the empty hash.**
   `h.slice(4, 2) == h.slice(0, 0)` is `YES` in both backends. An
   out-of-range end does assert ("0 <= l and r <= p1.len - 1"). This is the
   same "reversed range silently means empty" class that audit #11 made a trap
   for slices. `SegTree.fold`, `LazySegTree.apply` and `Fenwick.sum` all trap
   on reversed ranges now.

   ```cp
   let h = RollingHash("abcdef")
   print(h.slice(4, 2) == h.slice(0, 0))   // YES
   ```

### Real gap from the probes (1)

- **[REAL GAP] `cpl run` caps recursion depth below common CP sizes.** A
  plain recursive `fn` stops at depth 104,086, and an `@memo` fn over a domain
  stops at 137,912. Both report "calls are nested N deep: `f` never stops
  calling itself?". The release build finishes depth 200,000 (`n = 2e5`, a
  path graph, the usual worst case). `Rooted` on a 200,000-node path is fine
  because it is iterative. So hand-written DFS and linear `@memo` chains work
  on the judge but fail locally, and the error blames the program.

  ```cp
  fn f(i: Int) -> Int:
      if i == 0: return 0
      f(i - 1) + 1
  print(f(200000))         // cpl run: R0001 at depth 104086; release: 200000
  ```

### Diagnostic gaps

- **`Rooted` on a disconnected graph** fails with the internal
  "assertion failed: order.len == V.K", with no mention that the graph is not
  connected or is not a tree.
- **`t.ancestor(v, -1)`** reports "`!` on `none`" at line 1:1 instead of "k
  must be non-negative".
- **`SegTree.of(+, d => 1)`**, where the lambda gives no domain, reports
  "a `Int` has no member `ord`" or "doesn't know the name `D` here", both at
  1:1. Writing a parameter type, `(d: D) => 1`, is a parse error (E0103).
  Today `SegTree.new(+, [D] d => 1)` is the working form.
- **`RollingHash.slice` out of range** reports
  "0 <= l and r <= p1.len - 1", which names an internal field.
- **`Comb.catalan(-1)`** is `0`, but `Comb.fact(-1)` asserts.
  `choose(n, k)` returning `0` for `k < 0`, `k > n` or `n < 0` is the usual
  convention and is consistent.
- **`Rooted` with a self-loop edge** is accepted silently: depth and size are
  computed as if the loop were absent. That is lenient, but noted.

### Clean in both backends

- **SegTree:** a reversed `fold` traps ("the range runs backwards"). An empty
  `fold` gives the identity, `n = 0` gives `all() = 0`, and `max_right` from
  both ends is correct.
- **LazySegTree:** a reversed `apply` traps, and `n = 1` works.
- **Fenwick:** `kth(0)` asserts and `kth` past the total gives `none`. A
  reversed `sum` now traps; wave 11 reported it silently empty.
- **Dsu:** a self-`unite` returns false with size 1, and `n = 0` has 0
  components.
- **Rooted:** `ancestor(v, k)` with `k` above the depth, and `step(a, b, k)`
  with `k` above the distance, give `none`.
- **FlowGraph:** `s == t` and negative capacity assert, and a self-loop arc
  is harmless.
- **Comb:** it grows past `limit`, and `Comb<7>.choose(14, 7) = 2`, so
  Lucas is handled for small primes.
- **Bitset:** `n = 0`, `shifted(-1)` and `next` work. `@memo` works for
  `n = 1` and negative `Int` arguments.

## Real gaps from ports (2)

- **[REAL GAP] Worklist cursor loop** in `4342005` (a multi-source grid BFS
  carrying colour and distance) and `4553296` (an SPFA-style relaxation queue
  with a re-queue flag). Both are `var head = 0; while head < q.len: ...`.
  Neither fits `bfs` or `dijkstra`.

## Awkward lines with a clean form on HEAD (selected)

- **[STALE PORT] Optional and comparison idioms.**
  `ans = if let cur = ans: min(cur, d) else: d` instead of `ans min= d`
  (`cf2187a`). Argmin tracking with `least == none or t < least!`
  (`ccc02s4`). `if s1 < s2: s1 else: s2` instead of `min(s1, s2)`
  (`cf2180b`).
- **[STALE PORT] Grid neighbours.** Four `offset(±1)` blocks instead of
  `adj4` (`4342005`).
- **[STALE PORT] Index crossings.** `I(g.label + 1)` and `Window(w + 1)` in
  `aac5p2` and `cco23p1`, where a domain-typed step would do. `cco23p1` also
  hand-rolls a `w += k` stride over a domain.
- **[STALE PORT] Sentinels.** `1e18` (`cheerio1s4`) and `?? -1` on output
  (`4553296`).

## Suggested fixes, by leverage

1. **WA traps:** make `RollingHash.slice` reject `l > r`, matching the
   reversed-range traps on `SegTree`, `LazySegTree` and `Fenwick`. Check
   `Fenwick`'s non-negative precondition in debug builds, or make `kth` and
   `lower_bound` fall back when it is violated.
2. **Interpreter depth:** raise or remove `cpl run`'s recursion cap, for
   example by running the interpreter on a large dedicated stack, so depth
   200,000 works. Until then, say "the interpreter's depth limit" instead of
   "never stops calling itself?".
3. **Diagnostics:** give `Rooted` a "graph is not connected" message, give
   `ancestor` a negative-`k` message, point `SegTree.of` lambda-inference
   errors at the call, and accept typed lambda parameters.
