# ucup5s02e

Score: pending cold rescore (added dream, replaced hand inversion loop with rank[y].inverse(), derived why running max reach[i] == i implies equal prefix sets; emitted C++ 8,550 -> 9,852 B from .inverse() helper, <48 KB)
Dream: pending cold rescore (new dream: rank[y].inverse(), reach[i] == i bijection derivation)

Baseline review, cold, on main 6566cb18. The port checks with zero errors. The header (lines 1-3) states the rule, and the pair loop is short. No dream exists, so the port is capped at 3.

## Checklist failures

- Lines 12-13 invert `rank[y]` by hand; `rank[y].inverse()` does it (I probed it on a `[0..n]P`).
- Why `reach[i] == i` (line 15), a running max of positions, means equal prefix sets isn't said.

## Rewrites owed with today's language

- Write a dream using `.inverse()`.

## Gaps that need the language

- None found.
