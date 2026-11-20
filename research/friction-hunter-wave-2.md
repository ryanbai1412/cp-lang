# Friction Hunter, wave 2

Thirty new solved ports, none of them in wave 1, weighted towards the hard
end: NTT, flows, geometry, interactive problems, lazy segment trees,
matrix power, interval and tree DP. I read every line of each, not only the
lines with a `// WISH` comment. Every pattern was checked against `cpl` on
`main` at `67da0bd`, using the same labels as
[wave 1](friction-hunter-wave-1.md):

- **[STALE PORT]**: HEAD already has a clean form, checked with `cpl run`. Only the port needs rewriting.
- **[REAL GAP]**: still awkward on HEAD; the error is quoted.
- **[REJECTED]**: looks awkward, but the current form is already the obvious one.

There is no `dmoj/` directory, so the DMOJ problems were taken from
`proto/ports/` and its `scout_batch_*` folders. Five hard candidates had
to be swapped out because they do not compile on HEAD, so they are not
"solved ports": `yac6p3` (`where` guards were removed), `scout_batch_18/dpw`
(`std.lazy` has no `MaxOpt`), `scout_batch_10/noi15p5`, `scout_batch_36/cco19p4`
and `scout_batch_9/ioi10p6`.

## The numbers

The 30 ports have 875 non-blank lines. They hold 70 frictions
covering 182 awkward lines: 32 real gaps (80 lines),
35 stale-port patterns and 3 rejected.

- Real gaps per problem: 32 / 30 = **1.07** (wave 1: 0.60).
- Awkward lines per problem: 182 / 30 = **6.07** (wave 1: 3.73).
- Frictions of any kind per problem: 70 / 30 = 2.33.
- Brand-new real gaps, in neither wave 1 nor `research/friction-queue.md`: 14, in 6 clusters.
- Ports with no friction at all: 7 (`cf2261f1`, `ucup5s03f`, `dpp`, `dpy`, `cco18p1`, `dmopc21c5p1`, `cheerio1s3`).

The hard ports carry more friction than wave 1's mix. Most of it is stale:
`cf2262e`, `ucup5s06g` and `noi09p4` alone hold 42 lines that HEAD can
already write cleanly.

## The 30 ports

- `ecnerwala/cf2261d2` (hashing, intervals): 3 frictions.
- `ecnerwala/cf2261f1` (constructive): 0 frictions.
- `ecnerwala/cf2262d` (constructive simulation): 4 frictions.
- `ecnerwala/cf2262e` (NTT, combinatorics): 4 frictions.
- `ecnerwala/cf2268c` (bits, monotonic stack): 5 frictions.
- `ecnerwala/cf2268d` (Cartesian forest, counting): 3 frictions.
- `ecnerwala/cf2268e` (NTT, Catalan): 3 frictions.
- `ecnerwala/ucup5s01a` (2-D DP, Mod): 3 frictions.
- `ecnerwala/ucup5s02e` (permutations): 3 frictions.
- `ecnerwala/ucup5s03f` (combinatorics, Mod): 0 frictions.
- `ecnerwala/ucup5s05e` (geometry, convex hull): 11 frictions.
- `ecnerwala/ucup5s06g` (graphs, 0-1 Dijkstra): 2 frictions.
- `wave2/dpl` (interval DP): 1 frictions.
- `wave2/dpn` (interval DP): 1 frictions.
- `wave2/dpo` (bitmask DP): 1 frictions.
- `wave2/dpp` (tree DP): 0 frictions.
- `wave2/dpr` (matrix power): 3 frictions.
- `wave2/dpy` (grid paths, inclusion-exclusion): 0 frictions.
- `wave2/cco18p1` (2-D DP): 0 frictions.
- `wave2/dmopc21c5p1` (constructive): 0 frictions.
- `noip14p3` (DP over optionals): 2 frictions.
- `aac5p2` (interactive): 2 frictions.
- `cheerio1s3` (geometry): 0 frictions.
- `dpz` (convex hull trick): 2 frictions.
- `scout_batch_15/noi09p4` (max flow, closure): 5 frictions.
- `scout_batch_34/dmopc21c2p5` (tree knapsack): 3 frictions.
- `scout_batch_30/cco21p5` (lazy segment tree): 2 frictions.
- `scout_batch_17/ioi08p4` (permutation cycles, generators): 4 frictions.
- `scout_batch_2/ioi14p2` (lazy segment tree): 1 frictions.
- `scout_batch_15/dmopc20c7p5` (max flow, closure): 2 frictions.

## New real gaps, clustered into general fixes

### N1. Positions from the end, and slice bounds that never lie

Ports: `dpz`, `ioi08p4`, `ucup5s05e`.

Three behaviours on HEAD sit badly together:

- `a[-2]` traps with "index -2 is outside a list of length 3".
- `a[..k]` traps when `k > a.len`.
- `a[..-1]` and `a[1..-1]` **silently return an empty list**.

The third is a correctness bug, not only friction: a Python-trained hand
writes `lower[..-1]` and gets a wrong answer with no error. The ports dodge
all three with `c[c.len - 2]` (5 files in the corpus) and a `spare > 0`
counter in place of "the first `m`, or fewer".

```cp
// today
while c.len >= 2 and turn(c[c.len - 2].p, c[c.len - 1].p, s.p) == -side: c.pop()
let hull = lower[0..lower.len - 1] + upper[1..].rev
// wanted: one from-the-end form, and slice bounds that trap rather than go empty
while c.len >= 2 and turn(c.last(1)!.p, c.last!.p, s.p) == -side: c.pop()
let hull = lower[..lower.len - 1] + upper[1..].rev
for jumps in sorted(loops, by: desc).prefix(m): points += jumps + 2
```

Fix: make a negative or reversed slice bound an error (at check time when
it is constant, otherwise a trap), the same as `a[-2]`. Add `xs.last(k)`
(the k-th from the end, as `?T`) and a clamped `xs.prefix(k)` on every
`Seq`. No new syntax.

### N2. Stepping a domain value where the range is already known

Ports: `cf2262d`, `dpl`, `dpn`.

`l + 1` on a domain value is `?I`, so interval DP writes `(l + 1)!` and
`(r - 1)!` in the `else` of `if l == r`, where `l < r` is already
established (10 files and 20 lines in the corpus match `(l + 1)!`-style
unwraps). A `[I]I` array is worse: `a[cur] += 1` is rejected outright
("`+=` can't be applied to a `I`"), so `cf2262d` reads 1-based values into
`Int` and subtracts 1 from each.

```cp
// today
if l == r: a[l] else: max(a[l] - lead((l + 1)!, r), a[r] - lead(l, (r - 1)!))
// wanted: the range checker already knows l < r here
if l == r: a[l] else: max(a[l] - lead(l + 1, r), a[r] - lead(l, r - 1))
```

Fix: let the range checker refine domain values through guards (`l == r`,
`l < r`, loop bounds `m in l..r`), so the step is non-optional where it is
proven. Allow `+=` / `-=` on a domain value as a checked step that traps
on overflow, matching the overflow rule for `Int`. This is general range
inference, not a special case.

### N3. Uncuttable edges in `FlowGraph`

Ports: `dmopc20c7p5`, `noi09p4`.

Both project-selection (closure) ports invent an infinity, `1e12` in one
and `1e18` in the other. Each one is a silent overflow or wrong-cut risk
when the real capacities grow.

```cp
// today
const INF = 1000000000000000000
net.add(pnode, Node(u - 1), INF)
// wanted
net.add(pnode, Node(u - 1), cap: none)    // or net.add_uncut(..)
```

Fix: `FlowGraph.add` takes `cap: ?Int`, where `none` means unbounded. A
`min_cut` that would cross such an edge is infinite by construction.

### N4. Every finite range and `Bool` is a sequence

Ports: `cf2261d2`, `cf2268c`, `dmopc21c2p5`.

`sorted(0..n, by: f)` fails ("`sorted` takes a sequence, not a
`Range<Int>`"), though `List(0..n)` works, so ports write
`sorted([i for i in 0..n], by: ..)` (6 files). `for b in Bool` fails too,
so ports loop over `[false, true]` (5 files) or keep per-parity data in a
`List` indexed `0..2`.

Fix: one rule, that every finite domain (including `Bool`) and every
`Range` implements `Seq`. `sorted`, `min`, `max`, `count` and `for` then
accept them with no new function.

### N5. `hull` in `std.geometry`

Ports: `ucup5s05e`.

`std.geometry` has `turn`, `cross`, `area2` and `locate`, so the next thing
every geometry port writes by hand is Andrew's monotone chain. Fix:
`hull(pts) -> List<Pt>`, counter-clockwise with collinear points dropped,
plus `hull_chains` for the lower and upper halves that `ucup5s05e` needs.
This is a cohesive extension of an existing module, not a one-off.

### N6. Inverting a bijection

Ports: `ucup5s02e`.

`place[rank[i]] = i` inverts a permutation by hand. Fix: `p.inverse` on a
`[A]B` whose values are a permutation of `B`, returning `[B]A`. It is a
`Seq` method and traps if `p` is not a bijection. It is general:
permutations, rank arrays and position maps all need it.

## Real gaps already known, confirmed again

These are not new, but this wave adds ports to each:

- Tropical reductions over optionals: `sum` / `min` over `?Int` (`noip14p3`, `ucup5s06g`; wave 1, gap 1).
- Payload enums as index types, for flow nodes (`dmopc20c7p5`, `noi09p4`; wave 1, gap 3).
- Bit tests (`cf2268e`; wave 1, gap 4).
- Domain / `Int` label crossings such as `I(g.label + 1)` and `d.to_idx` (`aac5p2`, `cf2268c`, `dmopc21c2p5`, `ioi08p4`, `noi09p4`). These are in backlog batches 13, 17 and 18, but **missing from the friction queue**; the corpus has 477 `to_idx` / `from_idx` lines in 106 files.
- A `Matrix` with `pow` (`dpr`). This is in backlog batch 11 (N1), but **missing from the friction queue**.
- `find` in `cpl run` (`ucup5s05e`). The compiler knows it; the interpreter does not.
- `combinations(k)` next to the open `.pairs` (`ucup5s05e`), cyclic neighbours (`ucup5s05e`; C5), `LiChao` (`dpz`; P12), reading `Bool` (`dpo`), tuple patterns over optionals (`cf2261d2`).

## Stale ports: rewrites that need no compiler work

- `ecnerwala/cf2261d2` lines 22-23: `if first_bad == none or a < first_bad!: first_bad = a` is `first_bad min= a`; `min=` on `?Int` skips `none` (runs).
- `ecnerwala/cf2262d` lines 40-41: `via[cur] != none and via[cur] != cur` then `via[cur]!` is `if let t = via[cur] and t != cur`.
- `ecnerwala/cf2262d` lines 38-51: a `jump` / `decided` flag pair; an inner `fn jumps() -> Bool` with early returns says it.
- `ecnerwala/cf2262d` lines 60-65: `if res == none` then `res!` three times; `let r = plan() else:` runs on HEAD.
- `ecnerwala/cf2262e` lines 6-12: a hand-written `pow_mod` for a runtime modulus; `type Z = Mod<m>` with `Z(b).pow(e)` runs for a `m` read at runtime.
- `ecnerwala/cf2262e` lines 17-21: factorials and inverse factorials by hand; `Comb<m>()` with a runtime `m` runs (`fact`, `choose`).
- `ecnerwala/cf2262e` lines 23, 28, 30, 36: `[0 for _ in 0..k]` lists; with `Z` they are `var x: [0..k]Z = 0`.
- `ecnerwala/cf2262e` lines 25, 33, 40-42: `% m` after every product and `m - term` for a sign; all of it goes away with `Mod<m>`.
- `ecnerwala/cf2268c` lines 8, 29-30, 39: `INF` / `-INF` sentinels for "no boundary yet"; `?Int` with `min=` / `max=` says it.
- `ecnerwala/cf2268c` lines 37-38: `x = min(x, y)` twice; `x min= y`.
- `ecnerwala/cf2268c` lines 33: `stk.len > 0 and top >= stk.last!.top` is `while let s = stk.last and top >= s.top`.
- `ecnerwala/cf2268d` lines 9-10: `up = n` and `first = -1` as "none"; `?Int`.
- `ecnerwala/cf2268d` lines 14: `stk.len > 0 and p[stk.last!] < p[i]` is `while let t = stk.last and p[t] < p[i]`.
- `ecnerwala/cf2268d` lines 21: `n * (n - 1) / 2` is `choose(n, 2)`.
- `ecnerwala/cf2268e` lines 14: `[Z(1) for _ in 0..=n]` is `List.filled(n + 1, Z(1))`.
- `ecnerwala/cf2268e` lines 19: `[a[n - i] for i in 0..=n]` is `a.rev`, which passes as a `List` (runs).
- `ecnerwala/ucup5s01a` lines 10: `deg[p - 1]`; a `domain V = 1..=n` indexes directly.
- `ecnerwala/ucup5s01a` lines 21, 24: `[[Z(0) for _ in 0..size] for _ in 0..size]` is `var dp: [0..size, 0..size]Z = 0`.
- `ecnerwala/ucup5s02e` lines 15: `reach = max(reach, ..)` is `reach max= ..`.
- `ecnerwala/ucup5s02e` lines 17: `count(g for g in good)` is `count(good)` (runs).
- `ecnerwala/ucup5s05e` lines 7: `use std.seq.{bsearch_first}`: it is in the prelude.
- `ecnerwala/ucup5s05e` lines 11-12: reads `(x: Int, y: Int)` then builds `Pt(input[i].x, input[i].y)`; `[0..n]Pt = read()` works (cheerio1s3 does it).
- `ecnerwala/ucup5s05e` lines 32: compares `(p.x, p.y)` tuples; `Pt` is already ordered (`Pt(1, 2) < Pt(1, 3)` runs).
- `ecnerwala/ucup5s05e` lines 41: `off_diagonal != none` then `off_diagonal!` is `if let s = off_diagonal`.
- `ecnerwala/ucup5s06g` lines 12-20: Kahn peeling with a `head` index by hand; `peel(g)` is in the prelude.
- `wave2/dpr` lines 22-23: `for ways in walks: total += ways` is `sum(walks)`, which works on a 2-D array.
- `aac5p2` lines 11-12: `let p = ask(..)` then `product[g] = p`; `product[g] = ask(..)` reads straight into the place (runs).
- `scout_batch_15/noi09p4` lines 4: `use std.deque.{Deque}`: `Deque` needs no `use`.
- `scout_batch_15/noi09p4` lines 13-15, 19-22, 35-42: adjacency, reverse adjacency, in-degrees and a Kahn queue by hand; `Graph.directed(deps)` plus `peel` say it.
- `scout_batch_34/dmopc21c2p5` lines 58-60: a push loop; `[sum(..) for i in 0..=sz_u]`.
- `scout_batch_30/cco21p5` lines 23-26: a four-line `opt_min` wrapper; `Monoid<?Int>(none, min)` runs as is.
- `scout_batch_30/cco21p5` lines 28: a positional `Action(0, .., .., over: ..)`; the named form `Action(id:, compose:, apply:, over:)` is documented.
- `scout_batch_17/ioi08p4` lines 22-24: a temporary swap; `spot.swap(p, o)` runs on a domain array.
- `scout_batch_17/ioi08p4` lines 68: `sorted(loops, by: x => -x)` is `sorted(loops, by: desc)`.
- `scout_batch_2/ioi14p2` lines 16: a positional `Action`; use the named form.

## Rejected

- `ecnerwala/ucup5s01a` lines 16-18: multiplying a polynomial by `(d x + 1)` in place; three plain lines, not worth a `Poly` import.
- `ecnerwala/ucup5s05e` lines 33-36: widening a window to five candidates; specific to the problem.
- `noip14p3` lines 23: `Int(h) <= low`; one explicit conversion is fine.

## New detector rules for `scripts/friction_lint.py`

The script is still not on `main`. These rules extend wave 1's 15. Each is
a regex over code lines, with the corpus hit count today:

16. `\[[^]]*\.\.-\d` (a negative slice bound): **error**, it silently yields an empty list. 0 hits today; it guards against new code.
17. `\w\[\w+\.len - 2\]`: report N1 until `last(1)` lands. 5 files.
18. `\((l|r|m|lo|hi|i|j) [+-] 1\)!`: report N2. 10 files, 20 lines.
19. `sorted\(\[\w+ for \w+ in [^]]*\], by`: report N4, and suggest `sorted(0..n, by:)` once it lands. 6 files.
20. `in \[false, true\]`: report N4. 5 files.
21. `(INF|inf\w*|cap_inf)\)$` on a `.add(` line: report N3.
22. `fn pow_mod\(`, or `fact\[i\] = fact\[i - 1\] \* i % m`: use `Mod<m>` and `Comb<m>`, which accept a modulus read at runtime.
23. `var (\w+) = \1\n` and `head < \w+\.len` beside `indeg\[`: a hand-written Kahn peel, so use `peel(g)`.
24. `\b(\w+) != none and .*\1!` on one line: use `if let`.
25. `if (\w+) == none or \w+ [<>] \1!: \1 = `: use `x min= v` or `x max= v`.
26. `Action<[^>]*>\(\S+, \(`: a positional `Action`, so use the named form.
27. `\.(to_idx|from_idx)\b` and `\w+\(\w+\.label [+-] `: count them for the label-crossing metric (477 lines today) rather than warn per line.
