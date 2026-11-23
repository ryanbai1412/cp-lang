# ecnerwala Corpus Wave #4 (`59 -> 79` Ports)

Wave #4 expands `proto/ports/ecnerwala/` by **20 recent submissions** in decreasing recency order (`59 -> 79` total), rebased onto `origin/main` (including `ef6036e` trimmed `poly.cp`, `bcd873b`, and `b59851a`).

## Summary

- **New ports**: **20** (`cf2229e`, `cf2229f`, `cf2229g`, `cf2229i`, `ucup4s24f`, `cf2222a`, `cf2222b`, `cf2222c`, `cf2222d`, `cf2222g`, `cf2219a`, `cf2211a`, `cf2211b`, `cf2211c1`, `cf2211c2`, `cf2211d`, `cf2207a`, `cf2207b`, `cf2207c`, `cf2207d`).
- **Gate verification**: **20/20 pass** `proto/ports/ecnerwala/gate.sh NAME 40` (`cpl check`, `cpl build` < 64 KB, ASan/UBSan C++17 + `cpl run` interpreter + original C++ differential match across all upstream samples plus 40 seeded random cases each).
- **Friction lint (`python3 scripts/friction_lint.py` on all 20 new ports)**: **0 `wa_trap`**, **0 `real_gap`**, **0 `stale_port`** (0.00 workarounds/file).
- **Emitted C++17 size**: max is **18,081 B (17.7 KB)** (`ucup4s24f`, well under the 65,536 B judge cap).
- **Conciseness**: **521 CP Lang LOC vs 1,137 C++ LOC** (**54.2% raw LOC reduction**).

## Ported Submissions (Decreasing Recency Order)

1. `cf2229e` (`CF/2229/E`) — 31 CP LOC vs 125 C++ LOC, 9,617 B emitted, 1 sample + 40 random PASS.
2. `cf2229f` (`CF/2229/F`) — 30 CP LOC vs 55 C++ LOC, 10,786 B emitted, 2 samples + 40 random PASS.
3. `cf2229g` (`CF/2229/G`) — 45 CP LOC vs 62 C++ LOC, 8,762 B emitted, 1 sample + 40 random PASS.
4. `cf2229i` (`CF/2229/I`) — 66 CP LOC vs 86 C++ LOC, 11,336 B emitted, 1 sample + 40 random PASS.
5. `ucup4s24f` (`ACM/UCup4/Stage24-Wulin/F`) — 31 CP LOC vs 39 C++ LOC, 18,081 B emitted, 40 random PASS (upstream has no sample `.in`; `R >= 2` required because `ucup4s24f_orig.cpp` reads past `stk.end()` on `R == 1`, whereas CP Lang's `dq.first!` trapped cleanly on seed 1).
6. `cf2222a` (`CF/2222/A`) — 4 CP LOC vs 15 C++ LOC, 2,842 B emitted, 1 sample + 40 random PASS.
7. `cf2222b` (`CF/2222/B`) — 24 CP LOC vs 36 C++ LOC, 5,787 B emitted, 1 sample + 40 random PASS.
8. `cf2222c` (`CF/2222/C`) — 19 CP LOC vs 38 C++ LOC, 5,270 B emitted, 1 sample + 40 random PASS.
9. `cf2222d` (`CF/2222/D`) — 12 CP LOC vs 21 C++ LOC, 4,315 B emitted, 1 sample + 40 random PASS.
10. `cf2222g` (`CF/2222/G`) — 69 CP LOC vs 97 C++ LOC, 12,330 B emitted, 1 sample + 40 random PASS.
11. `cf2219a` (`CF/2219/A`) — 15 CP LOC vs 23 C++ LOC, 4,658 B emitted, 1 sample + 40 random PASS.
12. `cf2211a` (`CF/2211/A`) — 4 CP LOC vs 13 C++ LOC, 2,905 B emitted, 1 sample + 40 random PASS.
13. `cf2211b` (`CF/2211/B`) — 19 CP LOC vs 134 C++ LOC, 6,290 B emitted, 1 sample + 40 random PASS.
14. `cf2211c1` (`CF/2211/C1`) — 17 CP LOC vs 28 C++ LOC, 4,319 B emitted, 1 sample + 40 random PASS.
15. `cf2211c2` (`CF/2211/C2`) — 29 CP LOC vs 44 C++ LOC, 6,579 B emitted, 1 sample + 40 random PASS.
16. `cf2211d` (`CF/2211/D`) — 24 CP LOC vs 149 C++ LOC, 12,019 B emitted, 1 sample + 40 random PASS.
17. `cf2207a` (`CF/2207/A`) — 11 CP LOC vs 23 C++ LOC, 3,908 B emitted, 1 sample + 40 random PASS.
18. `cf2207b` (`CF/2207/B`) — 20 CP LOC vs 27 C++ LOC, 5,431 B emitted, 1 sample + 40 random PASS.
19. `cf2207c` (`CF/2207/C`) — 33 CP LOC vs 87 C++ LOC, 8,947 B emitted, 1 sample + 40 random PASS.
20. `cf2207d` (`CF/2207/D`) — 18 CP LOC vs 35 C++ LOC, 5,604 B emitted, 1 sample + 40 random PASS.

## Genuine Blockers & Minimal Repros

### 1. `CF/2255/F`: `lib.math.poly` FPS composition still exceeds 64 KB after `ef6036e` (`70,416 B > 65,536 B`)

Re-tested on `origin/main` after `ef6036e` trimmed `poly.cp`: emitted size dropped from **76,874 B** (Wave #3) to **70,416 B**, which is still **4,880 B over** the 65,536 B judge limit whenever a solution imports `compose_trunc`, `convolve`, `pow_trunc`, and `prod_linear` together.

Minimal repro:

```cp
use lib.math.poly.{Z, compose_trunc, convolve, pow_trunc, prod_linear}
const MOD = 998244353
let comb = Comb<MOD>()
@!multitest
let n: Int = read()
let a: [0..n]Z = read()
let vieta = prod_linear([-a[i] for i in 0..n])
let m = n / 2 + 1
let v2 = [vieta[k] * vieta[n - k] for k in 0..m]
let catalan: List<Z> = [comb.catalan(i) for i in 0..m]
var cat1: List<Z> = catalan
cat1[0] = 0
let ways = convolve(compose_trunc(v2, cat1, m), pow_trunc(catalan, -n, m))
var ans: Z = 0
for k in 0..m:
    let fk = comb.fact(k)
    ans += ways[k] * comb.fact(n) * comb.inv_fact(2 * k) * fk * fk * 2
print(ans / n)
```

Command: `cpl build _test_cf2255f.cp -o /tmp/test_cf2255f.cpp`
Observed error: `cpl: the C++ for ... is 70416 bytes, over the 65536-byte limit judges accept`.

Why it matters: `ef6036e` recovered ~6.5 KB, leaving only ~4.9 KB to shave via dead-function/runtime pruning or tighter FPS codegen in `crates/cpl-codegen` before `CF/2255/F` can enter the corpus cleanly.

### 2. Missing ordered set / ordered map (`BTreeSet<T>` / `BTreeMap<K, V>`)

Several recent ecnerwala submissions rely on ordered `std::set` / `std::map` queries (`lower_bound`, `prev`, `next`, `rbegin` with deletion). For `ucup4s24f`, only `rbegin()` was needed, so replacing `std::set` with `MaxHeap<(Int, Int)>` + lazy deletion against `Map<Int, Int>` was clean and idiomatic. However, `CF/2222/E` (`cf2222e`) and `CF/2219/C` (`cf2219c`) require general ordered-set operations (`insert`, `erase`, predecessor/successor) that cannot be replaced with a heap without re-implementing a balanced tree.

### 3. Interactive I/O & gate harness (`CF/2222/E`)

`CF/2222/E` is an interactive problem using `std::flush`. `proto/ports/ecnerwala/gate.sh` currently runs batch stdin/stdout differential comparisons only and has no interactor mode.

## Unfinished Upstream Drafts & Deferred Heavy Submissions (Between `CF/2255/F` and `CF/2207/D`)

- **Unfinished/empty upstream drafts (no output printed, skipped)**: `CF/2229/H`, `ACM/UCup4/Stage24-Wulin/A`, `CF/2222/H`, `CF/2219/B1`, `CF/2219/B2`, `CF/2219/D`, `CF/2219/E`.
- **Blocked or deferred to Wave #5**:
  - `CF/2255/F` — blocked by Blocker #1 above (`70,416 B > 65,536 B`).
  - `ACM/UCup4/Stage24-Wulin/C` — 1,407-line `__int128` + `double` + FFT + flow submission (blocked by Blocker #1 + float/flow).
  - `CF/2222/E` — interactive (`std::flush`) + ordered `std::set` (Blockers #2 and #3).
  - `CF/2219/C` — floating-point (`double`) geometry/optimization.
  - `CF/2237/I2`, `ACM/UCup4/Stage25-Zagreb/E`, `ACM/UCup4/Stage25-Zagreb/I`, `ACM/UCup4/Stage24-Wulin/M`, `CF/2222/F` — larger multi-hundred-line submissions queued for Wave #5.
