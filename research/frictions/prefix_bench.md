# prefix_bench — Prefix / Diff / FillPrefix matched C++17 benchmarks

Measured via `python3 bench/prefix/run.py --runs 5` (`g++ -std=c++17 -O2`, matched
layout: `int64_t` for `Int`, `uint32_t` for `Mod<998244353>`). Overall
**p50 = 1.10x**, **max = 1.35x**. Four rows remain above `1.10x` for
codegen-only reasons.

## Per-query `cpl::pos` bounds checks and `r.lo >= r.hi` guard on tight 1D prefix/diff loops
- **Area:** codegen
- **Goal:** performance
- **Severity:** low
- **Status:** open
- **Where:** `bench/prefix/prefix_1d.cp` (`1.25x`, 0.048s vs 0.038s) and
  `bench/prefix/diff_1d.cp` (`1.15x`, 0.056s vs 0.049s) on `n=524k, q=10M`.
- **Detail:** Each query constructs `D(l)..=D(r)` with `l = min(x, y)` and
  `r = max(x, y)` where `x, y` are masked by `n - 1`. Codegen emits two
  `cpl::pos(..., D_len)` bounds checks and `Prefix.sum` / `Diff.add` checks
  `r.lo >= r.hi` inside a 3.8–4.9 ns/query loop of two array accesses. Range
  analysis does not yet prove `x & (n - 1) in 0..n` or `min(x, y) <= max(x, y)`
  across `cpl::pos`.

## Per-cell `FillPrefix` order/filled assertions and `cpl::pos` checks on tight 2D `Mod` loops
- **Area:** codegen
- **Goal:** performance
- **Severity:** low
- **Status:** open
- **Where:** `bench/prefix/fill_2d_fwd.cp` (`1.33x`, 0.045s vs 0.034s) and
  `bench/prefix/fill_2d_rev.cp` (`1.35x`, 0.048s vs 0.035s) on `2048x2048`.
- **Detail:** `FillPrefix<R, C, Z>.set` and `.sum` check `in_order` and
  `in_filled` via `r * C_len + c` arithmetic plus four `cpl::pos` bounds checks
  per cell on a 3.3 ns/cell `uint32_t` loop. Removing the assertions and
  `cpl::pos` checks drops `fill_2d_rev` to `1.09x`, confirming the gap is
  checked-invariant overhead rather than layout or `Grid` indexing.
