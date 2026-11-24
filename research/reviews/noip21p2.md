# noip21p2

Score: 3/5
Dream: none (no dream.cp; the ideal is in the friction report)

Scored cold against the checklist and the port. With no dream.cp the port scores at most 3 (README); against the checklist alone it reads as a 4.

The port is a carry DP over the m + 1 weights: `dp[used, carry, ones]` counts the ways to place the first `used` picks. It is a direct digit DP with `Comb` and a running power, so it agrees with the v0.2 port and the accepted C++.

## Checklist failures

- `if cur == 0: continue` skips empty states by testing a `Mod` value for zero, which is an optimisation in the C++'s style, not part of the recurrence.
- `var power = Z(1)  // w ** take` keeps a running power beside the `take` loop instead of `w ** take`.
- `type Layer = [0..=n, 0..=n / 2, 0..=k]Z` and `dp = next` roll a 3-D table by full reassignment. The axes have no domain names (`Used`, `Carry`, `Ones`).
