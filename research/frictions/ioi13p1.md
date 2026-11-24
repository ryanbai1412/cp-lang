# ioi13p1: Dreaming

## Dream

```cp
let g = Graph.undirected(trails).weighted(a => trails[a.e].t)
let (domain Tree, comp) = components(g)
let ecc = g.eccentricities()  // per vertex, within its tree, O(n)
let radius = [Tree] _ => Int.MAX; for v in V: radius[comp[v]] min= ecc[v]
...
```

## Per-tree eccentricities are three hand-written walks
- **Where:** `spread`, `farthest` and the `reach`/`seen` loop: the
  two-sweep eccentricity of every vertex of a forest. A std forest
  `eccentricities` (or `diameter`/`radius` per component) would replace them;
  a per-component `dijkstra` costs O(n) per tree, so it can't be used here.
- **Severity:** medium.

## The judge's function task has no CP Lang form
- **Where:** `orig.cpp` wraps submission 4687433 calling `travelTime`.
- **Severity:** high.
