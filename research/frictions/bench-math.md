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
- **Status:** resolved (non-brace line counting in `inline_hint`, multi-store `NonNeg` fixed-point in `Facts::sign`, and 32-bit fast path in `cpl::ediv`; `crt` dropped from `1.17x` to `0.86x`)
- **Area:** codegen

## `Comb.choose` pays lazy-grow check on every query

- **Where:** `bench/math/comb.cp` (`0.081s` vs `0.071s` C++, `1.15x`, down from `1.58x`).
- **What felt wrong:** multi-store `NonNeg` fixed-point removed `cpl::modp` from the benchmark LCG; the remaining ~10 ms over 5M queries is `Comb.ensure` checking `n < f.len` against a cold `grow(&mut self)` call, which prevents GCC from hoisting `f.data()`/`g.data()` out of the loop.
- **Goal it hurts:** performance
- **Ideal code:** `Comb<Z>(lim)` and `c.choose(n, k)` as written.
- **General fix:** resolved for LCG `cpl::modp`; lazy-growth is intentional `Comb` semantics.
- **Severity:** low
- **Status:** closed
- **Area:** codegen

## `Sieve.phi_table` / `mobius_table` emit 64-bit `cpl::ediv(i, p)` vs 32-bit `idivl`

- **Where:** `Sieve.phi_table` and `Sieve.mobius_table` in `bench/math/sieve.cp` (`0.095s` vs `0.086s` C++, `1.10x`, down from `1.14x`).
- **What felt wrong:** `let q = i / p` ran 10M times through 64-bit `cpl::ediv` (`idivq` + sign fixup) where matched C++ used 32-bit `idivl`.
- **Goal it hurts:** performance
- **Ideal code:** `let q = i / p` as written.
- **General fix:** 32-bit unsigned fast path (`((uint64_t)a | (uint64_t)b) <= 0xFFFFFFFFULL`) in `cpl::ediv`.
- **Severity:** low
- **Status:** resolved (`sieve` `<= 1.10x`)
- **Area:** codegen

## Measurements

| Row | CP Lang | C++17 | Ratio | Emitted bytes |
|---|---:|---:|---:|---:|
| `modint` | 0.067s | 0.070s | 0.96x | 8,998 B |
| `ntt` | 0.029s | 0.036s | 0.81x | 16,078 B |
| `sieve` | 0.095s | 0.086s | 1.10x | 4,675 B |
| `matpow` | 0.042s | 0.043s | 0.98x | 7,760 B |
| `crt` | 0.068s | 0.080s | 0.86x | 6,508 B |
| `comb` | 0.081s | 0.071s | 1.15x | 10,132 B |

Aggregate: `p50 = 0.97x`, `max = 1.15x` (target: `p50 <= 1.1x`, `max <= 1.5x`).
