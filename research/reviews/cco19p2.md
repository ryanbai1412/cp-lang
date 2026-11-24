# cco19p2

Score: 2/5
Dream: 3/5

A `Dsu` over `Cell` names the objects, each column gives the floor seeds and the arcs between stacked objects, and std `dijkstra` gives each object's fall. The object model is forced by a real gap, but the `[Rest]` copy works around a limit that does not exist.

## Checklist failures

- idiomatic-code/R13: `domain Rest` and `let arcs: [Rest](...) = [Rest] e => rests[e.ord]` (lines 33-34) copy the list for `Graph.directed`, which takes a `List` today: `dijkstra(Graph.directed(rests), from: ...)` on a `List<(u: V, v: V, w: Int)>` runs under 0.2 and prints the distance. The friction's "needs a typed array" does not hold.

## Owed rewrites

- `let fall = dijkstra(Graph.directed(rests), from: floor)`; drop `Rest` and `arcs`, and the friction's second bullet.

## Language gaps

- `components` rejects a table over a product domain: research/frictions/cco19p2.md, with the `cpl run` repro added by this review.

## Dream notes

- `[q for q in p.adj4 if g[q] == '#' == g[p]]` chains two comparisons where "both are '#'" is meant (idiomatic-code/R2, caps at 3). The dream is three lines with no header.
