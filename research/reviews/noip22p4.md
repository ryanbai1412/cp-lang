# noip22p4

Score: 3/5
Dream: none (no dream.cp; the ideal is in the friction report)

Scored cold against the checklist and the port. With no dream.cp the port scores at most 3 (README); against the checklist alone it reads as a 4.

Cold review: the tag carries ?U64 assignments instead of has_a/ta flag pairs, composition substitutes g's constants into f in two lines, the segment length comes from the Action's len argument instead of a stored field, spans() gives each a[r]'s reign and buckets() groups queries by right end. The history action itself has no std counterpart.

## Checklist failures

- `Tag(set_a, set_b, ab, a, b, one)` has six fields, and most tags leave all but one of them at zero. Every literal tag spells all six with `U64(0)`, and `none = Sums(U64(0), ...)` does the same.
- `then` updates four locals through tuple reassignment `(ab, ka, kb, one) = (...)` in two `if let` branches. That is correct, but the composition law is only readable by expanding it on paper.
- `let (_, n): (0..=25, ...)` reads and drops the test id, which is right; the comment carries why.

## Gaps that need the language

- Tags with mostly zero fields are written out in full: research/frictions/noip22p4.md.
