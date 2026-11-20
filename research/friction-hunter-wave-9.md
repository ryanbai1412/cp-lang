# Friction Hunter — Wave 9 (ports 241–270)

Base: `main` at `dba47bc`. This is a read-only audit of 30 ports that waves 1–8
did not cover: the 17 largest compiling `proto/ports/ecnerwala/` ports, plus
the 13 largest compiling scout and canonical ports. Every candidate gap was
reproduced on HEAD with a minimal file before it was counted. Patterns that
HEAD already handles are listed as stale, not as gaps.

Labels as in earlier waves: **[WA-TRAP]**, **[REAL GAP]**, **[STALE PORT]**,
**[REJECTED]**.

## Ports (30)

**ecnerwala (17):** `wtf26d`, `cf2222g`, `cf2229i`, `cf2237i1`, `wtf26a`,
`ucup4s25c`, `cf2237e`, `wtf26c`, `cf2229g`, `cf2245f`, `ucup4s25b`,
`cf2207c`, `ucup4s24f`, `cf2229e`, `cf2229f`, `cf2211c2`, `wtf26b`.

**scout and canonical (13):** `noi21p5`, `egoi23p3`, `cco23p4`, `noi23p4`,
`cco01p4io`, `ioi12p4`, `dmopc21c9p4`, `dpz`, `ioi14p1`, `ioi22p5`,
`apio07p2`, `noi02p3`, `ioi06p4`.

## Averages

- **Real gaps:** 12 across the 30 ports, so `real_gaps / 30` = **0.40**
  (wave 8: 0.50). 10 come from the ports and 2 from the targeted probes.
- **Awkward lines:** 203 across the 30 ports, so `total_awkward_lines / 30` =
  **6.77** (wave 8: 7.33). About 85% of them already have a clean form on HEAD.
- **WA traps:** **1 new**: `Mod` multiplication overflows for moduli above
  about 3.04e9 (below).
- **Cumulative, 270 ports:** 159 real gaps and 1915 awkward lines, so
  **0.59** real gaps and **7.09** awkward lines per problem.

Per-port awkward lines / real gaps: wtf26d 14/0, cf2222g 16/0, cf2229i 12/0,
cf2237i1 10/1, wtf26a 12/0, ucup4s25c 9/0, cf2237e 6/1, wtf26c 5/1,
cf2229g 8/0, cf2245f 6/0, ucup4s25b 7/1, cf2207c 6/1, ucup4s24f 4/0,
cf2229e 6/0, cf2229f 5/0, cf2211c2 5/0, wtf26b 1/0, noi21p5 6/1,
egoi23p3 9/0, cco23p4 8/1, noi23p4 9/1, cco01p4io 4/0, ioi12p4 7/1,
dmopc21c9p4 2/0, dpz 9/1, ioi14p1 5/0, ioi22p5 2/0, apio07p2 4/0,
noi02p3 3/0, ioi06p4 3/0.

## Differential runs: `cpl run` vs sanitized C++17

All 17 ecnerwala ports were run on 15 inputs each from their `gen/NAME.py`
generators (`gen/NAME.py SEED SEED%8+1`, the same call `gate.sh` makes).
Each input went through `cpl run` and a sanitized binary (`cpl build --checked`,
or g++ `-fsanitize=address,undefined -fno-sanitize-recover=all
-D_GLIBCXX_ASSERTIONS` for `wtf26c`, which needs `lib.math.poly` from its
own folder). That is **255 runs with 0 divergences** and no sanitizer reports.

The 13 scout and canonical ports have no generator script; their `read(gen:)`
generators are only reachable from `cpl stress`. They were read but not run
differentially.

## Targeted WA-trap probes (14 files)

Each probe was run through `cpl run` and `cpl run --release`, and the
`Mod` probe also through a `--checked` binary.

- **Nested tuple read** `(Int, (Int, Int))`: correct in both.
- **`@multitest` with `@memo` and a global counter:** the memo table and the
  globals reset for each case in both (`3 4` then `15 4`).
- **`@multitest` with a `Set`:** resets per case in both.
- **Runtime `Mod<7>`:** `-1` gives 6, `Z(m)` gives 0, and products are
  correct in both.
- **Runtime `Mod<m>` with m above about 3.04e9:** **WA trap** (below).
- **`Mod<1>` and `Mod<0>`:** `Mod<1>` is fine. `Mod<0>` gives a clear
  division-by-zero error in `cpl run`, and the release build crashes (not
  silent).
- **Division in a non-prime runtime `Mod<6>`:** both trap, but with a bad
  message (below).
- **Non-exhaustive `match` on a fieldless enum:** passes `cpl check` (below).
- **Empty reductions:** `sum([])`, `max(..) ?? d` and `min(gen) ?? d` are
  correct in both. `product` is still missing (recurring).
- **`@multitest` where one case has an empty domain:** `D.first` traps in
  `cpl run`, and the release build now crashes instead of printing 0. That
  closes the silent half of wave 6's empty-domain trap.
- **`sorted(by:)` stability:** stable in both.
- **Fresh-domain arrays in tuple reads:** this extends wave 8's trap rather than
  being a new one.
  - A standalone `let a: [domain P]Int = read()` is rejected with E0501
    ("can't tell how many values").
  - `let (a, k): ([domain P]Int, Int) = read()` passes `cpl check`, and
    **both** backends read zero elements (`3 1 2 3 4` gives `0 3`). The
    differential fuzzer can't see this, because both sides agree.
  - The nested form `(Int, ([domain P]Char, Int))` shows wave 8's divergence
    (`cpl run` errors, release prints `1 0 0`).
  - The fix should be in the tuple path itself: give each element the
    standalone reader, so the E0501 check runs there too.

## [WA-TRAP] 1. `Mod` multiplication overflows silently for large moduli

```cp
let m: Int = read()
type Z = Mod<m>
let a = Z(m - 1)
print(a * a, Z(m - 2).pow(2))
```

The expected output is `1 4` for any modulus.

- **m = 3037000493:** correct.
- **m = 4000000007:** `cpl run --release` prints `-1427749545 2572250465` and
  exits 0.
- **m = 1000000000000000003:** release prints `-527149226598858745` for
  `a * a`.
- **Constant modulus:** `type Z = Mod<1000000000000000003>` behaves the same,
  and `cpl check` says nothing.
- **Other paths:** `cpl run` traps with "integer overflow", and the
  `--checked` binary reports UBSan signed overflow. The plain release binary,
  which is the one that gets submitted, prints negative residues.

The cause is `stdlib/mod.cp:56`, `fn mul(a, b) = Mod(a.v * b.v % P)`, which
works in 64 bits. Any modulus above `floor(sqrt(2^63))` = 3037000499
overflows. Problems that say "print the answer mod m (m ≤ 1e18)", or that hash
mod 2^61−1, will get a silent WA.

**Fix:** do the multiply in 128 bits in `Mod.mul` and `pow` whenever
`P > 3037000499`. For a constant P this choice is free at compile time; a
runtime modulus needs one branch. Add a fuzzer case with moduli near 2^62.

## [REAL GAP] findings (each reproduced on HEAD)

### New

1. **`match` on a fieldless enum isn't checked for exhaustiveness.**
   `enum E = A | B | C` with arms only for `A` and `B` passes `cpl check`, and
   both backends trap at runtime with "no arm of this `match` fits". This is
   loud, not a WA, but it is a missed compile-time error.
2. **Dividing in a non-prime `Mod<m>` gives a useless error.** `Z(1) / Z(2)`
   with m = 6 traps with "`!` on `none`" pointing at line 1, instead of
   naming the non-invertible divisor at the call site.
3. **No Li Chao tree or line container** (`dpz`, about 30 lines). The port
   hand-writes a Li Chao tree over `Node = 1..4*n` with
   `Node(2 * node.label)` crossings. Convex-hull and Li Chao DP is common
   enough to belong in the stdlib next to `SegTree`.

### Recurring (counted again)

- **`List.filled(n, Z(0))`** (`wtf26c` ×2): the fill still doesn't coerce to
  `List<Mod<P>>`. This is the third wave in a row.
- **`Char('0'.ord + e)`** (`cf2237i1`): there is no `Char.digit` or
  `from_digit`.
- **1-based reads into a domain** (`cf2237e`, `I(a_raw[i] - 1)` ×2, and
  `b[i]!.ord + 1` when printing).
- **Label crossings:** `ucup4s25b` (`B(l)..=B(r)`) and `noi23p4`, where a
  heap-shaped implicit tree needs `V(o * 2)` and `u.to_idx + 1` six times.
- **Sum domains:** `cf2207c` packs two node kinds into one id with `2 * i` and
  `2 * i + 1`.
- **Run-length grouping:** `cco23p4` hand-writes the `gaps` loop. Wave 5
  proposed `runs`.
- **Growable arena domains:** `noi21p5` hard-codes `type Id = 0..=250_005` for
  a treap arena.
- **Coordinate compression to a domain:** `ioi12p4` uses a
  `Map<(Int, Int), Int>` plus `Run(west)`.

**Watch, not counted:** cyclic successor `(c + 1) ?? City.first` (`ioi06p4`).
It is short, but every ring problem rewrites it.

## [STALE PORT] patterns (clean form verified on HEAD)

- **Adjacency and recursive DFS** (`cf2222g`, `cf2229i`, `cf2229e`,
  `ucup4s25c`): `List.filled(n, [])` with `u - 1`, then a recursive DFS with
  `some(nxt) == prv`. The clean form is a typed `[E](u: V, v: V)` read,
  `Graph.undirected` and `Rooted`. These are 22 of the 203 lines.
- **Hand DSU** (`wtf26a`): `par = -1` with `find_root`, while the same file
  already uses `Dsu<V>` in its second half.
- **Hand string helpers:** `repeat_str` (`egoi23p3`, 6 uses) is
  `s.repeat(k)`. The `flip` loop and `count_sub` (`cco23p4`) are
  `Str([.. for ch in w])` and `find_all(w, pat).len` (overlapping, checked),
  though `find_all` still needs a `use`.
- **`-1` sentinels and `1e18` infinities** (`cf2245f`, `cf2211c2`,
  `ucup4s25b`, `cf2229i`, `ioi12p4`): `?Int` with `min=`/`max=`.
- **Optional max/min:** `if let prev = max_ch: max_ch = max(prev, ch) else:
  max_ch = ch` (`cf2229e`) is `max_ch max= ch`, and `best_cost == none or
  cost <= best_cost!` (`cco01p4io`) is `min=` on `(cost, i)`.
- **Two optionals:** `l != none and r != none` followed by `cost[l ?? g]`
  (`apio07p2`) is `if let a = l and let b = r`.
- **Small enums written as `Int` or a domain:** `Who = 0..2; big = Who(1)`
  (`noi02p3`), `kind` coded 1/2 (`ioi14p1`), and `ca * 2 + cn` as an index
  (`cf2237i1`). The clean forms are `[Bool, Bool]Z`, `for b in Bool` or an
  enum.
- **Small fixes:** `size[size.len - 1] += 1` is `size.last! += 1`
  (`ioi12p4`); `[loc[S(i)] for i in 0..n]` is `[loc[i] for i in S]`
  (`ioi14p1`); `mask - (1 << i)` is `mask.without(i)`; hand argmax loops
  are `argmax`; `Z(fa) * Z(fb) * Z(w)` is `Mod * Int`.
- **Flag-controlled loops** (`wtf26d`): `extended`/`broke` flags in nested
  loops are `break 'outer`.

## [REJECTED]

- Max-plus convolution (`cf2229i`), flattening a 2D DP for `convolve`
  (`wtf26c`), lazy-deletion heaps (`ucup4s24f`), the parity-toggle `peel` with
  an `@brute` reference (`dmopc21c9p4`), and interactive binary searches with
  side effects (`ioi22p5`, `cco01p4io`) are algorithms, not language gaps.
- An implicit treap (`noi21p5`) is a heavy data structure. It is a
  stdlib-later candidate, not a language gap; only its fixed-size arena counts.

## Proposed Syntax Batch #8 additions

1. **[WA-TRAP] `Mod.mul`/`pow` in 128 bits for P > 3037000499**, plus
   fuzzer cases with large moduli.
2. **[WA-TRAP, wave 8] Tuple reads use the standalone reader for each element**,
   so a fresh-domain `[domain P]Int` gets E0501 and `[domain P]Char` reads a
   word. Both backends currently agree on the wrong answer for `Int`.
3. **Exhaustiveness check for `match` on fieldless enums and `Bool`.**
4. **Non-invertible `Mod` division:** a call-site error that names the divisor.
5. **`LiChao<D>` / line container** in the stdlib.
6. **Still open from earlier waves:** `List.filled` coercion, `Char.digit`,
   `runs`, and `product`.
