# wtf26b

Score: pending cold rescore (restructured to O(n^4) via suffix sums suf[y, k, cut] = sum_{y'>=y} dp[y', k, cut] dropping ~1e11 O(n^5) to ~1.3e8 O(n^4) at n=200, explained dp[x, x, 0] = 1 base seed; emitted C++ 10,713 -> 12,479 B from suffix-sum pass, <48 KB)
Dream: pending cold rescore (O(n^4) suffix-sum DP + base-seed derivation)

Baseline review, cold, on main 4cf4b2b1. Port and dream are the same file and check with zero errors. The header (lines 3-11) states every transition.

## Checklist failures

- Cost is unstated and looks too large: at `n = 200` the nest over `i`, `x`, `y`, `k`, `cut` (lines 22-28) visits on the order of 1e11 cells; the `!= 0` filter (line 28) still visits each one. Either argue why the reachable state space is small or restructure (e.g. prefix sums over `y`).
- The seed `dp[x, x, 0] = 1` (line 20) sets run length `k = x` for the last position; the header doesn't say why.

## Rewrites owed with today's language

- State and justify the running time, or restructure so it fits; explain the seed.

## Gaps that need the language

- None found.
