# Friction Hunter — Wave 8 (ports 211–240)

Base: `main` at `cfed027` (Batch #5 `a7e575b` and the three-way fuzzer
`cfed027` included). Read-only audit of 30 ports that waves 1–7 did not cover.
Every candidate gap was reproduced on HEAD with a minimal file before it was
counted. Patterns that HEAD already handles are listed as stale, not as gaps.

Labels as in earlier waves: **[WA-TRAP]**, **[REAL GAP]**, **[STALE PORT]**,
**[REJECTED]**.

## Ports (30)

`ioi08p2`, `etsr23r2p2`, `ioi05p1`, `ccc10s4`, `mmcc14p3`, `coci10c7p4`,
`noi07p6`, `noip19p2`, `noi05p1`, `noip04p4`, `egoi22p2`, `ioi00p4`,
`btoi15p2`, `noi09p2`, `btoi17p5`, `ioi01p1`, `ccc17s4`, `ceoi18p5`,
`ioi00p5`, `btoi19p4`, `apio09p3`, `dpv` (scout copy), `cco17p2`,
`noip19p4`, `coci21c2p4`, `ioi09p5`, `ccc20s4`, `yac1p4`, `ioi04p5`,
`ioi14p5`.

`scout_batch_27/noi07p1` came up in the candidate list but is the same
problem as wave 7's `scout_batch_23/noi07p1`. It was swapped for `ioi14p5`
in the audit, but it stays in the differential runs.

## Averages

- **Real gaps:** 15 across the 30 ports, so `real_gaps / 30` = **0.50**
  (wave 7: 0.67, wave 6: 0.43).
- **Awkward lines:** 220 across the 30 ports, so `total_awkward_lines / 30` =
  **7.33** (wave 7: 12.30). About 85% of them already have a clean form on HEAD.
- **WA traps:** **1 new**, a silent misread with a run/C++ divergence (below).
- **Cumulative, 240 ports:** 147 real gaps and 1712 awkward lines, so
  **0.61** real gaps and **7.13** awkward lines per problem.

Per-port awkward lines / real gaps: ioi08p2 12/1, etsr23r2p2 10/2,
ioi05p1 8/1, ccc10s4 6/2, mmcc14p3 11/0, coci10c7p4 8/1, noi07p6 10/0,
noip19p2 7/1, noi05p1 10/0, noip04p4 6/1, egoi22p2 7/1, ioi00p4 5/0,
btoi15p2 4/0, noi09p2 9/2, btoi17p5 7/0, ioi01p1 13/0, ccc17s4 2/0,
ceoi18p5 12/0, ioi00p5 4/0, btoi19p4 3/0, apio09p3 4/1, dpv 14/0,
cco17p2 4/0, noip19p4 10/0, coci21c2p4 9/0, ioi09p5 7/0, ccc20s4 5/1,
yac1p4 5/0, ioi04p5 2/0, ioi14p5 6/1.

## Differential runs: `cpl run` vs sanitized C++17

24 of the 30 port files were run on 15 random inputs each from their scout
generators, through `cpl run` and the `cpl build --checked` binary (g++ -O1
with ASan, UBSan and `_GLIBCXX_ASSERTIONS`). That is **360 runs with 0
divergences** and no sanitizer reports. The harness was stopped before
`btoi17p5`, `btoi19p4`, `ceoi18p5`, `coci21c2p4`, `ioi04p5` and `yac1p4`
finished, so those six were read but not run differentially.

The generators don't produce degenerate inputs. The trap below was found by
targeted read probes, and it is the one confirmed divergence this wave.

## [WA-TRAP] 1. A fresh-domain string inside a tuple read is silently empty

```cp
let (s, k): ([domain Pos]Char, Int) = read()
print(Pos.len, k)
```

`cpl check` reports no errors. On input `abc 4`:

- `cpl run` stops with `R0001: expected a number, found abc`.
- `cpl run --release` and the `cpl build --checked` ASan/UBSan binary both
  print `0 0` and exit 0.

On input `3 abc 4`, both print `0 3`. So the tuple element never reads the
string: `Pos` is always empty, and the next token goes to `k`. The same
string read on its own line (`let s: [domain Pos]Char = read()`) and the
tuple form with `Str` (`let (s, k): (Str, Int) = read()`) both work. That
makes this exactly the kind of line a contestant writes, and the submitted
binary gives a silent wrong answer. `etsr23r2p2` avoids it only because it
reads a `Str` and indexes with `l.to_idx`.

**Fix:** a fresh `[domain D]T` inside a tuple read gets the same reader as
the standalone form, or `cpl check` rejects it. Both backends must report the
same input error. Add this file to the three-way fuzzer's read cases.

## [REAL GAP] findings (each reproduced on HEAD)

### New

1. **Graph helpers can't infer the vertex domain from a closure.**
   `peel(v => adj[v])` fails with `doesn't know the name V` at 1:1, while
   `peel::<V>(v => adj[v])` works. `apio09p3` writes `scc::<Spot>` and
   `condense::<Spot>` for the same reason, and the natural rewrite of
   `ioi08p2`'s hand-written leaf queue hits it. This is the general form of
   the cold-writer benchmark's tuple-vertex `bfs` failure:
   `bfs::<(R, C)>(..)` still fails, even with the turbofish.
2. **No `Dir8`** (`coci10c7p4`): `std.grid` has only `Dir4`, so 8-neighbour
   moves are a hand-written `dr`/`dc` double loop.
3. **No `Char.digit`** (`etsr23r2p2`): `s[i] - '0'`.
4. **No `xs.windows(k)` or `zip`** (`noi09p2`): adjacent pairs are
   `for k in 0..xs.len - 1: f(xs[k], xs[k + 1])` twice.
5. **`Map.get_or_insert` doesn't return a place** (`ccc10s4`):
   `m.get_or_insert(k, []).push(x)` fails with E0401, so the port writes
   `if k not in m: m[k] = []`.

### Recurring (counted again)

- **`List.filled(n, 0)` doesn't coerce to `List<Mod<P>>`** (`egoi22p2`, ×3).
- **`Char` as an index domain** (wave 7): `noip04p4` (`Letter(c - 'A')`),
  `ccc20s4` (`.ord - 'A'.ord == g.to_idx`).
- **Label crossings with `Int` cursors:** `ioi05p1` (`C.from_idx(right)!`),
  `noi09p2` (`words[Sentence(i)]`), `ioi14p5` (`joins[u.to_idx - 1]`),
  `noip19p2` (`s[v.to_idx]` on a `Str`).
- **Sum domains:** `ccc10s4` uses `Yard(0)` as "outside" next to the pens.
- **Scalar `D.clamp(x)`:** `etsr23r2p2` writes `Val(min(x, CAP))` three times.

**Watch, not counted:** `ioi05p1` mirrors a position with
`R.from_idx(R.len - 1 - r.idx)!`; `r.mirror` doesn't exist. Only one port so far.

## [STALE PORT] patterns (clean form verified on HEAD)

- **`(mid + 1)!` and `(r - 1)!` inside guarded loops** (`etsr23r2p2`):
  after Batch #5 the plain `mid + 1` checks.
- **`x = max(x, ..)` and `x = min(x ?? n, ..)`** (`ioi08p2` ×5, `mmcc14p3`,
  `coci10c7p4`, `ioi00p4`, `apio09p3`): `max=` and `min=`, including on `?Int`.
- **`const FAR = 1e9` sentinels** (`ioi00p4`): `?Int` plus `min=`.
- **Hand 2D Fenwick** (`ioi01p1`, 12 lines): `Fenwick2D` is in the prelude.
- **Hand divisors** (`ceoi18p5`, 11 lines): `std.math.divisors`.
- **Hand memo through a `Map`** (`cco17p2`): `@memo` works on unbounded
  `Int` arguments (`f(1e9)` checked).
- **Hand rerooting prefix/suffix and `1 % m`** (scout `dpv`):
  `Rooted.excl_fold` and `Mod<m>` with a runtime modulus.
- **Negated keys in a `MinHeap` plus `Node(-top.1 + 1)`** (`noi07p6`), and
  a `MinHeap<Int>` of `to_idx` (`ioi09p5`): `MaxHeap<(Int, V)>` and
  `MinHeap<V>` work.
- **Index-plus-head queues** (`noi05p1`, `ioi09p5`): `Deque`.
- **`lo <= x and x <= hi`** (`coci10c7p4` ×2): `x in lo..=hi`.
- **`Mod` literal wrapping and nested `[[Z(0) ..]]` lists** (`noip19p4`,
  `coci21c2p4`, `egoi22p2`, `btoi17p5`): typed declarations,
  `[A, B]Z = 0`, `Mod * Int`.
- **Single reads pushed into a struct** (`noi07p6`, `btoi17p5`,
  `ioi14p5`): typed array reads.
- **Redundant prelude imports** in 11 of the 30 files.

## [REJECTED]

- Dynamic binary lifting (`btoi15p2`), backtracking with manual undo
  (`noip04p4`), and `@memo` argmin reconstruction (`ioi00p5`) are
  algorithms, not language gaps.
- `count(gen)` taking a `Bool` generator is consistent; `count(x % 3 == 0 for
  x in xs)` works.

## Proposed Syntax Batch #7 additions

1. **[WA-TRAP] Tuple reads with fresh domains:** same reader as the standalone
   form, or a check-time error. Both backends report the same input error.
2. **Domain inference for graph helpers:** `peel`, `scc`, `condense`, `bfs`
   and `dijkstra` infer `V` from the closure's return type, tuples included.
   This replaces the cold-writer item with its general form.
3. **`std.grid.Dir8`**, next to `Dir4`, with `step`/`ray`.
4. **`Char.digit`**, plus `[Char]T` and `Char` ranges (carried from wave 7).
5. **`xs.windows(k)` and `zip`** in `std.seq`.
6. **`Map.get_or_insert` returns a place**, so `.push` and `+=` work on it.
7. **`List.filled(n, 0)`** coerces the fill to the element type, like every
   other `Mod` literal.
