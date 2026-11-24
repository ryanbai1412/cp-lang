# ccc20s1

Score: 4/5
Dream: 5/5

Cold review of `port.cp` and the dream (`## Dream` of the friction report, or `dream.cp`) against `handbook/guidelines/idiomatic-code-checklist.md`.

## Checklist failures

- R1: header gives the neighbour argument and exact printing.
- R2: sort by time, the largest neighbour speed as a `Ratio`, `fixed` to 5 places: the editorial, no floats.
- R4: the `(dx, dt)` pair is printed with `fixed(top.dx, top.dt, 5)` because `fixed` takes no `Ratio` (reported).
- R10: 0.02 s vs 0.01 s, start-up.

## Owed rewrites

None.

## Language gaps

- `fixed` takes no `Ratio` (in the report).

## Dream notes

One line that is the editorial.
