# Friction report: bench/strings (remaining codegen/runtime gap on short benches)

Found while auditing `bench/strings` against matched C++17 (`bash bench/strings/run.sh`,
`g++ -std=c++17 -O2`, best of 5 runs). After adding `unchecked:` to `z_function` and
`prefix_function`, narrowing `Hash2D` tables to 32-bit (`0..=1_000_000_009`), and
borrowing immutable non-scalar captures in non-escaping closures, 8 of 10 rows are
0.58x–1.08x (**p50 = 0.87x**). The two remaining ~1.11x rows (`z_prefix` 26.6 vs 23.9 ms,
`manacher_lyndon` 16.0 vs 14.4 ms) emit zero bounds checks inside the library functions
themselves; the remaining ~2 ms is outside `std.strings`.

## Top-level global `List` loops keep `cpl::at` and `cpl::modp`

- **Where:** `bench/strings/z_prefix.cp` (`for i in 0..n:` over global `var s = List.filled(n, 0)`
  and globals `z`, `p` returned by `z_function`/`prefix_function`), plus `(x >> 16) % 3`
  after `x = (...) & 0x7fffffff` on a global `var x`.
- **What felt wrong:** `m1_z_function_Lst_Int_91` and `m1_prefix_function_Lst_Int_92` emit
  zero `cpl::at` calls and match the C++17 reference instruction-for-instruction, but
  top-level `solve()` still emits 3,000,000 `cpl::at` checks (`s[i]`, `z[i]`, `p[i]`) and
  1,000,000 `cpl::modp(x >> 16, 3)` calls despite `x & 0x7fffffff >= 0`.
- **Goal it hurts:** performance
- **Ideal code:** `for i in 0..n:` over `List.filled(n, 0)` and `(x >> 16) % 3` after
  masking to 31 bits, as written.
- **General fix:** extend loop-bound elision to globals initialized with `List.filled(n, ...)`
  that are never resized, and track non-negativity through bitwise `&` with a non-negative
  mask on globals.
- **Severity:** low
- **Status:** open
- **Area:** codegen

## Fixed ~1.5 ms `pthread_create` startup dominates short (~15–25 ms) benches

- **Where:** `cpl_on(1024 << 20)` in generated `main()` on `bench/strings/manacher_lyndon.cp`
  (16.0 vs 14.4 ms, +1.6 ms) and `bench/strings/z_prefix.cp` (26.6 vs 23.9 ms, +2.7 ms).
- **What felt wrong:** both `manacher`/`lyndon_factorization`/`min_rotation` and
  `z_function`/`prefix_function` are iterative (no recursion) and emit bounds-check-free
  C++17, yet every binary pays ~1–1.5 ms for `pthread_create`/`pthread_join` with a 1 GB
  stack (same root cause as `research/frictions/bench-flow.md`).
- **Goal it hurts:** performance
- **Ideal code:** nothing to write; iterative programs should not spawn a stack thread.
- **General fix:** spawn the big-stack thread only when the call graph contains recursion.
- **Severity:** low
- **Status:** open
- **Area:** codegen
