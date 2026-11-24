# noi10p3

Score: 3/5
Dream: none

Port 3/5, dream none. The friction report has no `## Dream` section and there is no `dream.cp`, so the port is capped at 3. The minimum cut as a shortest path in the planar dual (header lines 7-11) is a real rethink, and Dijkstra is one line (line 42).

## Checklist failures

- `arc` (lines 32-41) decodes an arc number into (row or column, i, j, direction) by division and parity, so each dual arc's endpoints are found by arithmetic rather than stated.
- `Face` numbers cells by hand (lines 21-23).
- The friction's `Graph.directed` entry quotes `domain Arc = 0..dual.len; Graph.directed([Arc] a => dual[a.ord])`, but the port has no `dual` list; it computes arcs from their numbers.

## Owed rewrites

- Add a `## Dream`; build the four kinds of arc as comprehensions over the flow tables and update the friction's snippet to the port.

## Language gaps

- `Graph.directed` over a `Seq` of edges: in the friction.

## Dream notes

None.
