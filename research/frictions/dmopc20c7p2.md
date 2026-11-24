# Friction report: dmopc20c7p2

## A map of corner -> edges costs 3x a sorted edge list

- **Where:** `edges.sort(by: e => e.from)` and `fn leaving(p) = [edges[i] for i in edges.equal_range(p, by: e => e.from)]`
- **What felt wrong:** the natural spelling, `Map<Pt<Int>, List<Edge>>` filled per tile, ran 1.99 s at the max case against 1.29 s for the accepted C++; sorting the edges once and reading a corner's edges as an equal range runs 0.68 s. The port has to know that the tree map is the slow part.
- **Goal it hurts:** fast (the obvious way is not the fast way)
- **Ideal code:** `let leaving = edges.group_by(e => e.from)` giving a sorted, flat grouping with O(log n) lookup and no per-key list.
- **General fix:** std: a flat `group_by` (sorted keys over one backing list) for build-once multimaps.
- **Severity:** medium

## Measurements

gen.py 1 200000 (N = 2e5, one tall strip): port 0.70 s, the map version 1.99 s, accepted C++ 1.28 s.

## Dream

```
let leaving = edges.group_by(e => e.from)
let start = min(p for (p, out) in leaving if out.len == 1)!
```
