# ucup5s03a

Score: 2/5

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
