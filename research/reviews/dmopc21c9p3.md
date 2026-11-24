# dmopc21c9p3

Score: 3/5
Dream: none (no dream.cp; the ideal is in the friction report)

Scored cold against the checklist and the port. With no dream.cp the port scores at most 3 (README); against the checklist alone it reads as a 4.

All-pairs BFS distances, then each query checks either trip alone or the two meeting-point orders.

## Checklist failures

- `dist` is filled by a nested loop over `bfs(...).iter()`; a table comprehension would make it a `let`.
- `x: Int` and `y: Int` in the query read have no bound.
