# dmopc20c7p2

Score: 3/5
Dream: none (no dream.cp; the ideal is in the friction report)

Scored cold against the checklist and the port.

Each tile becomes a square or an octagon by its parity; the port collects every directed edge, sorts them by start point, and walks the outer boundary from the lowest corner with one outgoing edge.

## Checklist failures

- `leaving(p)` rebuilds a list from `edges.equal_range` at each step, the friction report's sorted list standing in for a corner map.
- `(corners + [corners[0]]).adjacent` closes the polygon by appending its first corner by hand.
- `out.find(e => e.tile != by) ?? out[0]` falls back to an index when no edge changes tile.
- The boundary walk is a `loop:` with `break` at the start point, mutating `at` and `by` together.

## Gaps that need the language

- A corner-to-edges map costs 3x a sorted edge list: research/frictions/dmopc20c7p2.md.
