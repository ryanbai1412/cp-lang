# ucup5s06a

Score: pending (header derives `cyc(B)` as a cyclic block count and states the answer `cyc(B + x) - 2 cyc(B)`; second series is its own value, not an in-place `f[1] -= 1`; the `k == 1` exit is gone since it is `cyc(x) = 1`)

Dream: pending (new dream)

Short, and `log_deriv_at` names the editorial's tool, but the second series is made by mutating the first in place and the closing `2a - b` is not derived. No dream exists, so the score is capped at 3. Scored cold against `design/EXEMPLARS-PORTS.md`.

## Checklist failures

- Header (lines 1-2) names "two logarithmic derivatives" but not which series or why `2a - b`.
- Hidden second series (line 24): `f[1] -= 1` turns `f` into `f - x` between the two calls; declare `g = f - x` as its own value.
- Unexplained closing formula (line 25): `Z(2) * a - log_deriv_at(f, n)`.
- Special case (lines 15-17): `k == 1` exit without a reason.
- Mixed spelling (line 11): free `sum(...)`. Withdrawn: §53.1 keeps the free generator form canonical.
- Unbounded input (line 14): `(Int, Int)`.

## Owed rewrites

- Header deriving the formula; `let g = f - x`; bound `n` and `k`; method `sum`.

## Language gaps

No friction report exists. None needed.
