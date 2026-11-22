# std.math doc notes

Choices made where the spec is silent, and contradictions found, while bringing
`spec/std/math.cp` to the doc standard.

- `gcd`, `ext_gcd`: chose to trap when the result is `2^63` (`Int.MIN` with 0 or itself), the only overflow case.
- `lcm`: "stops on overflow" read as a trap (math.md says "traps on overflow"); computed as `|a| / g * |b|` so only a true overflow traps.
- `ext_gcd`: pinned the Bezout pair to the iterative extended Euclid with Euclidean division, negated if `g < 0`; `ext_gcd(0, 0) == (0, 1, 0)`. Without this, two implementations return different valid pairs.
- `iroot`, `nth_root`, `isqrt`, `ceil_sqrt`, `bit_len`, `digits`: chose to trap on the stated preconditions (`x < 0`, `k < 1`, `base < 2`) rather than return garbage.
- `fixed`: chose that `k == 0` prints no point and a negative `den` flips the sign; traps on `den == 0`, `k < 0` and the stated overflow bound.
- `choose`, `choose2`: negative `n` gives 0 (`k` outside `0..=n`); overflow traps.
- `ceil_div`, `divmod`: trap on `Int.MIN / -1` overflow as well as a zero divisor.
- `Sieve.new`: traps on `n < 0` or `n > 2_000_000_000` (the field element type bound).
- `Sieve.*` queries: trap outside `0..=n` (`is_prime`) or `1..=n` (others); tables put 0 at index 0 (`phi`, `d`, `sigma`, `mu`).
- `Sieve.divisors`, `Sieve.each_divisor`, `each_divisor`: the stub says "in no fixed order"; chose a fixed mixed-radix order (smallest prime's exponent fastest), so outputs are deterministic across implementations at no cost. Still not sorted.
- `sieve_multiplicative`: index 0 holds `T.one` (stated); traps on `n < 0`.
- `bit`: chose sign extension for `i >= 64` (all set for negatives, consistent with the stated "no bit from 63 up is set" for non-negatives); traps on `i < 0`.
- `splitmix64`: pinned to the full step (add the golden gamma, then mix), so `splitmix64(0) == 0xE220A8397B1DCDAF` and `Rng.next` is `splitmix64` of its state.
- `divisors_from_factors`: input is trusted as distinct primes (not checked); empty gives `[1]`.
- `inv_mod`: modulo 1 the inverse is `Some(0)`; `a` is reduced first.
- `crt`, `crt_all`: residues may be any `Int`; trap on a modulus below 1 or lcm overflow.
- `pow_mod`: `a ** 0` is `1 % m`, so `pow_mod(5, 0, 1) == 0`.
- `mod_log`: `a^0 = 1 % m`; `a` and `b` are reduced first.
- `mod_sqrt`: returns the smaller of the two roots; traps on non-prime `p`.
- `mod_kth_root`: the stub says "any `x`"; chose the least root so results are deterministic, adding O(gcd(k, p - 1)) to the cost; traps on `k < 1`.
- `primes_in`: an empty or sub-2 range gives `[]` instead of trapping on `lo < 0` or `lo > hi`.
- `prime_count`, `sum_phi`, `sum_mobius`: 0 for arguments below their domain instead of trapping.
- `floor_quotients`: yields nothing when `n < from`; traps on `from < 1`.
- `bit_subsets`: traps when `n > 62` (Gosper needs the bit above the top); `k > n` yields nothing; `k == 0` yields 0.
- `dirichlet_convolve`, `dirichlet_inv`: lists are indexed by the integer itself; index 0 is ignored and the result has `T.zero` there; traps on a list shorter than `n + 1`.
- `DirichletSum.new`: chose `f_small[k] = f(k)` (values, not prefix sums), index 0 ignored, and stated the Du recursion so `g_sum`/`fg_sum` have one meaning; `sum(m)` traps outside `0..=n`.
- `sum_multiplicative`: chose that `prime_sum` lists the distinct `n / i` in ascending order of value; traps on a wrong length.
- `linear_diophantine`: chose the canonical solution (`x0` the least non-negative `x` when `b != 0`; `x0 = c / a, y0 = 0` when `b == 0`), steps `b / g` and `a / g` with signs; traps when `a == b == 0`, where the solutions are not a one-parameter family.
- `best_rational`: ties go to the smaller denominator, then the smaller value.
- `continued_fraction`: floor division, so `a0` may be negative; the last term is at least 2 except in a one-term expansion.
- `Ratio`, `BigInt`: `Ratio.new` puts the sign in the numerator; `BigInt.from_str` accepts an optional `-` and one or more digits (leading zeros allowed, no `+`, no whitespace); `BigInt.divmod` is Euclidean like `Int`.
- Contradiction: math.md §2.1 has `@get fn sign(self: Int) -> Int`; the stub returns the `Sign` enum. Docs follow the stub. (resolved: stub `Sign` kept; chapter fixed)
- Contradiction: math.md §2.1 has `sieve_multiplicative(n, one, mul, prime_pow)`; the stub has `(n, prime_pow)` and relies on `T.one` and `*`. Docs follow the stub. (resolved: stub kept (`T.one`, `*`); chapter and example fixed)
- Contradiction: math.md §2.1 types `Sieve.spf` and `primes` as `List<Int>`; the stub uses `List<0..=2_000_000_000>`. Docs follow the stub. (resolved: stub types kept; chapter fixed)
- Contradiction: math.md gives `crt_all(eqs: List<(r: Int, m: Int)>) -> ?(r: Int, m: Int)` with named fields; the stub uses unnamed tuples. Docs follow the stub. (resolved: stub kept; chapter fixed)
- Contradiction (duplication): `iroot` and `nth_root` are the same function, as are `Sieve.divisors` and `Sieve.each_divisor`, and `each_divisor(x)` is `divisors(x)` unsorted. "One way per job" says one of each pair should go. (resolved: `nth_root` and `Sieve.each_divisor` removed; free `each_divisor` kept (no allocation, unsorted) beside sorted `divisors`; decision 167)
