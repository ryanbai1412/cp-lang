# wtf26c

Score: 2/5

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
