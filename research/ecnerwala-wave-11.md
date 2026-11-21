# `ecnerwala` Corpus Wave #11 (`200 -> 225` Verified Ports)

Wave #11 adds **25 newly verified recent submissions** from `ecnerwala/contest-programming` in decreasing recency order right after Wave #10 (`CF/2124/F2`, `CF/2115/F1`, `CF/2101/C`–`F`, `CF/2096/A`–`H`, `CF/2089/A`–`E`, `CF/2084/A`–`D`), bringing the verified `ecnerwala` corpus from **200 to 225 ports**.

No working submission was skipped. In decreasing recency order, `CF/2127/G1` in the upstream repo is an unfinished WIP whose built-in `if (true)` self-test aborts (`SIGABRT` on `assert(P == P_mine)` because `solve()` returns `P` instead of `P_shuf`), and `CF/2127/G2`, `CF/2120/A`–`H`, and `CF/2115/F2` are empty 8-line templates.

## Summary Metrics

- **Ports added**: 25 (`200 -> 225` total in `proto/ports/ecnerwala/`).
- **LOC**: **1,306 CP Lang LOC** vs **3,996 raw C++ LOC** (**-67.3%** raw reduction) and **1,854 library-stripped C++ LOC** (**-29.6%** stripped reduction; stripping `modnum`, `tensor`, `seg_tree`, and `ecnerwala::fft` boilerplate from the originals).
- **Emitted C++17 size**: all 25 strictly under the 64 KB (`65,536 B`) judge cap. Maximum is **42.5 KB** (`43,497 B` on `cf2089e` using `lib.math.poly.{Z, convolve}` for NTT polynomial tree DP, down from 930 raw C++ LOC to 118 CP LOC); next largest are `cf2115f1` (**19.3 KB**, `19,763 B`, implicit treap + mergeable segment tree) and `cf2096f` (**19.0 KB**, `19,496 B`, two-pointer dual segment trees + per-index multiset `Map`).
- **Verification**:
  - **25 / 25** pass `proto/ports/ecnerwala/gate.sh NAME 40` (samples + 40 seeded random cases; `cpl check`, `cpl build`, ASan/UBSan C++17, `cpl run` interpreter, and differential output match against the accepted original).
  - **25 / 25** pass `cpl fmt --check` (`0 would change`).
  - **25 / 25** pass `cpl lint` (**0 warnings, 0 errors** — Audit #15 gate).
  - **25 / 25** pass `scripts/friction_lint.py` (**0 `wa_trap`, 0 `real_gap`, 0 `stale_port`**).
  - `cf2124f2` was additionally verified on ecnerwala's `N = 5000` stress file (`69 420`) under ASan/UBSan with exact output match against the original (`samples/cf2124f2.1.in` keeps the 7-case official sample for interpreter speed).

## Wave #8–#9 (`685655e`) Features Validated

Every newly landed ergonomics feature exercised in Wave #11 worked cleanly:

- Guarded `while let x = opt, cond:` worked directly in monotonic stacks and lazy pop loops (`cf2101d`, `cf2124f2`, `cf2089b1`, `cf2089b2`).
- Nested counted-array fill `[0..n - 1][0..3]Bool = false`, `[0..n][0..n + 2]Mod<MOD> = 0`, and `[0..=k_max][0..n]Mod<MOD> = 0` worked cleanly in `cf2096c`, `cf2124f2`, and `cf2089c2`.
- Live-pipe interactive `cpl run` worked on `cf2096g` (balanced ternary weighing) and existing interactive ports (`cf2178e`), allowing `proto/ports/ecnerwala/gate.sh` to replace transcript replay with live interactor-driven `cpl run` execution.

## Ports Added (25)

- `cf2124f2` (`CF/2124/F2`): 59 CP LOC vs 349 raw / 78 stripped C++ LOC, `12,833 B` (`12.5 KB`) — 2D lazy diff-array DP mod `998244353`.
- `cf2115f1` (`CF/2115/F1`): 173 CP LOC vs 503 raw / 229 stripped C++ LOC, `19,763 B` (`19.3 KB`) — index-based implicit treap (`std.rand.Rng`) + mergeable persistent segment tree.
- `cf2101c` (`CF/2101/C`): 43 CP LOC vs 55 raw / 55 stripped C++ LOC, `8,828 B` (`8.6 KB`) — DSU greedy from both ends.
- `cf2101d` (`CF/2101/D`): 45 CP LOC vs 53 raw / 53 stripped C++ LOC, `8,391 B` (`8.2 KB`) — guarded `while let` monotonic stacks.
- `cf2101e` (`CF/2101/E`): 112 CP LOC vs 247 raw / 247 stripped C++ LOC, `15,539 B` (`15.2 KB`) — iterative rounds of centroid decomposition on a tree.
- `cf2101f` (`CF/2101/F`): 64 CP LOC vs 201 raw / 80 stripped C++ LOC, `17,385 B` (`17.0 KB`) — subdivided-tree diameter DP mod `998244353`.
- `cf2096a` (`CF/2096/A`): 16 CP LOC vs 29 raw / 29 stripped C++ LOC, `4,540 B` (`4.4 KB`) — greedy permutation from `<>` string.
- `cf2096b` (`CF/2096/B`): 14 CP LOC vs 30 raw / 30 stripped C++ LOC, `4,332 B` (`4.2 KB`) — min/max pair greedy + top-`k` sum.
- `cf2096c` (`CF/2096/C`): 47 CP LOC vs 64 raw / 64 stripped C++ LOC, `7,579 B` (`7.4 KB`) — independent row/column 2-state DP with `[0..n - 1][0..3]Bool = false`.
- `cf2096d` (`CF/2096/D`): 13 CP LOC vs 23 raw / 23 stripped C++ LOC, `3,232 B` (`3.2 KB`) — shifted XOR invariant.
- `cf2096e` (`CF/2096/E`): 15 CP LOC vs 29 raw / 29 stripped C++ LOC, `4,550 B` (`4.4 KB`) — parity-adjusted inversion count.
- `cf2096f` (`CF/2096/F`): 126 CP LOC vs 423 raw / 133 stripped C++ LOC, `19,496 B` (`19.0 KB`) — two pointers over lazy cover segment tree + min-endpoint segment tree backed by per-index ordered `Map<Int, Int>`.
- `cf2096g` (`CF/2096/G`): 62 CP LOC vs 91 raw / 91 stripped C++ LOC, `8,894 B` (`8.7 KB`) — interactive balanced ternary weighing with live interactor (`interactors/cf2096g.py`).
- `cf2096h` (`CF/2096/H`): 73 CP LOC vs 218 raw / 82 stripped C++ LOC, `16,899 B` (`16.5 KB`) — FWHT over `Mod<998244353>` + `popcount`.
- `cf2089a` (`CF/2089/A`): 26 CP LOC vs 68 raw / 68 stripped C++ LOC, `5,343 B` (`5.2 KB`) — linear sieve + prime-centered permutation.
- `cf2089b1` (`CF/2089/B1`): 27 CP LOC vs 46 raw / 46 stripped C++ LOC, `6,176 B` (`6.0 KB`) — circular rotation + guarded `while let` stack.
- `cf2089b2` (`CF/2089/B2`): 37 CP LOC vs 54 raw / 54 stripped C++ LOC, `6,812 B` (`6.7 KB`) — circular stack + `k`-budget reduction.
- `cf2089c1` (`CF/2089/C1`): 24 CP LOC vs 157 raw / 33 stripped C++ LOC, `11,827 B` (`11.5 KB`) — cyclic prefix-sum probability DP mod `10^9 + 7`.
- `cf2089c2` (`CF/2089/C2`): 58 CP LOC vs 199 raw / 75 stripped C++ LOC, `16,199 B` (`15.8 KB`) — 2D cyclic prefix-sum DP mod `10^9 + 7`.
- `cf2089d` (`CF/2089/D`): 72 CP LOC vs 79 raw / 79 stripped C++ LOC, `8,495 B` (`8.3 KB`) — constructive ternary expression formatting (`Str(List<Char>)`).
- `cf2089e` (`CF/2089/E`): 118 CP LOC vs 930 raw / 128 stripped C++ LOC, `43,497 B` (`42.5 KB`) — NTT polynomial convolution tree DP mod `998244353` via `lib.math.poly.{Z, convolve}`.
- `cf2084a` (`CF/2084/A`): 12 CP LOC vs 34 raw / 34 stripped C++ LOC, `3,610 B` (`3.5 KB`) — modular divisibility permutation.
- `cf2084b` (`CF/2084/B`): 14 CP LOC vs 25 raw / 25 stripped C++ LOC, `5,926 B` (`5.8 KB`) — minimum-multiple GCD check.
- `cf2084c` (`CF/2084/C`): 43 CP LOC vs 62 raw / 62 stripped C++ LOC, `7,195 B` (`7.0 KB`) — involution pairing + cycle swaps via `p.swap(i, j)`.
- `cf2084d` (`CF/2084/D`): 13 CP LOC vs 27 raw / 27 stripped C++ LOC, `5,074 B` (`5.0 KB`) — stride assignment construction.

## Genuine Compiler, Interpreter, and Lint Findings

### 1. Codegen bug: nested `fn` with `mut` counted-array parameter emits `const&` in C++ lambda (`cf2101c`)

When a local `fn` inside a loop takes a `mut` counted array `[0..=n]Int`, `cpl check` accepts it, but `cpl build` emits a C++ lambda parameter without mutability so `g++` fails with `assignment of read-only location 'cpl::at<long long int>((* & par_2), ...)'`.

Minimal repro:

```cp
let n: Int = read()
for _ in 0..2:
    var par: [0..=n]Int = -1
    fn touch(mut par: [0..=n]Int, x: Int) -> Int:
        par[x] = 0
        x
    print(touch(mut par, 0))
```

Workaround used in `cf2101c.cp`: capture `par` directly in the nested `fn` instead of passing it as a `mut` parameter.

### 2. Interpreter bug (`R0001`): `a.reverse()` on a domain-indexed array `[I]Int` crashes `cpl run` (`cf2101c`)

Calling `.reverse()` on `var a: [I]Int` passes `cpl check`, `cpl build`, and ASan/UBSan C++17 execution, but aborts in `cpl run` with `error[R0001]: internal: this value can't be changed that way`.

Minimal repro:

```cp
let n: Int = read()
domain I = 0..n
var a: [I]Int = read()
a.reverse()
print(a)
```

Workaround used in `cf2101c.cp`: index `a[I(n - 1 - i.ord)]` on the second pass instead of mutating `a`.

### 3. Stale `cpl lint` (`L0414`) diagnostic suggests nonexistent `List<T>.indices` (`cf2101c`)

`warning[L0414]` advises: `loop over the values (for x in a) or the indices (for i in a.indices)`. However, `List<Int>` has neither `.indices` (`error[E0203]: a List<Int> has no member indices that cpl run knows`) nor `.enumerate()`. Either `List<T>.indices` should be implemented or the `L0414` help text should stop suggesting it on `List<T>`.

### 4. False-positive `cpl lint` (`L0203`) shadows a top-level `var` declared *later* in the file (`cf2089a`)

In `cf2089a`, a `for i in 2..=n:` loop on line 5 is warned with `L0203: this binding hides the top-level i` pointing to `var i = 2 * p - n - 1` declared on line 22 after the loop has already exited (and `for p in primes:` on line 9 against `var p = n / 2 + 1` on line 15). Because top-level bindings are only visible after their declaration line (`E0205`), a loop binding before that line cannot shadow it.
