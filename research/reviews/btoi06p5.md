# btoi06p5

Score: 4/5

Cold review against the idiomatic checklist.

- R1: complete; the Solution states the four DP updates and that one min SegTree with std's
  `assign_add` serves them, folding either side of the current escape.
- R2: decode into runs, backward DP over escapes, forward replay: the editorial order.
- R3: passes: `act: assign_add` from std, `right_while` for the argmin.
- R5: no infinity sentinel and no forbidden-escape `None`; run lengths are unbounded `Int`.
- R7: `add_run` mutates the global `runs`; it is named for that and used only in the decode.
- R10: 0.99x time, 1.59x memory (friction 2).

## Checklist failures

R5 (run length), R10 (memory, reported).

## Owed rewrites

Bound the run length (`len: 1..=2e6`).

## Language gaps

None open (`assign_add`, e9b4b0aa5).
