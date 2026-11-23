# ecnerwala Corpus Wave #3

Ported **20 additional recent submissions** from [`ecnerwala/contest-programming`](https://github.com/ecnerwala/contest-programming) in decreasing recency order (`CF/2245/F`, `AtCoder/WTF26/A..D`, `CF/2237/A..F,I1`, `ACM/UCup4/Stage25-Zagreb/B,C,H`, `CF/2229/A..D`), expanding `proto/ports/ecnerwala/` from **39 -> 59 passing ports**.

## Summary Metrics

- **Base commit**: `606c9b9` (`origin/main` with 0.2.2 syntax batches, slice bounds traps, and updated `poly.cp`)
- **New ports passing `gate.sh NAME 40`**: **20 / 20** (`cpl check`, `cpl build` < 64 KB, ASan/UBSan C++17, `cpl run` interpreter, and 40-case C++ differential match)
- **Total `proto/ports/ecnerwala/` ports**: **59** (up from 39)
- **Wave 3 max emitted C++17**: `wtf26c` at **50,341 B (49.2 KB)** (< 64 KB judge limit)
- **Friction lint (`python3 scripts/friction_lint.py` on all 20 new ports)**: **0 `wa_trap`, 0 `real_gap`, 0 `stale_port`**
- **Total Wave 3 LOC**: **627 LOC** CP Lang vs **3,620 LOC** C++ (**5.8x reduction**)
- **Gate harness fix**: added a 1-line C++23 `std::ranges::iota` shim to the `zig c++` `<bits/stdc++.h>` fallback in `proto/ports/ecnerwala/gate.sh` so `wtf26a_orig.cpp` compiles cleanly.

## Ported Submissions (Decreasing Recency Order)

| Port | Upstream Source | CP LOC | Orig LOC | Emitted C++17 | Gate (`gate.sh NAME 40`) |
|---|---|---:|---:|---:|---|
| `cf2245f` | `CF/2245/F/F.cpp` | 40 | 316 | 15,367 B (15.0 KB) | PASS (1 sample, 40 random) |
| `wtf26a` | `AtCoder/WTF26/A/A.cpp` | 59 | 166 | 13,373 B (13.1 KB) | PASS (1 sample, 40 random) |
| `wtf26b` | `AtCoder/WTF26/B/B.cpp` | 26 | 399 | 8,324 B (8.1 KB) | PASS (6 samples, 40 random) |
| `wtf26c` | `AtCoder/WTF26/C/C.cpp` | 49 | 1,487 | 50,341 B (49.2 KB) | PASS (4 samples, 40 random)* |
| `wtf26d` | `AtCoder/WTF26/D/D.cpp` | 109 | 208 | 16,249 B (15.9 KB) | PASS (1 sample, 40 random) |
| `cf2237a` | `CF/2237/A/A.cpp` | 5 | 16 | 2,872 B (2.8 KB) | PASS (1 sample, 40 random) |
| `cf2237b` | `CF/2237/B/B.cpp` | 16 | 30 | 6,393 B (6.2 KB) | PASS (1 sample, 40 random) |
| `cf2237c` | `CF/2237/C/C.cpp` | 8 | 263 | 2,939 B (2.9 KB) | PASS (1 sample, 40 random) |
| `cf2237d` | `CF/2237/D/D.cpp` | 15 | 27 | 4,042 B (3.9 KB) | PASS (1 sample, 40 random) |
| `cf2237e` | `CF/2237/E/E.cpp` | 50 | 62 | 9,908 B (9.7 KB) | PASS (1 sample, 40 random) |
| `cf2237f` | `CF/2237/F/F.cpp` | 22 | 35 | 6,321 B (6.2 KB) | PASS (1 sample, 40 random) |
| `cf2237i1` | `CF/2237/I1/I1.cpp` | 63 | 184 | 14,402 B (14.1 KB) | PASS (1 sample, 40 random) |
| `ucup4s25b` | `ACM/UCup4/Stage25-Zagreb/B/B.cpp` | 38 | 155 | 11,834 B (11.6 KB) | PASS (0 samples, 40 random) |
| `ucup4s25c` | `ACM/UCup4/Stage25-Zagreb/C/C.cpp` | 50 | 70 | 11,010 B (10.8 KB) | PASS (0 samples, 40 random) |
| `ucup4s25h` | `ACM/UCup4/Stage25-Zagreb/H/H.cpp` | 15 | 25 | 5,695 B (5.6 KB) | PASS (0 samples, 40 random) |
| `cf2229a` | `CF/2229/A/A.cpp` | 5 | 13 | 5,192 B (5.1 KB) | PASS (1 sample, 40 random) |
| `cf2229b` | `CF/2229/B/B.cpp` | 7 | 15 | 3,710 B (3.6 KB) | PASS (1 sample, 40 random) |
| `cf2229c1` | `CF/2229/C1/C1.cpp` | 11 | 24 | 3,382 B (3.3 KB) | PASS (1 sample, 40 random) |
| `cf2229c2` | `CF/2229/C2/C2.cpp` | 23 | 41 | 4,414 B (4.3 KB) | PASS (1 sample, 40 random) |
| `cf2229d` | `CF/2229/D/D.cpp` | 21 | 50 | 5,965 B (5.8 KB) | PASS (1 sample, 40 random) |

\*`wtf26c` upstream `sample5` (`N = 80`) matched `wtf26c_orig.cpp` byte-for-byte under ASan/UBSan C++17 (`diff` exit 0), but exceeded `gate.sh`'s 20 s `cpl run` interpreter timeout; `sample1..4` plus 40 random cases pass both ASan/UBSan and `cpl run`.

## Unported Recent Submissions Encountered in Decreasing Recency Order

- **`CF/2255/F`** — **NEW P0 BLOCKER (emitted C++17 > 64 KB)**; see Blocker #1 below. The port runs and matches `CF/2255/F/sample1.in` (`120 / 12 / 265885269`) verbatim in `cpl run`, but `cpl build` rejects it at **76,874 B > 65,536 B**.
- **`CF/2245/G`, `CF/2245/H`** — deferred (large 325-line tensor DP / divide-and-conquer segment tree).
- **`AtCoder/WTF26/E`** — known roadmap blocker (`std::bitset` + 541-line DP).
- **`AtCoder/ARC223/A..E`** — empty 8-line template files upstream (not submitted).
- **`AtCoder/ARC223/F`** — known roadmap blocker (`__int128` + flow).
- **`CF/2237/G`** — interactive (`std::flush`, needs interactor harness).
- **`CF/2237/H`** — known roadmap blocker (692-line CRTP intrusive top tree).
- **`CF/2237/I2`**, **`ACM/UCup4/Stage25-Zagreb/E,I`** — deferred to Wave #4 (already reached 20 new ports).

## Compiler & Standard-Library Blockers and Minimal Repros

### Blocker 1 (P0, Hard Blocker for `CF/2255/F`): `lib.math.poly` NTT + FPS Emits > 64 KB C++17

Commit `606c9b9` expanded `examples/local-lib/math/poly.cp` with specialized radix-4/radix-7 autocorrelation NTT kernels (`m1_conv_z_r7_autocorr`, `m1_dif_stages`, `m1_dit_stages`, `m1_autocorr4_range`, `m1_autocorr4_blk`, etc.). Even a port calling only `convolve` (`wtf26c.cp`) now emits **50,341 B**, pre-existing `ucup5s05f.cp` is pushed to **64,690 B** (stripping its source header to fit under 65,536 B), and any port combining `convolve` + `compose_trunc` + `pow_trunc` + `prod_linear` (`cf2255f.cp`) blows past the 64 KB judge limit at **76,874 B**.

Minimal repro (requires `proto/ports/ecnerwala/cpl.toml` mapping `math` to `examples/local-lib/math`):

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

Observed behavior:
- `cpl check` passes (`0 errors`).
- `cpl run` matches `CF/2255/F/F.cpp` verbatim on `sample1.in` (`120 / 12 / 265885269`).
- `cpl build` fails: `cpl: the C++ for ... is 76874 bytes, over the 65536-byte limit judges accept`.

Suggested fix: either compact `poly.cp`'s NTT/FPS codegen footprint (e.g., trim specialized radix-7/autocorr code duplication or share butterfly helpers) and/or add dead-function/minification compaction in `cpl-codegen` when approaching 64 KB.

### Blocker 2 (Compiler Bug, Hit in `wtf26a`): `Dsu<0..n>()` Crashes Checker with Bogus `V` Error at Line 1:1

Passing an anonymous range domain `0..n` to `Dsu<...>` instead of a named `domain V = 0..n` causes an internal checker failure referring to `Dsu`'s own type parameter `V` at span `1:1`.

Minimal repro:

```cp
let n: Int = read()
var dsu = Dsu<0..n>()
print(dsu.find(0))
```

Observed error:
```text
error[E0201]: `cpl run` doesn't know the name `V` here
  --> repro.cp:1:1
```

Workaround used in `wtf26a.cp`: declare `domain V = 0..total_nodes` and construct `Dsu<V>()`.

### Blocker 3 (Borrow Prover Gap, Hit in `cf2237b`): `swap(mut a[k], mut a[k - 1])` Rejected as Potentially Aliasing

The disjointness prover does not automatically discharge `k != k - 1` (or more generally `x != x + c` for nonzero constant `c`) for `Array.at` mutable borrows.

Minimal repro:

```cp
let n: Int = read()
var a: [0..n]Int = read()
for k in (1..n).rev:
    swap(mut a[k], mut a[k - 1])
```

Observed error:
```text
error[E0601]: `a[k]` and `a[(k - 1)]` may be the same storage, and both are lent `mut` to this call: Array.at lends disjoint parts only when `i != j`, and that is not known here
```

### Blocker 4 (Type-Inference / Expected-Type Propagation Gaps, Hit in `cf2255f`, `wtf26c`, `cf2237i1`)

Three small expected-type propagation friction points surfaced during Wave 3:

1. **Unresolved modulus in comprehension over `Comb<MOD>` methods (`cf2255f`)**:
   ```cp
   use lib.math.poly.{Z, pow_trunc}
   const MOD = 998244353
   let comb = Comb<MOD>()
   let catalan = [comb.catalan(i) for i in 0..5]
   let p = pow_trunc(catalan, -3, 5)
   // error[E0302]: expected `List<Mod<998244353>>`, found `List<Mod<_>>`
   ```
2. **`List.filled(n, 0)` ignores outer `List<Z>` type annotation (`wtf26c`)**:
   ```cp
   type Z = Mod<998244353>
   var a: List<Z> = List.filled(4, 0)
   // error[E0302]: expected `List<Mod<998244353>>`, found `List<Int>`
   ```
3. **Nested fixed-array literal inside list literal (`cf2237i1`)**:
   ```cp
   type Z = Mod<1_000_000_007>
   var dp: List<[0..4]Z> = [[1, 0, 0, 0]]
   // error[E0302]: `dp` is declared a List<[0..4]Mod<_>>, but `[[1, 0, 0, 0]]` is a List<List<Int>>
   ```
