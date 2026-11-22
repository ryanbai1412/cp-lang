# wtf26c

Score: 4/5
Dream: 4/5

The header names the method (2D DP, NTT per step over flattened coordinates, two inclusion-exclusion passes) but not the recurrence; the flattening `i + stride * j` is hand bookkeeping, and the closing passes are unexplained. No dream exists, so the score is capped at 3. Scored cold against `design/EXEMPLARS-PORTS.md`.

## Checklist failures

- Header (lines 1-2) omits what `dp[i, j]`, `claim` and `ways_groups[g]` count and why the two passes (lines 51-57) invert to the answer.
- Hand power tables (lines 16-20) where `Z(k) ** i` or a powers helper says it.
- Hand 2D-to-1D flattening (lines 31-47): `stride = 2 * cur + 1`, `idx = i + stride * j`; the stride is a convolution-safety bound the header never mentions.
- Copy into lists for `convolve` (line 44): `[mul_a[t] for t in 0..sz]`.
- Raw `[0..n]`, `[0..=n]` index spaces; `cur + 1 - i`, `j + 1` weights (lines 39-40) unexplained.
- Unbounded input (line 9): `Int`.

## Owed rewrites

- Header with the recurrence and the inversion; name the flattened 2D convolution; bound `n`; a dream.

## Language gaps

No friction report exists. Possibly missing: a 2D (flattened) convolution in `std.ntt`; needs a fresh-compiler check before filing.

## Cold review (HEAD)

Dream 4/5 (new), port → 3/5, rescored on current main. The header gives the DP, the `claim` table and both inversions.
- The only error is `convolve_2d` (line 12), which is reported.
- Minor: the first inversion is `binomial_inverse()` (line 38), but the second is a hand loop (line 40); give both the same spelling or say why the second isn't a binomial inverse. `n: Int` (line 17) has no bound, so the O(n³) `power_sum` cost isn't checked.

Port 3/5: a hand-flattened 2D convolution (lines 9-14) and both inversions as loops (the reported gap).

## Cold review (HEAD)

Port 3 → 4/5, rescored cold on current main; dream 4/5 holds. The port checks with zero errors. The header derives the DP and both binomial inversions. `Z(g - 1) ** i * Z(g) ** j` (line 32) replaces the power tables, and `convolve_2d` (lines 22-27) states its stride invariant. It is a small stand-in for the reported `convolve_2d`.
- Minor: `n: Int` (line 17) has no bound, so the `n` convolutions of `O(n^2)` terms per `g` (line 39) have no visible cost bound.
