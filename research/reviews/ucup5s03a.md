# ucup5s03a

Score: 4/5
Dream: 4/5

The header says only that the answer is a quotient of two series; the series are assembled by hand-placed coefficients at unexplained offsets. No dream exists, so the score is capped at 3. Scored cold against `design/EXEMPLARS-PORTS.md`.

## Checklist failures

- Header (lines 1-2) does not name the two series or why their quotient counts the answer.
- Unexplained coefficient layout (lines 24-33): `2 * i + 1`, `2 * i + 2`, `2 * i + 3`, the `all[1] -= k + 1` term and `[0..n + 5]` padding.
- Unexplained closing term (line 36): `+ n % 2`.
- Opaque names (line 23): `neg_st`, `neg_all`, `pos_st`, `pos_all` from `egf(0/1, 0/1)` flags.
- Unbounded input (line 7): `(Int, Int)`.

## Owed rewrites

- Header deriving both series; name the four `egf` variants by meaning; write the series as sums of shifted series (`x * f(x^2)`) rather than index offsets; bound `n` and `k`.

## Language gaps

No friction report exists. Possibly missing: series substitution (`f(x^2)`, shift by `x^k`) in `std.poly`; needs a fresh-compiler check before filing.

## Cold review (HEAD)

Dream 4/5 (new), port → 3/5, rescored on current main. Once `Fps` exists, lines 25-27 are the header's formula.
- The only errors are `Fps` and `X` from `std.poly` (line 10), which are reported.
- Owed (minor): `S` and `A` (header lines 7-9) are stated, not derived. `(n, k): (Int, Int)` (line 13) has no bounds, so `Comb<Z>()` has no stated size.

Port 3/5: hand `spread` substitution and an EGF `convolve` with index arithmetic (the reported series gap).

## Cold review (HEAD)

Port 3 → 4/5, rescored cold on current main; dream 4/5 holds. The port checks with zero errors. `std.poly.inv` and `convolve` replace the hand series code. `binomial_transform` and `spread` (lines 20-30) are the header's `T[ds, dw]` and `x^shift f(x^2)`.
- Minor: `s` and `a` (lines 34-39) are built coefficient by coefficient where the dream writes `Fps` sums (reported). `(n, k): (Int, Int)` (line 14) has no bound.
