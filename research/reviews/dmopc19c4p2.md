# dmopc19c4p2

Score: 4/5
Dream: 4/5 (the port as written, per the friction's `## Dream`)

Scored cold against the checklist and the port.

A BFS over `State = (marble: Cell, hole: Cell)`: moving the hole into the marble swaps them, otherwise the hole moves. The answer is the nearest state with the marble at the goal. Product-domain reads (`let marble: Cell = read()`) work since Core's 50ed27f43.

## Checklist failures

- `let hole = if marble == empty[0]: empty[1] else: empty[0]` picks the other empty cell by index.
- `State((s.hole, s.marble))` constructs a product state through a nested tuple.
