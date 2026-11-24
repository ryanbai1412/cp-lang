# btoi15p2

Score: 4/5

Cold review against the idiomatic checklist.

- R1: complete.
- R2: "the active operations form a chain; an undo of level L jumps below the topmost op
  of level < L" is the editorial, and `below` is that one search.
- R4: `Lg = 0..19` restates a bound of n (friction 1).
- R5: `?Op` for "no operation below", no `-1`; `ops: [Op]-n..=n`.
- R6: `Op` and `Lg` distinct.
- R10: 0.18x time, half the memory.

## Checklist failures

R4 (jump depth literal).

## Owed rewrites

Derive `Lg` from `n`; start the fill at `Lg.drop(1)` (frictions 1-2).

## Language gaps

None confirmed.
