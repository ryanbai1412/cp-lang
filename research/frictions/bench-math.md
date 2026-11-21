# Friction report: bench/math (generated C++ costs found profiling `std.math`, `std.mod`, `std.ntt`, `std.matrix`)

Found while benchmarking `bench/math/run.sh` (best of 5, `g++ -std=c++17 -O2`, exact stdout match against matched C++17).
General fixes landed in the same commit cut `comb` from 1.58x to 1.15x and `matpow` from 1.05x to 0.98x (`U32`-narrowed struct fields proving non-negativity in `Facts::sign` so `Mod<P> * Mod<P>` drops `cpl::modp`, factoring `Comb.choose_lucas` so `Comb.choose` inlines, pre-allocating `Comb.new`, and `unchecked:` on proven-in-bounds `Comb`/`Sieve`/`Mat` loops).
Three rows remain slightly above 1.10x (`crt` 1.17x, `comb` 1.15x, `sieve` 1.11x); each remaining gap is a general codegen-only cost.

## `ext_gcd_x` is not inlined and pays `cpl::ediv` per Euclidean step

- **Where:** `std.math.crt` / `ext_gcd_x` in `bench/math/crt.cp` (`0.093s` vs `0.080s` C++, `1.17x`).
- **What felt wrong:** tuple destructuring (`var (r0, r1, x0, x1) = ...` and `(r0, r1) = ...`) expands `ext_gcd_x` to 20 C++ lines, above `inline_hint`'s `MAX_LINES = 16`, so every `crt` call makes a non-inlined call returning `std::tuple<ll, ll>`. Inside `while r1 != 0`, `r0 / r1` emits `cpl::ediv(r0, r1)` (zero-divisor check plus Euclidean sign fixup) on ~15M iterations, and `crt` emits five more `cpl::ediv`/`cpl::emod` calls because `Facts::sign` does not propagate parameter positivity (`m1 > 0, m2 > 0`) or loop/branch guards (`r1 != 0`, `g > 0`).
- **Goal it hurts:** performance
- **Ideal code:** `ext_gcd_x` and `crt` as written.
- **General fix:** emit tuple destructuring/swaps without inflating line count (or count IR statements in `inline_hint`), and let `Facts::sign` use simple non-zero / positivity guards from `while`/`if` conditions and `assert`s.
- **Severity:** low
- **Status:** open
- **Area:** codegen

## `Comb.choose` pays `cpl::modp` in the benchmark LCG plus signed `%` and lazy-grow checks

- **Where:** `bench/math/comb.cp` (`0.086s` vs `0.074s` C++, `1.15x`, down from `1.58x`).
- **What felt wrong:** after factoring `choose_lucas` and dropping `cpl::modp` from `Mod<P> * Mod<P>`, the remaining 12 ms over 5M queries (~2.4 ns/query) comes from three small codegen differences: `s = (...) & 2147483647` is a mutated `var`, so `Facts::sign` does not know `s >= 0` and emits `cpl::modp(s, lim + 1)` / `cpl::modp(s, n + 1)`; `Mod<P> * Mod<P>` emits signed 64-bit `% 998244353` rather than `uint64_t % 998244353`; and `Comb.choose` checks `n >= M.MOD` and `n < f.len` (lazy growth) on every query.
- **Goal it hurts:** performance
- **Ideal code:** `Comb<Z>(lim)` and `c.choose(n, k)` as written.
- **General fix:** track bitmask non-negativity across simple `var` updates (`x = ... & mask`) in `Facts::sign`, and emit unsigned `%` when both operands of `BinOp::Rem` are proven `NonNeg`/`Positive`.
- **Severity:** low
- **Status:** open
- **Area:** codegen

## `Sieve.phi_table` / `mobius_table` emit 64-bit `cpl::ediv(i, p)` vs 32-bit `idivl`

- **Where:** `Sieve.phi_table` and `Sieve.mobius_table` in `bench/math/sieve.cp` (`0.105s` vs `0.095s` C++, `1.11x`).
- **What felt wrong:** `let q = i / p` runs 10M times across `phi_table` and `mobius_table`. Because `spf` has element type `0..=2_000_000_000` (which includes `0`, so `p` is not statically `> 0`) and CP Lang widens `uint32_t` reads to 64-bit `ll`, codegen emits 64-bit `cpl::ediv(i, p)` (`idivq` + zero check + sign fixup) where matched C++ uses 32-bit `int` division (`idivl`).
- **Goal it hurts:** performance
- **Ideal code:** `let q = i / p` as written.
- **General fix:** narrow `BinOp::Div` to 32-bit when both operands are proven within `int32_t`/`uint32_t` bounds, and propagate `p != 0` / `p > 0` when proven.
- **Severity:** low
- **Status:** open
- **Area:** codegen

## Measurements

| Row | CP Lang | C++17 | Ratio | Emitted bytes |
|---|---:|---:|---:|---:|
| `modint` | 0.075s | 0.070s | 1.07x | 8,915 B |
| `ntt` | 0.030s | 0.036s | 0.82x | 15,958 B |
| `sieve` | 0.105s | 0.095s | 1.11x | 4,670 B |
| `matpow` | 0.042s | 0.043s | 0.98x | 7,753 B |
| `crt` | 0.093s | 0.080s | 1.17x | 6,428 B |
| `comb` | 0.086s | 0.074s | 1.15x | 10,021 B |

Aggregate: `p50 = 1.09x`, `max = 1.17x` (target: `p50 <= 1.1x`, `max <= 1.5x`).
