# dmopc21c7p1

Score: 3/5
Dream: none (no dream.cp; the ideal is in the friction report)

Scored cold against the checklist and the port.

Fill each blank from the bottom-right with one less than the smaller of its lower and right neighbours, then check every row and column increases.

## Checklist failures

- `ROOM = 1e17` and `TOP = 1e18` are sentinels standing for past the border; an optional minimum would not need them.
- `var a: [Row, Col]Int = read<[Row, Col]0..=1e9>()` widens the read to fill it in place, so 0 means blank inside an `Int` table.
- `below(p, q)` takes a `?(Row, Col)` and returns `true` for a missing neighbour, and the `fits` call is split across lines by the formatter.
