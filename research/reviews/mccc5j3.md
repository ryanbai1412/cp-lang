# mccc5j3

Score: 4/5
Dream: 4/5

Cold review: distinctness makes every good pair unique, so the count is a `Map` lookup per j (lines 16-20); the header proves it.

## Checklist failures

- None beyond the reported gaps.

## Owed rewrites

- `good` (lines 18-19) can be `at.get(m - b[j]) is Some(i) and abs(Int(i) - Int(j)) >= k`: `is Some(i) and` in a value position compiles today and is canonical (decision §111); the friction lists it as a gap.

## Language gaps

- The hand-built inverse `Map` and `abs(Int(i) - Int(j))` for a distance: both in research/frictions/mccc5j3.md.

## Dream notes

- None.
