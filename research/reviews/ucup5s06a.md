# ucup5s06a

Score: 4/5

Dream: 4/5

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

## Cold review (HEAD)

Dream 4/5 (new), port → 3/5, rescored on current main. The header derives `cyc(B)` as a weighted cyclic block count, and the program is three lines.
- `Series` (lines 8-17) and the dependent bound `(Int, 1..n)` (line 14) are reported gaps.
- Minor: "the editorial's answer is cyc(B + x) - 2 cyc(B)" (line 6) is cited, not derived. Say what the `+ x` and the `- 2` count.

Port 3/5: bare `List<Z>` series with `inv(..., n + 1)` and `Z(Int(i == 0))`/`Z(Int(i == 1))` indicator arithmetic (lines 15, 19); these are the reported gaps.

## Cold review (HEAD)

Port 3 → 4/5, rescored cold on current main; dream 4/5 holds. The port checks with zero errors. Immutable `b` and `b_or_single` (lines 20-21) remove the in-place `f[1] -= 1`, and `cyc` (lines 16-18) is the header's formula. The header matches the 4/5 dream, including the cited `cyc(B + x) - 2 cyc(B)`.
- Minor: `(n, k): (Int, Int)` (line 14) is unbounded where the dream has `(Int, 1..n)`.
