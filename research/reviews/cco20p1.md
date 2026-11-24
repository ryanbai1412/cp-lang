# cco20p1

Score: 3/5
Dream: 3/5

Each friend's view as a clipped interval, then a hand sweep of sorted start/end events to count positions by coverage, then prefix sums. Correct, but the sweep is a standard algorithm std lacks (reported).

## Checklist failures

- idiomatic-code/R3: the coverage sweep (lines 19-33) is hand-rolled; the gap is in the friction.
- idiomatic-code/R13: `if lo <= hi:` (line 23) always holds: `f.x` is in `l..=r` by type and `reach >= 0` since `h, y >= 1`.

## Owed rewrites

- Drop the `lo <= hi` guard.

## Language gaps

- No `coverage(intervals, over:)`: research/frictions/cco20p1.md.

## Dream notes

- `(f.h * y - 1) / f.v` is written twice in one comprehension (idiomatic-code/R4, caps at 3); bind `reach` as the port does. Otherwise `coverage(...)` then `prefix_fold` is the editorial.
