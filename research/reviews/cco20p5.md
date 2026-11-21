# cco20p5

Score: 2/5

Baseline review, cold, on main 96359635. The header (lines 4-8) names the plan: an offline segment tree over time for disjoint pairs, and heaps for overlapping pairs. Both structures are hand-rolled with node arithmetic, a rollback Fenwick written as `i & -i` loops, and negated heap keys. No dream exists, so the score is capped at 3.

## Checklist failures

- Hand segment tree over time: `[[] ; 4 * q]`, `o * 2`, `(l + r) / 2` (lines 24-32, 81-93).
- Hand Fenwick with rollback logging (lines 45-74); `i -= i & -i` and `j += j & -j` (lines 58, 65, 73).
- `INF` sentinels (lines 76-79, 123) where `?Int` works.
- `kind: Char` compared to `'A'` (lines 15, 36, 105); use an enum.
- Unbounded `l`/`r` (lines 16-17) beside the magic `MAXC = 1_000_001` (line 12); the statement bound belongs in the type.
- Negated keys `-e.l` for max order in the heaps (lines 98-119) where `MaxHeap` (`stdlib/heap.cp:77`) says it.
- Lazy deletion by hand with a `dead` array (lines 101, 110-113); `ErasableMinHeap`/`ErasableMaxHeap` (`stdlib/heap.cp:112, 156`) exist.

## Rewrites owed with today's language

- `MaxHeap`/`ErasableMaxHeap` instead of negated keys and the `dead` array; an enum for the event kind; typed coordinates.
- `Fenwick<Coord, Max>` for the prefix max, if rollback can be layered on it.

## Gaps that need the language

- Offline segment tree over time (add over intervals, then DFS with rollback) and a rollback-able Fenwick or undo log. The stdlib doesn't have them, and no friction report exists; flag it.
