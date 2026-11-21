# cco20p5

Score: 3/5
Dream: 3/5

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

## Cold review (HEAD)

Dream (new) 3/5, cold-reviewed on current main; sent back. The structure reads like the editorial: `enter`/`answer_at`/`leave` over a `TimeSegTree<Q, Span>`, with two `RollbackFenwick<Coord, Max>` (lines 57-104). The `TimeSegTree`, `RollbackFenwick` and `Max` errors are reported in research/frictions/cco20p5.md.
- Owed now: lines 81-82 pass `fwd`/`mir` to `insert_pair`, which mutates its argument (E0402). Write `insert_pair(mut fwd, …)` / `insert_pair(mut mir, …)`; with that fix it's a 4.
- Minor: `better_min_r`/`better_max_l` (lines 66-76) are one keyed `min`, written twice. `answer_at` compares `(inter, hull)` lexicographically by hand (lines 93, 96). Line 51's comma guard should be `and`: `if let … and cond` / `while let … and cond` is the guard spelling (decisions 4.9).

Port 3/5 (new score): blocked by the reported gaps. It hand-writes the segment tree over time on raw `o * 2` node ids (`seg: [0..4 * q + 5]`, lines 42-50, 101-131) and the rollback Fenwick with an undo log (lines 68-84). Owed now: lines 57-59 test `contains` and then assign, where the dream's `active.entry(key).push(i)` already checks. `dfs` inlines the whole `answer_at` body (lines 110-124).

## Cold review (HEAD)

Port 3/5 and dream 3/5, rescored cold on current main; the dream goes back. The dream's `mut fwd`/`mut mir` (lines 75-76) now pass, `pick_min` is shared, and `answer_at` is one function that both files use. Its remaining errors are the reported `TimeSegTree`, `RollbackFenwick` and `Max` (friction items 1-2), plus one more:
- **Owed (dream):** line 62 queries `bit.prefix(l.r)`, but `l` is a `Coord`, which has no `.r`. The disjoint test needs the largest stored left end over right ends `<= l`, so write `bit.prefix(l)`. That fix makes the dream a 4.
- Line 48 `active.entry(key).push(i)` has no `entry` method on `HashMap` (E0203 in `cpl run`). I added it as friction item 3.
- Port: lines 42-50 and 68-85 hand-write the reported time segment tree (raw `o * 2` node ids, `0..q` as `Int`) and the rollback Fenwick (`bits[side, i]`, an `Undo` log). Line 58 builds the default list by hand because `entry` is missing.
