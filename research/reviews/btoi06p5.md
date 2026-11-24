# btoi06p5

Score: 3/5

Cold review against the idiomatic checklist.

- R1: complete; the Solution states the four DP updates and why one lazy tree serves.
- R2: decode into runs, backward DP over escapes, forward replay: the editorial order.
- R3: the assign-then-add action is hand-written (friction 1).
- R5: `None` for the forbidden escape, no infinity sentinel; run lengths are unbounded `Int`.
- R7: `add_run` mutates the global `runs`; it is named for that and used only in the decode.
- R10: 0.90x time, 1.76x memory (friction 2).

## Checklist failures

R3, R5 (run length), R10 (memory, reported).

## Owed rewrites

Use a std assign-add action once it exists; bound the run length.

## Language gaps

No std assign-then-add lazy action (Content).
