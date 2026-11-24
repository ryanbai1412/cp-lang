# atcoder_dp_y

Score: 5/5
Dream: 5/5

Cold review against the idiomatic checklist.

- R1: complete.
- R2: `clean[i]` = paths minus earlier walls' contributions, then the same for the goal.
- R3: `Comb<Z>` for binomials, `walls.sort()` on the record.
- R4: one `paths` serves every pair.
- R5/R6: `Cell` bounds both coordinates; walls are `Wall`.
- R10: 0.66x.

## Checklist failures

None.

## Owed rewrites

None.

## Language gaps

None.
