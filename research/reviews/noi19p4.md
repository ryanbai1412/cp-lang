# noi19p4

Score: 4/5
Dream: 4/5

Cold review on main c8a46b201: port, dream and friction report, not the C++. Dijkstra over cities and portals with a std `PointSet` drained per portal; the header proves each city is offered once.

## Checklist failures

- Line 38 drains `Int(r.xl)..Int(r.xr) + 1` and line 39 maps the index back with `City.nth(i)` (reported, friction 1).

## Owed rewrites

- None.

## Language gaps

- `PointSet<D>` keyed by a domain with inclusive typed ranges: friction 1.
- The dream drives the search with `dijkstra(start, next: ...)`; the port hand-drives `Settle`. The friction report does not say whether std `dijkstra` takes such a `next:` closure; a run is owed.

## Dream notes

- The dream is a snippet without the reads or a header.
