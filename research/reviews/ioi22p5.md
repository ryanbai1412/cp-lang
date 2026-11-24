# ioi22p5

Score: 3/5
Dream: 3/5

Cold review: `min_cardinality` is the editorial (one of each type by `fill(1)`, then a binary search on t with keep-or-undo), but the search's predicate moves insects, and the port keeps a machine simulator because the judge half of the interface can't be supplied.

## Checklist failures

- R7: `last_true(holds)` (line 78) runs a predicate that moves insects in and out of the machine and edits `inside`; the search's correctness rests on `last_true`'s probe order, which nothing in the call shows.
- R4: `move_inside` and `move_outside` (lines 24-46) are one update written twice; dream.cp already merges them into `move(i, into:)`.

## Owed rewrites

- Merge the two moves as dream.cp does (dream.cp lines 22-36).
- Write the search as an explicit `lo..hi` loop that keeps or undoes each fill, so the state change is in a statement, not in a predicate.

## Language gaps

- `@grader fn` with a local driver emitting no `main`, and no local stub for the judge's `move_inside`/`press_button`: both in research/frictions/ioi22p5.md.

## Dream notes

- dream.cp has the same `last_true(holds)` with a mutating `holds` (dream.cp line 66): a hidden side effect, so the dream caps at 3. The friction has no `## Dream` section; the dream should be the contestant's `min_cardinality` alone against judge-supplied moves, with the search's keep/undo explicit.
