# Friction Hunter — Wave 10 (ports 271–300)

Base: `main` at `a55d616` (includes `98137b4`, "Seven wrong-answer traps stop both
back ends"). This is a read-only audit of the next 30 unaudited compiling ports.
Every candidate gap was reproduced on HEAD before it was counted.

**Pool note:** the long ports are used up. Only 16 unaudited compiling ports
reached the usual thresholds (ecnerwala with a generator and at least 15 lines,
or other ports with at least 30), so the last 14 are the longest remaining ports
with at least 10 lines. These 30 ports average 22 lines (wave 9: 46). That is
most of why this wave's per-problem numbers drop. 52 compiling unaudited ports
with at least 10 lines are left.

Labels as in earlier waves: **[WA-TRAP]**, **[REAL GAP]**, **[STALE PORT]**,
**[REJECTED]**.

## Ports (30)

**ecnerwala Wave 4 (14):** `cf2222b`, `cf2211d`, `cf2229c2`, `cf2237f`,
`cf2229d`, `cf2207b`, `cf2222c`, `cf2211b`, `cf2207d`, `cf2211c1`,
`cf2237b`, `ucup4s25h`, `cf2237d`, `cf2219a`.

**scout and canonical (16):** `btoi04p3`, `wide`, `dpy`, `joi16op3`, `bits`,
`btoi08p6`, `coci21c1p4`, `dmopc21c6p3`, `cco20p1`, `dpt`, `ioi07p4`,
`ioi11p2`, `ioi11p4`, `sac22cc4jp4`, `coci21c2p3`, `dmopc21c7p2`.

## Averages

- **Real gaps:** 6 across the 30 ports, so `real_gaps / 30` = **0.20**
  (wave 9: 0.40). 5 come from the ports and 1 from a probe.
- **Awkward lines:** 79 across the 30 ports, so `total_awkward_lines / 30` =
  **2.63** (wave 9: 6.77). About 90% of them already have a clean form on HEAD.
- **WA traps:** **1 new**: release builds skip the range checks on `**` and
  shifts (below).
- **Cumulative, 300 ports:** 165 real gaps and 1994 awkward lines, so
  **0.55** real gaps and **6.65** awkward lines per problem.

Per-port awkward lines / real gaps: cf2222b 4/0, cf2211d 1/0, cf2229c2 2/0,
cf2237f 4/0, cf2229d 1/0, cf2207b 3/0, cf2222c 3/0, cf2211b 0/0,
cf2207d 4/0, cf2211c1 3/0, cf2237b 3/1, ucup4s25h 2/0, cf2237d 1/1,
cf2219a 3/0, btoi04p3 2/0, wide 8/0, dpy 6/0, joi16op3 1/0, bits 8/1,
btoi08p6 4/0, coci21c1p4 1/1, dmopc21c6p3 3/0, cco20p1 1/0, dpt 2/1,
ioi07p4 3/0, ioi11p2 1/0, ioi11p4 0/0, sac22cc4jp4 2/0, coci21c2p3 3/0,
dmopc21c7p2 0/0.

## Differential runs: `cpl run` vs sanitized C++17

17 ports were run on 15 inputs each, through `cpl run` and the
`cpl build --checked` binary (ASan, UBSan, `_GLIBCXX_ASSERTIONS`). The 14
ecnerwala ports used `gen/NAME.py SEED SEED%8+1`; `dmopc21c6p3`,
`dmopc21c7p2` and `ioi11p4` used their scout `check.py` generators. That is
**255 runs with 0 divergences** and no sanitizer reports. The other 13 ports
have no generator script and were read only.

## Targeted WA-trap probes (10 areas, 23 runs)

Each probe was run through `cpl run` and `cpl run --release`.

- **Shifts:** `1 << 64`, `1 << 70` and `1 << -1` → **WA trap** (below).
  `1 << 63` traps in `cpl run` and gives `MIN` in release. That is
  overflow, so it is local-only by design.
- **`**`:** `2 ** -1`, `-2 ** -1` and `1 ** -5` → **WA trap** (below).
  `0 ** 0` = 1 and `-2 ** 3` = -8 in both. `3 ** 40` is overflow, so
  local-only by design.
- **`-MIN`, `abs(MIN)`, `MIN / -1`:** both backends trap. After `98137b4`,
  release does trap here.
- **`Bitset` with n = 0, 64 and 65:** element counts and `shifted(1)` are
  correct in both (`0 0 0`, `22 21 21`, `22 22 22`), including the word
  boundary. `Bitset` has no `count`/`len`, so counting needs
  `count(true for _ in b.each())` (real gap).
- **`F64` formatting:** both print the same: NaN as `-nan`, `inf`,
  `-0.000000`, and `1e-7` as `0.000000`. The fixed six decimals lose small
  values silently. That's a watch item, since earlier waves found the format
  documented.
- **`LazySegTree` with n = 1:** `AddMin` apply then fold gives 8 in both.
- **`Dsu` self-merge:** `unite(v, v)` returns `NO` and the size stays 1, in
  both.
- **`FlowGraph` with zero-capacity and parallel edges:** max flow is 7 in both
  (correct).

## [WA-TRAP] 1. Release builds skip the range checks on `**` and shifts

```cp
let (a, b): (Int, Int) = read()
print(a ** b)
```

On input `2 -1`, `cpl run` stops with "`**` needs a power that is not
negative, and this is -1". `cpl run --release` prints `1` and exits 0.

```cp
let s: Int = read()
print(1 << s, 5 >> s)
```

`cpl run` traps with "a shift by 64 is outside 0..64". The release build
prints the following, all with exit 0:

- `1 5` for s = 64.
- `64 0` for s = 70.
- `-9223372036854775808 0` for s = -1.

The C++ shift by 64 or more is undefined behaviour, and on x86 it masks the
count.

**Why it isn't covered by the local-only overflow rule:** that rule is about
values that don't fit in 64 bits. A negative exponent and a shift count outside
0..64 are invalid operations, like the reversed ranges and empty domains that
`98137b4` now traps in both backends. Release already traps `-MIN` with exit 3,
so these two checks are just missing from it.

**Where it bites:** `1 << k` with a computed `k` (bitmask DP with `k = n`, or
`1 << (60 - i)`) gives the wrong mask silently on the judge.

**Fix:** emit the same range check in release, as `if (s >> 6) trap` before the
shift and `if (b < 0) trap` in `pow`. Both are a single, well-predicted
branch. Add shift counts in `{-1, 63, 64, 70}` and negative exponents to the
three-way fuzzer.

## [REAL GAP] findings (each reproduced on HEAD)

### New

1. **No wrapping unsigned 64-bit integer** (`bits`, a local-library file).
   The port builds `U64(hi, lo)` from two 32-bit halves, with `parse`, `shl`,
   `shr` and `plus` written by hand (about 8 lines). xorshift generators,
   splitmix hashing and "answer mod 2^64" problems all need this. `I128`
   exists, but it doesn't wrap.
2. **`Bitset<D>` has no `count`** (from the probe): only `each` and `next`, so
   a popcount is a generator count.

### Recurring (counted again)

- **`Char.digit`:** `cf2237d` (`s[i] - '0'`) and `coci21c1p4` (`ch - '1'`).
- **`List.insert`/`remove`** (wave 7): `cf2237b` moves `a[j]` to position `i`
  with a hand-written shift loop.
- **Sub-domain label crossing:** `dpt` uses `Rank(g.label)` to go from
  `Gap = 1..n` to `Rank = 0..n`.

## [STALE PORT] patterns (clean form verified on HEAD)

- **Hand 128-bit `Wide` struct** (`wide`, 8 lines): `I128` exists, and
  `coci21c1p4` uses it in this same wave.
- **Hand `fact`/`inv_fact` tables over a domain** (`dpy`, 6 lines): `Comb<P>`.
- **Adjacency and recursive DFS** (`cf2207d`): `List.filled(n, [])` with
  `u - 1`, then `some(nxt) == prv`. The clean form is `Graph.undirected` plus
  `Rooted`.
- **Sentinels:** `-(n + 1)` and `-(2n + 2)` (`cf2222c`, `cf2237f`) are `?Int`
  with `max=`; `?? -1` followed by `if j == -1` (`cf2237b`) is `else: return -1`;
  `0` meaning "no food" (`ioi07p4`) is `?Food`.
- **Hand argmin, argmax and best-pair loops** (`cf2207b`, `btoi08p6`,
  `cf2219a`): `argmin`, `min=` by key, and `first(..)`.
- **Resizing by `push`/`pop` loops** (`cf2207b`): `resized(nm, 0)`.
- **Split and count loops** (`cf2222b`): comprehension filters and `count(..)`.
- **Small fixes:** `left.reverse()` followed by a `push` loop
  (`ucup4s25h`) is `left.rev + right`; `0 <= st and st <= hi` (`cf2237f`) is
  `st in 0..=hi`; `DIRS` with a hand-written `step` (`dmopc21c6p3`) is
  `Dir4` and `step`.
- **`Int` masks next to `Subset<E>`** (`coci21c2p3`): `1 << e.idx` and
  `popcount(flip.idx)` should use `Subset` operations.

## [REJECTED]

Slope-trick heap merging (`btoi04p3`), small-to-large map merging (`ioi11p2`),
cycle decomposition (`sac22cc4jp4`), Eisenstein-integer FWHT (`coci21c1p4`),
and linked-list splicing (`dmopc21c7p2`) are algorithms, not language gaps.

## Proposed Syntax Batch #8 additions

1. **[WA-TRAP] Release range checks for `**` and shifts.** The exponent must not
   be negative and the shift count must be in 0..64, in both backends.
2. **`U64` (wrapping)** or `wrapping_mul`/`wrapping_add` on `Int`, for hashing
   and RNG.
3. **`Bitset.count`.**
4. **Still open:** `Char.digit`, `List.insert`/`remove`, and sub-domain
   crossings.
