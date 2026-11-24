# coci20c6p2

Score: 4/5
Dream: 4/5

Cold review against the friction's `## Dream`. Words get vertices from an `Interner` (lines 14-17), the arcs fill an adjacency table (lines 19-22), and each query is one `dijkstra` with "Roger" for an unreached word (lines 25-29), as the header says.

## Checklist failures

- None beyond the reported gaps.

## Owed rewrites

- None.

## Language gaps

- A graph built from named endpoints while reading (the `var out` table, line 19): in the friction.
- `print(d ?? "Roger")` across `Int` and `Str` (the `if let`, lines 28-29): in the friction.

## Dream notes

- Two lines (`Graph.directed`, `dijkstra(...)[b] ?? "Roger"`), judged as the port with them; reads like the editorial.
