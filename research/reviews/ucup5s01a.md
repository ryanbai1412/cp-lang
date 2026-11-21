# ucup5s01a

Score: 3/5

The header (lines 1-4) states the state and the initial polynomial, and the transition comments (lines 31-39) read like the editorial. The polynomial product is hand-rolled, and the closing `/ i` and `/ (n + z)` divisions are not derived. No dream exists, so the score is capped at 3. Scored cold against `design/EXEMPLARS-PORTS.md`.

## Checklist failures

- Hand polynomial product (lines 14-21): `ulen` and the reverse in-place update.
- Unexplained closing (lines 42-43): why the answer sums `dp[i, 0] / i` and divides by `n + z` per step.
- `size = n + m + 2` (line 23) unexplained; the `+ 2` is slack, not a bound.
- Rebuilding `next` each step (lines 27-40) instead of a rolling layer.
- Mixed spelling: free `count(...)`, `sum(...)` (lines 13, 42).

## Owed rewrites

- Header deriving the closing; method `count`/`sum`; a tight `size`; a dream.

## Language gaps

No friction report exists. Missing and owed as a friction: a product of linear factors `prod (d + x)` (or `Poly` multiply) for `Mod` coefficients.
