# atcoder_dp_h

Score: 4/5
Dream: 5/5

Cold review of `port.cp` against `dream.cp` (the same code), without the C++. The walls are a
filter on the `for`, `r.prev`/`c.prev` take the edges with no padding, and `Mod<1e9+7>` keeps
the arithmetic in the type.

## Checklist failures
- idiomatic-code/R10: 1.21x on a 9 ms run, with the `Mod` table fill as one measured part. Routed
  to Core in the friction; nothing in the port to change.

## Owed rewrites
None.

## Language gaps
- A `Mod` table filled with zero takes the fill path, not the zeroed allocation (Core).

## Dream notes
Exemplar grid DP.
