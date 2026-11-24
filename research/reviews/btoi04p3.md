# btoi04p3

Score: 4/5

Cold review against the idiomatic checklist.

- R1: complete; the Solution explains the `x_i = t_i - i` shift and the negated heap.
- R2: pool-adjacent-violators with median blocks is the editorial; the `break` hides the
  invariant (friction 3).
- R3: std `MeldHeaps` with `reserve`, not a hand leftist heap.
- R4: `len` duplicates `from`/`to`; the median is spelled four times (friction 1).
- R5: `Key` bounds the negated values; `t: [V]0..=2e9`.
- R10: 0.67x time, 0.67x memory.

## Checklist failures

R4, R13 (one-element temporary).

## Owed rewrites

`span: V.Range`, one `median` helper, the merge condition in the loop head.

## Language gaps

None; both spellings the dream needs were probed and work.
