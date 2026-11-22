# std.mod doc notes

Choices made where the spec is silent, and contradictions found, while bringing
`spec/std/mod.cp` to the doc standard.

- `Mod.try_inv` with `P == 1`: chose `Some(Mod<1>(0))`, since `gcd(0, 1) == 1` and 0 is `one` there; consistent with the stated rule `gcd(v, P) == 1`.
- `Mod.from_int`: chose the same Euclidean reduction as `new` (negative inputs included); the stub only says "lift an `Int` literal".
- `Mod.sqrt`: chose to return the root with the smaller value `v` of `r` and `P - r`, so implementations agree; traps when `P` is not prime (Tonelli-Shanks needs a prime; the old doc said "mod prime `P`").
- `Mod.log`: chose the least `k >= 0`, for any modulus (non-coprime part peeled off, the standard extended BSGS); `x.log(one)` is `Some(0)`.
- `Mod.inv_all`: traps on any non-invertible element, not only 0 (the old doc said "traps on 0", which is the prime-`P` special case).
- `Comb.new`: chose to trap when `limit < 0` or `P` is not prime (the struct doc already requires prime `P`; Lucas and `inv_fact` are wrong otherwise); tables are capped at `P` entries per the math.md invariant `f.len <= P`.
- `Comb.fact`, `Comb.catalan`, `Comb.derange`: chose to trap on `n < 0` (undefined counts), while `choose`, `arrange`, `multichoose` return 0 outside their domain as the stub states for `choose`.
- `Comb.arrange` for `n >= M.MOD`: chose the exact falling factorial modulo `P` (0 if the `k` factors include a multiple of `P`, else `fact(n % P) / fact((n - k) % P)`), the analogue of `choose` using Lucas.
- `Comb.multichoose`: chose `choose(n + k - 1, n)`, with `multichoose(0, 0) == 1` and negative arguments giving 0.
- `Comb.catalan`: chose the form `choose(2n, n) - choose(2n, n + 1)`, which stays exact when `n + 1` is a multiple of `P`.
- `Comb.multinomial`: chose the product of prefix `choose` calls, so Lucas applies past `M.MOD`; the empty list gives 1.
- `Barrett.mul`/`Barrett.reduce`: chose to trap outside the stated input ranges (`a, b` in `0..m`; `a < m * m`) rather than return an unspecified residue.
- `PrimePowerComb.new`: traps when `p` is not prime, `e < 1` or `p^e` overflows; `choose` returns 0 outside `0..=n`. `ModComb.new` traps when `m < 1`; `m == 1` gives 0.
- Contradiction: spec/lib/math.md §2.2 declares `@get fn inv(self)`, the stub has a plain `pub fn inv(self)`. Left as in the stub; docs use `x.inv()`.
- Contradiction: spec/lib/math.md §2.2 lists `fn pow(self, e: Int)` as a method; the stub has `impl Pow` instead. Not a pub fn, so not documented here.
- Contradiction: spec/lib/math.md §2.2 does not list `Mod.from_int`, `sqrt`, `log`, `inv_all`, `Comb.derange`, `Barrett`, `PrimePowerComb` or `ModComb`; the stub has them. Left as in the stub.
