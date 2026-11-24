# atcoder_dp_l

Score: 3/5
Dream: 5/5

Cold review of `port.cp` against `dream.cp` (the same code), without the C++. `lead: [I.ranges]Int`
indexed by the range itself, `r.drop(1)` and `r.drop_last(1)` are the editorial's
`dp[l+1][r]`/`dp[l][r-1]` with no index arithmetic in sight, and `I.ranges` gives the
by-length order.

## Checklist failures
- idiomatic-code/R10: 3.3x the accepted C++ at `--max`, from the range table's layout and index
  code (measured in the friction, routed to Core). The code is the dream, so the 3 comes from a
  measured std/codegen gap, not from translation.

## Owed rewrites
None in the port.

## Language gaps
- `[V.ranges]` tables are end-major and pay per-read index checks (Core).

## Dream notes
Exemplar interval DP, as code.
