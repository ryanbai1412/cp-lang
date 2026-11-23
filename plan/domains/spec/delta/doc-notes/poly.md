# Doc notes: `std.poly`

- Examples write `type Z = Mod<998244353>;` and typed `let`s, since the free
  fns are generic in `P` and take plain lists.
- `inv` / `log` / `exp` / `pow` / `sqrt` / `compose` / `eval_all` /
  `transposed_mul` / `SubproductTree.new`: chose to list "a transform length
  exceeds what `P` supports" (the power of two dividing `P - 1`) as a trap.
- `integral`, `log`, `exp`, `taylor_shift`, `stirling2_*`, `bell_table`,
  `bernoulli_table`, `lagrange_at`, `sample_points_shift`: trap when they need
  the inverse of a multiple of `P` (sizes at or above `P`); the stub is silent.
- `pow`: `k == 0` gives `[1, 0, ...]` even for a zero `a` (math.md §3 says
  "agrees with repeated `convolve` (including leading zeros and `k == 0`)");
  `k < 0` with `a[0] == 0` traps.
- `sqrt`: chose the root whose lowest nonzero coefficient is the smaller
  square root (as an integer in `0..P`) of `a`'s; the stub does not say which
  of the two roots. An `a` that is zero below `x^n` gives all zeros.
- `div_mod`: `b`'s trailing zeros are ignored; `q` has
  `max(0, deg a - deg b + 1)` coefficients and `r` is trimmed (zero is `[]`).
  The stub fixes only `r.len < b.len`.
- `interpolate` / `SubproductTree.interpolate`: return exactly `xs.len`
  coefficients (trailing zeros kept); mismatched lengths and repeated points trap.
- `transposed_mul` (free and `Poly`): chose the complete middle product, length
  `max(0, a.len - b.len + 1)`; the stub gives the formula but not the length.
- `pow_mod_poly`: result has exactly `deg mod_p` coefficients; `k < 0` or a zero
  `mod_p` traps.
- `gcd_poly`: monic, trailing zeros trimmed; `gcd(0, 0)` is `[]`.
- `SubproductTree.new(keep)`: chose that `eval` and `interpolate` trap on a tree
  built with `keep >= 0`, and `eval_transposed` traps when `keep` is in `0..m`;
  the stub says only that `keep` truncates the products.
- `SubproductTree.eval_transposed`: chose output length `m`,
  `out[k] = sum_i c[i] * xs[i]^k`, and `c.len == m`.
- `stirling1_col` / `stirling2_col` return `n + 1`
  values, zero for rows below `k`.
- Contradiction: `Poly.deg` and `Poly.is_zero` say O(1), but `Poly.new`, `set`
  and `truncated` keep trailing zeros, so the degree needs a scan of them.
  Documented "O(1) plus the number of stored trailing zeros". (resolved: `deg()`/`is_zero()` are calls, decision 166)
- Contradiction: `Poly.zero`, `one` and `x` are `@get` with no `self`; every
  other static constructor in the library is a plain `fn` (`Subset.empty()`). (resolved: kept `@get`: decision 62 does the same for `Subset.empty`/`full`)
- Contradiction: lib/math.md §4 says multipoint evaluation, interpolation,
  Taylor shift, Bostan-Mori, polynomial GCD and composition stay in
  `examples/local-lib`; the stub declares them all in `std.poly`. (resolved: kept stub; math.md §4 fixed)
- Contradiction: lib/math.md §2.4 makes `Mod.inv` an `@get`; mod.cp has
  `pub fn inv(self)`. Not this module's, noted because poly examples divide. (resolved: already fixed: math.md §2.4 writes `fn inv(self)`, O(log P), so a call is right)
