# ccc23s4

Score: 4/5
Dream: 2/5

Cold review of `port.cp` and the dream (`## Dream` of the friction report, or `dream.cp`) against `handbook/guidelines/idiomatic-code-checklist.md`.

## Checklist failures

- R1: header derives the sorted greedy with a bounded Dijkstra.
- R2: roads by (length, cost); keep one unless the kept graph already joins its ends within its length: the editorial.
- R3: the growing graph is an adjacency list with two `push`es (L24-25), reported.
- R10: 0.01 s vs 0.12 s.

## Owed rewrites

None.

## Language gaps

- No incremental `Graph.add` (in the report).
- No `is_some_and` on optionals (in the report).

## Dream notes

`... and kept.add(e, roads[e])` adds edges inside the `sum` filter (R7), and `roads[e]` is repeated six times (R4); a `for` loop with `let (u, v, len, cost) = roads[e]` is the dream.
