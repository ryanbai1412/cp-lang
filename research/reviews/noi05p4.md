# noi05p4

Score: 4/5
Dream: 4/5

Port 4/5, dream 4/5 (`dream.cp`). BFS distances, the cat's greedy step `toward` (line 18), and the memoised expected turns over the mouse's stay-or-move choices (lines 20-27): the editorial's DP, the same as the dream line for line.

## Checklist failures

None beyond the friction: `chain([m], g.next(m)).mean(...)!` (line 27) unwraps a mean of a sequence that is never empty, reported.

## Owed rewrites

None.

## Language gaps

- Non-empty sequences whose `mean` is total: in the friction.

## Dream notes

The dream keeps the same `!`; once non-empty chains exist it should drop it and be 5.
