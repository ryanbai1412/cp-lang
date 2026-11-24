# Friction Report: ioi00p4

## Dream

```cp
let walls = Graph.undirected(Region, sides_shared_by_two_regions)
let hops = walls.all_pairs_bfs()
let meet = Region.argmin(r => homes.sum(t => touching[t].min(h => hops[r, h])))
```

## Gaps

- **Region adjacency is built by hand.** Pairing the two regions that name a
  wall takes a `Map` from wall to first region; a "group by key, then pair"
  helper would build the graph directly. Severity: low.
- **Unit-weight all-pairs distances go through `floyd_warshall`.** O(M^3) is
  fine at M = 200, but the `?Int` matrix is unwrapped with `!` because the
  regions of a plane graph are connected, and the types can't show that.
  Severity: low.
