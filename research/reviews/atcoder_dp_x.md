# atcoder_dp_x

Score: 5/5
Dream: 5/5

Cold review against the idiomatic checklist.

- R1: complete.
- R2: sort by s + w, then 0/1 knapsack downward over `..=s`: the editorial.
- R5: `?Int` for unreachable weights, no sentinel; every block field bounded.
- R12: `Weight(x.label + b.w)` is in range because `x <= s <= 1e4` and `w <= 1e4`, as the
  Solution states.
- R10: 1.24x on 6.8 ms, reported (layout, Core).

## Checklist failures

None in the code; R10 reported.

## Owed rewrites

None.

## Language gaps

None.
