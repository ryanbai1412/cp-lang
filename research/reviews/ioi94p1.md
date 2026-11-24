# ioi94p1

Score: 4/5
Dream: 4/5

Cold review: the triangle DP in one comprehension per row (line 14), best of the last row at the end; the row type `[0..r]0..=99` carries the statement bound.

## Checklist failures

- None beyond the reported gaps.

## Owed rewrites

- None.

## Language gaps

- `best.get(c - 1) ?? 0` stands in 0 for a missing parent (safe only because values are non-negative), and the rows are read one at a time into a rebuilt `var`: both in research/frictions/ioi94p1.md.

## Dream notes

- None: the ragged read and the fold say the whole solution in two lines.
