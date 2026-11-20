# ecnerwala/contest-programming: roadmap to 100%

The target is every real submission in
<https://github.com/ecnerwala/contest-programming> ported to CP Lang 0.2.2,
each one passing `proto/ports/ecnerwala/gate.sh`, emitting under 64 KB of
C++17, and running within the performance bar (p95 under 1.1x and worst case
under 1.5x to 2x of the original at `g++ -O2`). This file says how far that is
today, what blocks the rest, and in what order to fix it.

## Where things stand

The upstream tree has 7,856 C++ files; about 6,990 of them have a non-trivial
`main`. This scan covers the 202 most recent real submissions, from
2026-11-08 back to 2026-02-23 (Codeforces 2201). 39 are ported and pass the
gate: the 34 from waves 1 and 2 plus the five added here. 163 are not.

"Most recent" is by the date a file was first committed upstream. Three
contests that only appear in upstream's 2026-10-31 bulk import (Codeforces
2261, Universal Cup 5 stages 3 and 5) have no such date and are placed in
October, where waves 1 and 2 put them.

Left out of the count: 22 empty templates (`AtCoder/AGC079/D`, `AtCoder/AGC078/B`, `CF/2268/F`, `CF/2262/F`, `CF/2245/D1`, `AtCoder/ARC223/E`, `AtCoder/ARC223/D`, `AtCoder/ARC223/C`, `AtCoder/ARC223/B`, `AtCoder/ARC223/A`, `Misc/MITIT/2026-spring-finals/E`, `CF/2222/H`, `CF/2219/E`, `CF/2219/B2`, `CF/2219/B1`, `AtCoder/AGC077/E`, `AtCoder/AGC077/B`, `POI/PA2026/R1/kon`, `POI/PA2026/R1/grm`, `CF/2207/H3`, `CF/2207/H2`, `CF/Blitz/675573/G`), and seven files
that are not submissions: [`AtCoder/AGC079/B`](https://github.com/ecnerwala/contest-programming/blob/master/AtCoder/AGC079/B/B.cpp) (unfinished upstream); [`CF/2261/F2`](https://github.com/ecnerwala/contest-programming/blob/master/CF/2261/F2/F2.cpp) (same program as F1, which is ported); [`AtCoder/AGC078/C`](https://github.com/ecnerwala/contest-programming/blob/master/AtCoder/AGC078/C/C.cpp) (game-value explorer, reads no input); [`AtCoder/AGC078/A`](https://github.com/ecnerwala/contest-programming/blob/master/AtCoder/AGC078/A/A.cpp) (game-value explorer, reads no input); [`CF/2229/I`](https://github.com/ecnerwala/contest-programming/blob/master/CF/2229/I/I-ksun.cpp) (a teammate's alternate of I); [`CF/2229/F`](https://github.com/ecnerwala/contest-programming/blob/master/CF/2229/F/F-bad.cpp) (abandoned alternate of F); [`ACM/UCup4/ExtraState4-Chongqing/M`](https://github.com/ecnerwala/contest-programming/blob/master/ACM/UCup4/ExtraState4-Chongqing/M/M-slow.cpp) (slow reference for M).

**How the 163 were classified.** A script read each file's `main` and the
helpers it calls, and sorted it by the first thing it needs that CP Lang
lacks. Only the submissions named in the notes below were read by hand, and
none of the 163 was attempted. So the counts under each heading are a lower
bound on that blocker, and the last group (95) is "nothing found by reading",
not "ports cleanly". The claims about `cpl` itself (128-bit, unsigned,
`sqrt`, `flush`) were each checked with a small program on `main` at
`680633a`.

## The five NTT ports added here

`poly.cp` is `examples/local-lib/math/poly.cp`. The ports import it as
`use lib.math.poly.{...}` through `proto/ports/ecnerwala/lib.toml` and a
`math` link to `examples/local-lib/math`.

- `ucup5s06a` (Universal Cup 5 Stage 6 A): two logarithmic derivatives of a
  Catalan series, with `inv_trunc`.
- `ucup5s05f` (Stage 5 F): suffix array, a stack walk over the LCP array in
  place of upstream's Cartesian tree, power sums by logarithmic derivative,
  then a transposed multipoint evaluation written with `convolve`.
- `ucup5s03a` (Stage 3 A): four exponential convolutions and one inverse.
- `cf2268e` (Codeforces 2268 E): an autocorrelation per bit.
- `cf2262e` (Codeforces 2262 E): repeated series products under a modulus read
  from the input, with `convolve_mod`.

Four pass the plain gate on their samples and 100 random cases each; `ucup5s06a` passes it on 40 with a longer interpreter timeout (below). Measured on
one large input each, emitted C++ at `g++ -O2` against the original at `-O2`:

- `ucup5s06a`: 21 KB emitted; 1.11 s against 0.47 s (2.4x), five tests of n = 100000.
- `ucup5s05f`: 39 KB; 1.24 s against 0.48 s (2.6x), n = 100000.
- `ucup5s03a`: 24 KB; 0.39 s against 0.12 s (3.3x), n = k = 200000.
- `cf2268e`: 21 KB; 0.60 s against 0.14 s (4.2x), n = 200000.
- `cf2262e`: 33 KB; 1.83 s against 0.57 s (3.2x), n = 2000.

So all five are under 64 KB and **none meets the performance bar**. These are
single runs on inputs I made up, not the judges' limits, so read them as
"about 3x", not as exact figures.

`ucup5s06a` also needs `INTERP_TIMEOUT=200`: its upstream sample has
n = 100000, which `cpl run` takes 78 s to finish, and the gate allows 20 s by
default. `gate.sh` gained that variable for this; the default is unchanged,
so the plain gate still fails this one port on that sample.

Codeforces 2255 F was not ported. It needs series composition and a power
with a negative exponent, neither of which `poly.cp` has.

### Friction with `poly.cp` and around it

1. No logarithmic derivative, middle product, product tree, Taylor shift or
   composition. `ucup5s06a` and `ucup5s05f` write the first by hand,
   `ucup5s05f` gets middle products by running a full `convolve` and slicing,
   and `cf2262e` does the Taylor shift in O(n^2).
2. `convolve` is about 3x slower than upstream's NTT in every port above.
   Upstream keeps transforms cached and reuses buffers; `poly.cp` allocates
   per call.
3. Everything is fixed to `Z = Mod<998244353>` except `convolve_mod`, which
   takes and returns `List<Int>`. With no `ModR`, `cf2262e` reduces every
   product by hand.
4. `print` on a `List<Z>` writes one value per line, on a `List<Int>` it
   writes them on one line. `ucup5s05f` prints `.v` to get one line. The gate
   compares after collapsing whitespace, so it does not catch this.
5. A function-typed parameter is written `T -> U`; `Fn(Int) -> List<Z>` is
   rejected with a parse error that does not suggest the right form.
6. `open.last.0` is rejected because `last` is optional, and `pop()` needs
   `!`. A stack of pairs ended up as two parallel lists in `ucup5s05f`.
7. `a.reversed` does not exist on a list.
8. `cpl run` is too slow for upstream's own large samples (78 s above).

## Blockers, by how many submissions each one holds

### Bespoke segment tree (custom node layout, descent, or time-indexed): 15

These keep a hand-written tree outside `main` (upstream's `in_order_layout`): a node type with its own merge, descent by predicate, or one tree per time step. `SegTree` and `LazySegTree` take a monoid and have `max_right` / `min_left`, so some of these may fit; none was tried. Fix: port two of them onto `SegTree`, and add whatever descent hook they are missing.

[`ACM/UCup5/Stage02-EPS/B`](https://github.com/ecnerwala/contest-programming/blob/master/ACM/UCup5/Stage02-EPS/B/B.cpp), [`ACM/UCup5/Stage02-EPS/A`](https://github.com/ecnerwala/contest-programming/blob/master/ACM/UCup5/Stage02-EPS/A/A.cpp), [`CF/2262/B`](https://github.com/ecnerwala/contest-programming/blob/master/CF/2262/B/B.cpp), [`CF/2255/E2`](https://github.com/ecnerwala/contest-programming/blob/master/CF/2255/E2/E2.cpp), [`CF/2255/E1`](https://github.com/ecnerwala/contest-programming/blob/master/CF/2255/E1/E1.cpp), [`ACM/UCup4/Stage24-Wulin/M`](https://github.com/ecnerwala/contest-programming/blob/master/ACM/UCup4/Stage24-Wulin/M/M.cpp), [`Misc/MITIT/2026-qualification-2/C`](https://github.com/ecnerwala/contest-programming/blob/master/Misc/MITIT/2026-qualification-2/C/C.cpp), [`AtCoder/AGC077/F`](https://github.com/ecnerwala/contest-programming/blob/master/AtCoder/AGC077/F/F.cpp), [`CF/2211/H`](https://github.com/ecnerwala/contest-programming/blob/master/CF/2211/H/H.cpp), [`ACM/UCup4/Stage20-India/H`](https://github.com/ecnerwala/contest-programming/blob/master/ACM/UCup4/Stage20-India/H/H.cpp), [`CF/2207/F`](https://github.com/ecnerwala/contest-programming/blob/master/CF/2207/F/F.cpp), [`ACM/UCup4/Stage19-Belarus/M`](https://github.com/ecnerwala/contest-programming/blob/master/ACM/UCup4/Stage19-Belarus/M/M.cpp), [`ACM/UCup4/ExtraState4-Chongqing/M`](https://github.com/ecnerwala/contest-programming/blob/master/ACM/UCup4/ExtraState4-Chongqing/M/M.cpp), [`ACM/UCup4/ExtraState4-Chongqing/J`](https://github.com/ecnerwala/contest-programming/blob/master/ACM/UCup4/ExtraState4-Chongqing/J/J.cpp), [`CF/2201/D`](https://github.com/ecnerwala/contest-programming/blob/master/CF/2201/D/D.cpp).

### Power-series operations `poly.cp` does not have yet: 10

All use upstream's power-series library for something `examples/local-lib/math/poly.cp` lacks. Read so far: CF 2255 F needs series composition and a power with a negative exponent. The other nine were classified by the library they call, not read line by line. Fix: add composition, power projection, negative and rational powers, middle product, subproduct tree, Taylor shift and evaluation at an arithmetic progression to `poly.cp`.

[`CF/2255/F`](https://github.com/ecnerwala/contest-programming/blob/master/CF/2255/F/F.cpp), [`AtCoder/WTF26/C`](https://github.com/ecnerwala/contest-programming/blob/master/AtCoder/WTF26/C/C.cpp), [`ACM/UCup4/Stage24-Wulin/C`](https://github.com/ecnerwala/contest-programming/blob/master/ACM/UCup4/Stage24-Wulin/C/C.cpp), [`ACM/UCup4/Stage23-Nanyang/H`](https://github.com/ecnerwala/contest-programming/blob/master/ACM/UCup4/Stage23-Nanyang/H/H.cpp), [`ACM/UCup4/Stage22-Kyoto/G`](https://github.com/ecnerwala/contest-programming/blob/master/ACM/UCup4/Stage22-Kyoto/G/G.cpp), [`ACM/UCup4/Stage22-Kyoto/E`](https://github.com/ecnerwala/contest-programming/blob/master/ACM/UCup4/Stage22-Kyoto/E/E.cpp), [`ACM/UCup4/Stage21-Kamigo/J`](https://github.com/ecnerwala/contest-programming/blob/master/ACM/UCup4/Stage21-Kamigo/J/J.cpp), [`ACM/UCup4/Stage19-Belarus/I`](https://github.com/ecnerwala/contest-programming/blob/master/ACM/UCup4/Stage19-Belarus/I/I.cpp), [`ACM/UCup4/Stage18-Hongo/I`](https://github.com/ecnerwala/contest-programming/blob/master/ACM/UCup4/Stage18-Hongo/I/I.cpp), [`CF/2201/E`](https://github.com/ecnerwala/contest-programming/blob/master/CF/2201/E/E.cpp).

### Ordered set / map walking: 10

`std::set` / `std::map` with `lower_bound`, `prev` and `next`. `SortedBag<K>` exists and `Set` / `Map` work in both backends (checked with a probe), but no ordered map was found in `stdlib/`. Not attempted.

[`ACM/UCup4/Stage25-Zagreb/C`](https://github.com/ecnerwala/contest-programming/blob/master/ACM/UCup4/Stage25-Zagreb/C/C.cpp), [`ACM/UCup4/Stage24-Wulin/F`](https://github.com/ecnerwala/contest-programming/blob/master/ACM/UCup4/Stage24-Wulin/F/F.cpp), [`Misc/MITIT/2026-spring-finals/B`](https://github.com/ecnerwala/contest-programming/blob/master/Misc/MITIT/2026-spring-finals/B/B.cpp), [`CF/2222/G`](https://github.com/ecnerwala/contest-programming/blob/master/CF/2222/G/G.cpp), [`Misc/MITIT/2026-qualification-2/E`](https://github.com/ecnerwala/contest-programming/blob/master/Misc/MITIT/2026-qualification-2/E/E.cpp), [`AtCoder/AGC077/C`](https://github.com/ecnerwala/contest-programming/blob/master/AtCoder/AGC077/C/C.cpp), [`Misc/MITIT/2026-qualification-1/E`](https://github.com/ecnerwala/contest-programming/blob/master/Misc/MITIT/2026-qualification-1/E/E.cpp), [`Misc/MITIT/2026-qualification-1/C`](https://github.com/ecnerwala/contest-programming/blob/master/Misc/MITIT/2026-qualification-1/C/C.cpp), [`ACM/APC2026/G`](https://github.com/ecnerwala/contest-programming/blob/master/ACM/APC2026/G/G.cpp), [`ACM/UCup4/Stage19-Belarus/G`](https://github.com/ecnerwala/contest-programming/blob/master/ACM/UCup4/Stage19-Belarus/G/G.cpp).

### Interactive problems: 5

The program asks, flushes and reads the reply. `cpl` has no `flush` (`flush()` and `stdout.flush()` are both rejected by `cpl check`) and `gate.sh` has no interactor to run the port against. Fix: a flush on `print`, and an interactor mode in the gate.

[`CF/2237/G`](https://github.com/ecnerwala/contest-programming/blob/master/CF/2237/G/G.cpp), [`CF/2222/E`](https://github.com/ecnerwala/contest-programming/blob/master/CF/2222/E/E.cpp), [`ACM/UCup4/Stage22-Kyoto/H`](https://github.com/ecnerwala/contest-programming/blob/master/ACM/UCup4/Stage22-Kyoto/H/H.cpp), [`CF/2211/E`](https://github.com/ecnerwala/contest-programming/blob/master/CF/2211/E/E.cpp), [`CF/2207/H1`](https://github.com/ecnerwala/contest-programming/blob/master/CF/2207/H1/H1.cpp).

### Floating point: 5

`F64` exists but `x.sqrt` is rejected by `cpl check` ("a `F64` has no member `sqrt`"), so the math surface on floats is the open question. Stage 1 C also needs a lazy segment tree over logarithms. Not attempted.

[`ACM/UCup5/Stage01-NizhnyNovgorod/C`](https://github.com/ecnerwala/contest-programming/blob/master/ACM/UCup5/Stage01-NizhnyNovgorod/C/C.cpp), [`CF/2219/C`](https://github.com/ecnerwala/contest-programming/blob/master/CF/2219/C/C.cpp), [`ACM/NAC2026/G`](https://github.com/ecnerwala/contest-programming/blob/master/ACM/NAC2026/G/G.cpp), [`ACM/NAC2026/F`](https://github.com/ecnerwala/contest-programming/blob/master/ACM/NAC2026/F/F.cpp), [`ACM/UCup4/Stage20-India/C`](https://github.com/ecnerwala/contest-programming/blob/master/ACM/UCup4/Stage20-India/C/C.cpp).

### One-off upstream library pieces: 5

Each leans on one upstream helper: LARSCH (UCup 3 Finals L), `pairnum` (Kamigo I, India G), a Cartesian tree (CF 2207 C), an interval type (AGC 077 A). The Cartesian tree can be replaced by a stack walk, as `ucup5s05f` now does.

[`ACM/UCup3/Finals/L`](https://github.com/ecnerwala/contest-programming/blob/master/ACM/UCup3/Finals/L/L.cpp), [`AtCoder/AGC077/A`](https://github.com/ecnerwala/contest-programming/blob/master/AtCoder/AGC077/A/A.cpp), [`ACM/UCup4/Stage21-Kamigo/I`](https://github.com/ecnerwala/contest-programming/blob/master/ACM/UCup4/Stage21-Kamigo/I/I.cpp), [`ACM/UCup4/Stage20-India/G`](https://github.com/ecnerwala/contest-programming/blob/master/ACM/UCup4/Stage20-India/G/G.cpp), [`CF/2207/C`](https://github.com/ecnerwala/contest-programming/blob/master/CF/2207/C/C.cpp).

### Geometry on points (rational or floating coordinates): 4

`stdlib/geometry.cp` has integer `Pt` only. These need fractions or `long double` coordinates; two also need 128-bit cross products.

[`ACM/UCup5/Stage02-EPS/D`](https://github.com/ecnerwala/contest-programming/blob/master/ACM/UCup5/Stage02-EPS/D/D.cpp), [`ACM/UCup5/Stage01-NizhnyNovgorod/H`](https://github.com/ecnerwala/contest-programming/blob/master/ACM/UCup5/Stage01-NizhnyNovgorod/H/H.cpp), [`ACM/UCup4/Stage25-Zagreb/E`](https://github.com/ecnerwala/contest-programming/blob/master/ACM/UCup4/Stage25-Zagreb/E/E.cpp), [`ACM/UCup4/Stage22-Kyoto/M`](https://github.com/ecnerwala/contest-programming/blob/master/ACM/UCup4/Stage22-Kyoto/M/M.cpp).

### 128-bit integers: 4

Compiler bug, reproduced today: `let a: I128 = 10^18` then `a * a` overflows in `cpl run` (R0001) and in the emitted C++ (UBSan: signed overflow in `long long`). Wave 2 finding 1. Fixing it also unblocks the two geometry submissions that need 128-bit cross products.

[`ACM/UCup5/Stage02-EPS/H`](https://github.com/ecnerwala/contest-programming/blob/master/ACM/UCup5/Stage02-EPS/H/H.cpp), [`ACM/UCup5/Stage02-EPS/F`](https://github.com/ecnerwala/contest-programming/blob/master/ACM/UCup5/Stage02-EPS/F/F.cpp), [`AtCoder/WTF26/A`](https://github.com/ecnerwala/contest-programming/blob/master/AtCoder/WTF26/A/A.cpp), [`CF/Blitz/675282/F`](https://github.com/ecnerwala/contest-programming/blob/master/CF/Blitz/675282/F/F.cpp).

### Top tree / link-cut tree: 3

Upstream's top tree (about 500 lines). Nothing like it exists in `stdlib/` or `examples/local-lib`.

[`CF/2237/H`](https://github.com/ecnerwala/contest-programming/blob/master/CF/2237/H/H.cpp), [`CF/2222/F`](https://github.com/ecnerwala/contest-programming/blob/master/CF/2222/F/F.cpp), [`ACM/APC2026/D`](https://github.com/ecnerwala/contest-programming/blob/master/ACM/APC2026/D/D.cpp).

### Unsigned 64-bit wraparound and hashing: 3

Hashing that relies on unsigned wraparound. `U64` is accepted but arithmetic overflows like `Int` in both backends (probe: `x = x * 3 + 1` in a loop traps).

[`Misc/MITIT/2026-qualification-1/D`](https://github.com/ecnerwala/contest-programming/blob/master/Misc/MITIT/2026-qualification-1/D/D.cpp), [`CF/2201/F2`](https://github.com/ecnerwala/contest-programming/blob/master/CF/2201/F2/F2.cpp), [`CF/2201/F1`](https://github.com/ecnerwala/contest-programming/blob/master/CF/2201/F1/F1.cpp).

### Modulus read at run time (`ModR`): 1

`ModR` is in the approved design but not in `stdlib/`. `cf2262e` shows the cost today: every product is written `a * b % m` by hand.

[`AtCoder/AGC077/D`](https://github.com/ecnerwala/contest-programming/blob/master/AtCoder/AGC077/D/D.cpp).

### Min-cost max-flow: 1

`stdlib/flow.cp` has max flow and matching only; `MinCostFlow` lives in `examples/local-lib`. Not attempted with it.

[`ACM/UCup5/Stage03-Gorkovskaya/H`](https://github.com/ecnerwala/contest-programming/blob/master/ACM/UCup5/Stage03-Gorkovskaya/H/H.cpp).

### `std::bitset`: 1

`Bitset<D>` exists; whether it covers this use (WTF 26 E) was not checked.

[`AtCoder/WTF26/E`](https://github.com/ecnerwala/contest-programming/blob/master/AtCoder/WTF26/E/E.cpp).

### Hash map: 1

`Map` works in both backends; not attempted.

[`ACM/UCup4/Stage22-Kyoto/F`](https://github.com/ecnerwala/contest-programming/blob/master/ACM/UCup4/Stage22-Kyoto/F/F.cpp).

### No blocker identified by reading; not attempted yet: 95

Reading `main` and the helpers it calls found nothing outside what the 39 existing ports already use. That is not a claim that they port cleanly: none was attempted, and every wave so far found friction only by writing the port.

[`CF/2245/H`](https://github.com/ecnerwala/contest-programming/blob/master/CF/2245/H/H.cpp), [`CF/2245/G`](https://github.com/ecnerwala/contest-programming/blob/master/CF/2245/G/G.cpp), [`CF/2245/F`](https://github.com/ecnerwala/contest-programming/blob/master/CF/2245/F/F.cpp), [`AtCoder/WTF26/D`](https://github.com/ecnerwala/contest-programming/blob/master/AtCoder/WTF26/D/D.cpp), [`AtCoder/WTF26/B`](https://github.com/ecnerwala/contest-programming/blob/master/AtCoder/WTF26/B/B.cpp), [`AtCoder/ARC223/F`](https://github.com/ecnerwala/contest-programming/blob/master/AtCoder/ARC223/F/F.cpp), [`CF/2237/I2`](https://github.com/ecnerwala/contest-programming/blob/master/CF/2237/I2/I2.cpp), [`CF/2237/I1`](https://github.com/ecnerwala/contest-programming/blob/master/CF/2237/I1/I1.cpp), [`CF/2237/F`](https://github.com/ecnerwala/contest-programming/blob/master/CF/2237/F/F.cpp), [`CF/2237/E`](https://github.com/ecnerwala/contest-programming/blob/master/CF/2237/E/E.cpp), [`CF/2237/D`](https://github.com/ecnerwala/contest-programming/blob/master/CF/2237/D/D.cpp), [`CF/2237/C`](https://github.com/ecnerwala/contest-programming/blob/master/CF/2237/C/C.cpp), [`CF/2237/B`](https://github.com/ecnerwala/contest-programming/blob/master/CF/2237/B/B.cpp), [`CF/2237/A`](https://github.com/ecnerwala/contest-programming/blob/master/CF/2237/A/A.cpp), [`ACM/UCup4/Stage25-Zagreb/I`](https://github.com/ecnerwala/contest-programming/blob/master/ACM/UCup4/Stage25-Zagreb/I/I.cpp), [`ACM/UCup4/Stage25-Zagreb/H`](https://github.com/ecnerwala/contest-programming/blob/master/ACM/UCup4/Stage25-Zagreb/H/H.cpp), [`ACM/UCup4/Stage25-Zagreb/B`](https://github.com/ecnerwala/contest-programming/blob/master/ACM/UCup4/Stage25-Zagreb/B/B.cpp), [`CF/2229/I`](https://github.com/ecnerwala/contest-programming/blob/master/CF/2229/I/I.cpp), [`CF/2229/H`](https://github.com/ecnerwala/contest-programming/blob/master/CF/2229/H/H.cpp), [`CF/2229/G`](https://github.com/ecnerwala/contest-programming/blob/master/CF/2229/G/G.cpp), [`CF/2229/F`](https://github.com/ecnerwala/contest-programming/blob/master/CF/2229/F/F.cpp), [`CF/2229/E`](https://github.com/ecnerwala/contest-programming/blob/master/CF/2229/E/E.cpp), [`CF/2229/D`](https://github.com/ecnerwala/contest-programming/blob/master/CF/2229/D/D.cpp), [`CF/2229/C2`](https://github.com/ecnerwala/contest-programming/blob/master/CF/2229/C2/C2.cpp), [`CF/2229/C1`](https://github.com/ecnerwala/contest-programming/blob/master/CF/2229/C1/C1.cpp), [`CF/2229/B`](https://github.com/ecnerwala/contest-programming/blob/master/CF/2229/B/B.cpp), [`CF/2229/A`](https://github.com/ecnerwala/contest-programming/blob/master/CF/2229/A/A.cpp), [`ACM/UCup4/Stage24-Wulin/A`](https://github.com/ecnerwala/contest-programming/blob/master/ACM/UCup4/Stage24-Wulin/A/A.cpp), [`Misc/MITIT/2026-spring-finals/D`](https://github.com/ecnerwala/contest-programming/blob/master/Misc/MITIT/2026-spring-finals/D/D.cpp), [`Misc/MITIT/2026-spring-finals/C`](https://github.com/ecnerwala/contest-programming/blob/master/Misc/MITIT/2026-spring-finals/C/C.cpp), [`Misc/MITIT/2026-spring-finals/A`](https://github.com/ecnerwala/contest-programming/blob/master/Misc/MITIT/2026-spring-finals/A/A.cpp), [`CF/2222/D`](https://github.com/ecnerwala/contest-programming/blob/master/CF/2222/D/D.cpp), [`CF/2222/C`](https://github.com/ecnerwala/contest-programming/blob/master/CF/2222/C/C.cpp), [`CF/2222/B`](https://github.com/ecnerwala/contest-programming/blob/master/CF/2222/B/B.cpp), [`CF/2222/A`](https://github.com/ecnerwala/contest-programming/blob/master/CF/2222/A/A.cpp), [`CF/2219/D`](https://github.com/ecnerwala/contest-programming/blob/master/CF/2219/D/D.cpp), [`CF/2219/A`](https://github.com/ecnerwala/contest-programming/blob/master/CF/2219/A/A.cpp), [`Misc/MITIT/2026-qualification-2/D`](https://github.com/ecnerwala/contest-programming/blob/master/Misc/MITIT/2026-qualification-2/D/D.cpp), [`Misc/MITIT/2026-qualification-2/B`](https://github.com/ecnerwala/contest-programming/blob/master/Misc/MITIT/2026-qualification-2/B/B.cpp), [`Misc/MITIT/2026-qualification-2/A`](https://github.com/ecnerwala/contest-programming/blob/master/Misc/MITIT/2026-qualification-2/A/A.cpp), [`CF/2211/G`](https://github.com/ecnerwala/contest-programming/blob/master/CF/2211/G/G.cpp), [`CF/2211/F`](https://github.com/ecnerwala/contest-programming/blob/master/CF/2211/F/F.cpp), [`CF/2211/D`](https://github.com/ecnerwala/contest-programming/blob/master/CF/2211/D/D.cpp), [`CF/2211/C2`](https://github.com/ecnerwala/contest-programming/blob/master/CF/2211/C2/C2.cpp), [`CF/2211/C1`](https://github.com/ecnerwala/contest-programming/blob/master/CF/2211/C1/C1.cpp), [`CF/2211/B`](https://github.com/ecnerwala/contest-programming/blob/master/CF/2211/B/B.cpp), [`CF/2211/A`](https://github.com/ecnerwala/contest-programming/blob/master/CF/2211/A/A.cpp), [`ACM/UCup4/Stage21-Kamigo/F`](https://github.com/ecnerwala/contest-programming/blob/master/ACM/UCup4/Stage21-Kamigo/F/F.cpp), [`POI/PA2026/R1/zmi`](https://github.com/ecnerwala/contest-programming/blob/master/POI/PA2026/R1/zmi/zmi.cpp), [`ACM/NAC2026/E`](https://github.com/ecnerwala/contest-programming/blob/master/ACM/NAC2026/E/E.cpp), [`ACM/NAC2026/B`](https://github.com/ecnerwala/contest-programming/blob/master/ACM/NAC2026/B/B.cpp), [`ACM/UCup4/Stage20-India/F`](https://github.com/ecnerwala/contest-programming/blob/master/ACM/UCup4/Stage20-India/F/F.cpp), [`Misc/MITIT/2026-qualification-1/B`](https://github.com/ecnerwala/contest-programming/blob/master/Misc/MITIT/2026-qualification-1/B/B.cpp), [`Misc/MITIT/2026-qualification-1/A`](https://github.com/ecnerwala/contest-programming/blob/master/Misc/MITIT/2026-qualification-1/A/A.cpp), [`CF/2207/G`](https://github.com/ecnerwala/contest-programming/blob/master/CF/2207/G/G.cpp), [`CF/2207/E2`](https://github.com/ecnerwala/contest-programming/blob/master/CF/2207/E2/E2.cpp), [`CF/2207/E1`](https://github.com/ecnerwala/contest-programming/blob/master/CF/2207/E1/E1.cpp), [`CF/2207/D`](https://github.com/ecnerwala/contest-programming/blob/master/CF/2207/D/D.cpp), [`CF/2207/B`](https://github.com/ecnerwala/contest-programming/blob/master/CF/2207/B/B.cpp), [`CF/2207/A`](https://github.com/ecnerwala/contest-programming/blob/master/CF/2207/A/A.cpp), [`ACM/APC2026/L`](https://github.com/ecnerwala/contest-programming/blob/master/ACM/APC2026/L/L.cpp), [`ACM/APC2026/J`](https://github.com/ecnerwala/contest-programming/blob/master/ACM/APC2026/J/J.cpp), [`ACM/APC2026/I`](https://github.com/ecnerwala/contest-programming/blob/master/ACM/APC2026/I/I.cpp), [`ACM/APC2026/C`](https://github.com/ecnerwala/contest-programming/blob/master/ACM/APC2026/C/C.cpp), [`ACM/APC2026/B`](https://github.com/ecnerwala/contest-programming/blob/master/ACM/APC2026/B/B.cpp), [`ACM/UCup4/Stage19-Belarus/E`](https://github.com/ecnerwala/contest-programming/blob/master/ACM/UCup4/Stage19-Belarus/E/E.cpp), [`CF/Blitz/675573/F`](https://github.com/ecnerwala/contest-programming/blob/master/CF/Blitz/675573/F/F.cpp), [`CF/Blitz/675573/E`](https://github.com/ecnerwala/contest-programming/blob/master/CF/Blitz/675573/E/E.cpp), [`CF/Blitz/675573/D`](https://github.com/ecnerwala/contest-programming/blob/master/CF/Blitz/675573/D/D.cpp), [`CF/Blitz/675573/C`](https://github.com/ecnerwala/contest-programming/blob/master/CF/Blitz/675573/C/C.cpp), [`CF/Blitz/675573/B`](https://github.com/ecnerwala/contest-programming/blob/master/CF/Blitz/675573/B/B.cpp), [`CF/Blitz/675573/A`](https://github.com/ecnerwala/contest-programming/blob/master/CF/Blitz/675573/A/A.cpp), [`ACM/UCup4/ExtraState4-Chongqing/K`](https://github.com/ecnerwala/contest-programming/blob/master/ACM/UCup4/ExtraState4-Chongqing/K/K.cpp), [`ACM/UCup4/ExtraState4-Chongqing/H`](https://github.com/ecnerwala/contest-programming/blob/master/ACM/UCup4/ExtraState4-Chongqing/H/H.cpp), [`CF/Blitz/675282/G`](https://github.com/ecnerwala/contest-programming/blob/master/CF/Blitz/675282/G/G.cpp), [`CF/Blitz/675282/E`](https://github.com/ecnerwala/contest-programming/blob/master/CF/Blitz/675282/E/E.cpp), [`CF/Blitz/675282/D`](https://github.com/ecnerwala/contest-programming/blob/master/CF/Blitz/675282/D/D.cpp), [`CF/Blitz/675282/C`](https://github.com/ecnerwala/contest-programming/blob/master/CF/Blitz/675282/C/C.cpp), [`CF/Blitz/675282/B`](https://github.com/ecnerwala/contest-programming/blob/master/CF/Blitz/675282/B/B.cpp), [`CF/Blitz/675282/A`](https://github.com/ecnerwala/contest-programming/blob/master/CF/Blitz/675282/A/A.cpp), [`ACM/UCup4/Stage18-Hongo/L`](https://github.com/ecnerwala/contest-programming/blob/master/ACM/UCup4/Stage18-Hongo/L/L.cpp), [`ACM/UCup4/Stage18-Hongo/K`](https://github.com/ecnerwala/contest-programming/blob/master/ACM/UCup4/Stage18-Hongo/K/K.cpp), [`ACM/UCup4/Stage18-Hongo/G`](https://github.com/ecnerwala/contest-programming/blob/master/ACM/UCup4/Stage18-Hongo/G/G.cpp), [`ACM/UCup4/Stage18-Hongo/E`](https://github.com/ecnerwala/contest-programming/blob/master/ACM/UCup4/Stage18-Hongo/E/E.cpp), [`ACM/UCup4/Stage18-Hongo/D`](https://github.com/ecnerwala/contest-programming/blob/master/ACM/UCup4/Stage18-Hongo/D/D.cpp), [`CF/Blitz/674543/G`](https://github.com/ecnerwala/contest-programming/blob/master/CF/Blitz/674543/G/G.cpp), [`CF/Blitz/674543/F`](https://github.com/ecnerwala/contest-programming/blob/master/CF/Blitz/674543/F/F.cpp), [`CF/Blitz/674543/E`](https://github.com/ecnerwala/contest-programming/blob/master/CF/Blitz/674543/E/E.cpp), [`CF/Blitz/674543/D`](https://github.com/ecnerwala/contest-programming/blob/master/CF/Blitz/674543/D/D.cpp), [`CF/Blitz/674543/C`](https://github.com/ecnerwala/contest-programming/blob/master/CF/Blitz/674543/C/C.cpp), [`CF/Blitz/674543/B`](https://github.com/ecnerwala/contest-programming/blob/master/CF/Blitz/674543/B/B.cpp), [`CF/Blitz/674543/A`](https://github.com/ecnerwala/contest-programming/blob/master/CF/Blitz/674543/A/A.cpp), [`CF/2201/G`](https://github.com/ecnerwala/contest-programming/blob/master/CF/2201/G/G.cpp), [`CF/2201/C`](https://github.com/ecnerwala/contest-programming/blob/master/CF/2201/C/C.cpp), [`CF/2201/B`](https://github.com/ecnerwala/contest-programming/blob/master/CF/2201/B/B.cpp).

## Order of work

Ranked by submissions unlocked in this scan, with the wider tree as a
tie-break (see the next section).

1. **Port the 95 with no blocker found.** Nothing in the compiler needs to
   change first, and this is where the unknown friction is. It is also the
   only way to turn the static counts above into real ones.
2. **Emitted-code speed for NTT, then the interpreter.** Every power-series
   port is about 3x over the bar today, so the ten below would land failing
   it too.
3. **Segment-tree descent hooks (15).**
4. **Finish `poly.cp` (10):** composition, power projection, negative powers,
   middle product, subproduct tree, Taylor shift.
5. **Ordered map (10).**
6. **128-bit integers (4, plus 2 of the geometry group).** A compiler bug, not
   a missing feature: `I128` is accepted and compiled as 64-bit.
7. **Interactive: `flush` and an interactor in the gate (5).**
8. **Float math surface (5) and rational or `long double` geometry (4).**
9. **Top tree (3), unsigned wraparound (3), then the single-submission items:**
   `ModR`, min-cost flow in `stdlib/`, `pairnum`, LARSCH.

## The long tail in the rest of the tree

A pattern search over all 6,990 submissions, in `main` and below it. These are
upper bounds (a search for `endl` is not a search for interactive problems),
given to size the work, not to rank it: floating point 707, ordered set or map
420, geometry on points 361, suffix structures and string matching 151,
flows and matching 115, treap / splay / link-cut / top tree 106, NTT / FFT /
power series 99, `goto` 95, unsigned 64-bit 76, hash map 67, 128-bit 46,
`std::bitset` 45, run-time modulus 35, `long double` 30, matrices and
Gaussian elimination 19. Interactive could not be counted this way.

Floating point and geometry are small in the recent 202 and large in the
tree, so they will move up as the scan goes further back.

## Not done here

- No submission older than 2026-02-23 was read; the long-tail numbers are
  pattern counts only.
- Performance was measured for the five new ports only, once each. The 34
  earlier ports were not re-measured, and emitted size was not re-checked for
  them.
- The nine power-series submissions other than CF 2255 F were not read line by
  line, so which missing operation each needs is not known yet.
