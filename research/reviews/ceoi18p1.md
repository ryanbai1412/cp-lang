# ceoi18p1

Score: 3/5
Dream: 4/5

Cold review against the idiomatic checklist.

- R1: complete; the Solution states the sweep order and the knapsack.
- R2: the sweep is the editorial, but the tie rule is a sign trick in a positional tuple.
- R4: `top = 50 * n` restates a bound the data gives; the deal ordering lives in a sign.
- R5: `?Int` for unreachable core counts; `Offer` bounds every field.
- R8: the final max is a hand loop over `?Int`.
- R10: 1.07x.
- Dream: named records and `desc(d.clock)` say the order; rebuilding `best` per deal is
  clearer but allocates 2000 times.

## Checklist failures

R2, R4, R8.

## Owed rewrites

Named deal records with an explicit sort key; `Free` from the machines' cores;
`best.present().max()!`.

## Language gaps

None.
