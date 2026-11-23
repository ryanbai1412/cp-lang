# Doc notes: `std.matrix`

- `Mat.new`: row `i` / column `j` are the `i`-th positions of `R` / `C`; the
  domain keys are not kept (a `Mat` is `Int`-indexed). The stub is silent.
- `Mat.get` / `set` / `at`, `filled`, `zeros`, `identity`, `from_flat`,
  `from_rows`: chose to trap on out-of-range indices, negative sizes, a flat
  list of the wrong length, and ragged rows; `from_rows([])` is `0 x 0`.
- `Mat.mul_vec`, `solve*`, `MatPow.apply_pow`, `BitMat.mul`, `simplex`,
  `sparse_solve`: dimension mismatches trap.
- `Mat.det` / `pfaffian` / `det_real` / `BitMat.det`: the `0 x 0` value is one
  (`true`); `det_mod` gives `1 % m`; non-square input traps.
- `Mat.solve` / `solve_space` / `BitMat.solve*` / `solve_real`: chose free
  variables zero for the particular solution, and one nullspace vector per
  free column (1 there, 0 at other free columns), by increasing column, so
  the result is unique. The stub fixes only "free variables 0" for `solve`.
- `Mat.rref`: RREF with unit pivots and increasing pivot columns (unique).
- `Mat.det_mod`: entries are reduced modulo `m` first, so negative entries
  work; the result is in `0..m`; `m < 1` traps.
- `Mat.pfaffian`: reads only the entries above the diagonal and does not check
  skew-symmetry; sign convention `pf([[0, a], [-a, 0]]) == a`; odd `n` traps.
- `Mat.adjugate`: transpose of the cofactor matrix; zero when `rank < n - 1`.
- `det_real` / `rank_real` / `solve_real` / `inverse_real`: an entry counts as
  zero when `abs(x) <= eps`; pivot = largest `abs` in the column. Examples
  compare floats with `abs(x - y) < 1e-9` (math.md: float `==` is rejected).
- Contradiction: lib/math.md §2.4 lists `Mat(n, m, a)`, `zero(n, m)`, `of(rows)`,
  `pow`, `apply`, `rows()` and an `impl Neg`; the stub has `rows`/`cols`/`data`,
  `zeros`, `from_rows`, `impl Pow`, `mul_vec`, and no `apply`, `rows()` or `Neg`.
  math.md §3's laws are written with `A.apply(...)`. (resolved: kept stub; math.md §2.4 note maps the names)
- Contradiction: `XorBasis` says 63-bit values (`0 <= x < 2^63`, `W <= 63`);
  lib/math.md and `basis_vecs` / `intersect` costs say 62. Documented "x >= 0". (resolved: kept 63-bit; `basis_vecs`/`intersect` costs O(W); math.md fixed)
- Contradiction: lib/math.md gives `XorBasis.new()` O(1) and `add` O(log x);
  the stub says O(W) for both. Documented the stub. (resolved: kept stub; math.md fixed)
- Contradiction: lib/math.md §4 says simplex/LP stays in `examples/local-lib`;
  the stub declares `simplex` in `std.matrix`. (resolved: kept stub; math.md §4 fixed)
- `XorBasis`: chose to keep the basis fully reduced (O(W) per `add`), so
  `basis_vecs` is unique for a span; returned in decreasing order.
- `XorBasis.add(0)` is `false`; negative inputs to `add`/`reduce`/`contains`/
  `max_xor`/`min_xor` trap.
- `PrefixXorBasis.push`: chose to trap unless `0 <= x < 2^62`; a non-empty
  range reaching outside `0..len` traps; an empty range spans `{0}`.
- `MatPow.new`: stores `a^(2^k)` for `2^k <= max_exp`; `max_exp < 0` traps.
- `sparse_det` / `sparse_solve`: repeated cells add up; chose a fixed seed and
  retry-until-certified, so results are deterministic and exact and only the
  time is expected (goal 4: "Everything is deterministic").
- `sparse_solve`: chose `None` for any singular `A`, even if `b` is in range
  (Wiedemann solves non-singular systems only).
- `simplex`: chose two-phase with Bland's rule; with several optima `x` is the
  vertex reached. The stub does not say which optimal `x`.
