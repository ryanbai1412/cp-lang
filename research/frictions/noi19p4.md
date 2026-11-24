# Friction Report: noi19p4

## 1. No std structure that drains the points of a rectangle
- **Where:** `cell`, `place`, `harvest`: an x-segment tree of `SortedSet<(Int, City)>`, written by hand.
- **What felt wrong:** the idea is "remove every unreached city in this rectangle"; the port spells the tree's index
  arithmetic and recursion, and `cpl lint` flags it (L0514 hand-rolled recursive segment tree) while its suggested
  `SegTree` cannot hold drainable per-node sets (`walk` yields node values, not places).
- **Goal it hurts:** express, fast (R3, R9).
- **Ideal code:** `var unreached = PointSet.of(at)`, `for c in unreached.drain(xs, ys): ...`.
- **General fix:** a std 2D point set with remove and rectangle drain (segment tree of sorted runs with next-alive
  links, or a k-d tree), amortized O(log^2) per point; also any "segment tree of containers" (`SegTree.walk_mut` or a
  `SegTreeOf<D, C>`) for offline rectangle problems.
- **Severity:** high.

## 2. Ranges as row fields
- **Where:** `xl: X, xr: X, yl: 1..=h, yr: 1..=h`.
- **Ideal code:** `xs: 1..=w ..= 1..=w`: read a pair as a range.
- **General fix:** a range type readable from two numbers.
- **Severity:** low.

## 3. Performance
- **Where:** the treap-backed `SortedSet` per node.
- **What felt wrong:** 2.4x the C++ (static per-node arrays with path-compressed next-alive links).
- **General fix:** the `PointSet` of item 1 over static arrays.
- **Severity:** medium (R10 miss).

## Measurements
Stress: 30/30 agree with AC 3800719 on gen.py inputs (every city reachable). Max input (`gen.py --max`, n = 7e4,
m = 1.5e5): port 1.76 s, C++ 0.72 s, same output. The frozen v0.2 port no longer checks (E0602/E0603 lent
bucket elements), so the AC is the oracle.

## Dream
The ungated ideal (does not check today; the gaps above are the difference):

```cp
let (n, m, w, h) = read<(1..=7e4, 1..=1.5e5, 1..=7e4, 1..=7e4)>()
domain City = 1..=n
let at = read<[City](x: 1..=w, y: 1..=h)>()
let portal = read<[m](from: City, t: 1..=1e4, xs: 1..=w ..= 1..=w, ys: 1..=h ..= 1..=h)>()
let out = portal.map(.from).buckets()

enum Node: At(City) | Via(portal.Index)
var unreached = PointSet.of(at)
var frontier = Settle<Node, Int>()
var dist: [City]Int = 0
frontier.offer(At(City.first!), 0)
while let (u, d) = frontier.pop():
    match u:
        At(c):
            dist[c] = d
            unreached.remove(c)
            for p in out[c]: frontier.offer(Via(p), d + portal[p].t)
        Via(p):
            for c in unreached.drain(portal[p].xs, portal[p].ys): frontier.offer(At(c), d)
print(dist[c] for c in City.drop(1), sep: "\n")
```
