# noi23p4

Score: 3/5
Dream: none (no dream.cp and no `## Dream` in the friction report)

Cold review on main 6a989896d: port, dream and friction report, not the C++. There is no dream (neither dream.cp nor a `## Dream` in the friction report), so the score is capped at 3 (research/reviews/README.md). The shifted-distance DFS with a `RollbackArray` and a Dijkstra seeded at each subtree root is the header's argument.

## Checklist failures

- Lines 35-45 write a lazy-deletion Dijkstra by hand over a `MinHeap`; std `dijkstra` (std/graph.cp) owns its own `[V]?W` result, so it cannot improve a caller's `RollbackArray` in place. A hand-rolled std algorithm with the gap unreported (idiomatic-code/R3).
- Line 33 `var (sum, reached) = (sum, reached)` shadows both parameters to make them mutable.
- Line 49 finds the children as `2 * x.ord + 1..min(2 * x.ord + 3, City.count)`: heap-shape `ord` arithmetic where `parent` (line 23) already names the inverse (idiomatic-code/R6).

## Owed rewrites

- Write the dream.
- Report the Dijkstra-over-a-caller's-store gap in the friction with a run of the std call it wants, or name why the hand loop stays.

## Language gaps

- None.
