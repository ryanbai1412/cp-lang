# cco22p2

Score: 4/5
Dream: 4/5

The greedy from the header: each market fills its left stop, then its right, then buys; a shortfall pulls spare umbrellas from earlier markets off a stack and shifts people through a `Diff`. Long, but every loop is a step the header names.

## Checklist failures

- idiomatic-code/R6: `room[Stop(i.ord)]`, `room[Stop(i.ord + 1)]` (lines 32-33); reported.

## Owed rewrites

- None.

## Language gaps

- No inner-cuts domain: research/frictions/cco22p2.md (repro added by this review).
- The capped run stack is hand-kept: same report.

## Dream notes

- `Stop.inner_cuts`, `spare.take_top(need)` and `spare.keep(right)` remove the ordinals and both stack loops.
