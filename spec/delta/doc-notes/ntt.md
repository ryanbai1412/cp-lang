# std.ntt doc notes

Choices made where the spec is silent, and contradictions found, while bringing
`spec/std/ntt.cp` to the doc standard.

- `ntt`: chose the forward transform `â[k] = sum_j a[j] w^(jk)` with `w = primitive_root(P)^((P-1)/n)` in natural order, so two implementations give the same `â`, not just the same round trip.
- `ntt(bit_reversed: true)`: chose "forward leaves bit-reversed output, inverse expects bit-reversed input"; that is the only reading under which the flag is useful (pointwise products without reordering).
- `ntt`, `sos`, `fwht`, `subset_exp`, `subset_log` on length 0: chose to trap, because 0 is not a power of two (`is_pow2(0)` is false in std.math), rather than special-casing an empty no-op.
- `convolve(on: gcd | lcm)`: "index from 1" read as: position 0 contributes nothing and `c[0]` is zero; output length `max(a.len, b.len)`, with `lcm` values at or past it dropped. The spec gives no length; this keeps the stated O(n log log n).
- `convolve`: giving `shape:` with `Semiring(+, *)` is a compile error, symmetric with leaving it out for other semirings (which the stub already calls an error).
- `convolve` on `Int`: chose "traps when an output overflows `Int`"; math.md only says it is exact when every output fits, and silent wrong answers violate the safety rule.
- `convolve`: bitwise and `subset` forms need equal power-of-two lengths (math.md says so for bitwise); applied the same to `subset`, whose output length is that `2^k`.
- `fwht(inverse: true)` on `Int`: chose exact division, trapping on an entry not divisible by `n` (never happens on forward-transformed input).
- `RelaxedConvolve.push`, `SemiRelaxedConvolve.push`, `convolve_all`: chose to trap when the needed transform length exceeds what `P - 1` supports; the alternative (silent CRT fallback) would hide an algorithm switch.
- `SemiRelaxedConvolve.new`: `b` is zero-padded, so pushes past `b.len` are allowed and an empty `b` gives all-zero `c`.
- `convolve_all`: an empty factor makes the product `[]`, matching linear `convolve` with an empty input; the empty product is `[1]` (already stated).
- `subset_exp`: defined `b[m]` as the sum over set partitions of `m` of products of `a`; traps when `a[0] != 0`. `subset_log` traps when `a[0] != 1`.
- Contradiction: spec/lib/math.md §2.3 gives `ntt<P>(mut a, inverse: Bool = false)` without `bit_reversed`, and `sos<T>(mut a: List<T>, ...)` where the stub takes any `S: IndexedMut`. Left as in the stub. (resolved: stub kept; chapter fixed)
- Contradiction: spec/lib/math.md §1 names `sos_sub`, `sos_sup`, `sos_sub_inv`, `sos_sup_inv`; the stub and §2.3 have one `sos(over:, inverse:)`. Left as in the stub. (resolved: one `sos(over:, inverse:)` kept; chapter fixed)
