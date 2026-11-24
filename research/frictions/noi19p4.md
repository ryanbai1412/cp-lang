# Friction Report: noi19p4

## 1. Rectangle drain needed a std structure (fixed)
- **Where:** `unreached.drain(Int(r.xl)..Int(r.xr) + 1, r.yl..r.yr + 1)`.
- **What felt wrong:** the earlier port hand-rolled an x-segment tree of `SortedSet<(Int, City)>` to
  remove every unreached city in a rectangle once. `std.geometry.PointSet` now does exactly that.
- **Remaining gap:** `PointSet` names points by `Int` index into the build list, so the port converts
  with `City.nth(i)` and the ranges are half-open `Int` ranges over typed bounds.
- **Ideal code:** `PointSet<City>.of(at)` with `drain(r.xl..=r.xr, r.yl..=r.yr) -> yields City`.
- **General fix:** a `PointSet<D>` keyed by a domain, taking inclusive typed ranges.
- **Severity:** low.

## Measurements
Stress: 30/30 agree with AC 3800719 (the v0.2 port does not check today); max (n = 7e4,
m = 1.5e5) 0.51 s vs C++ 0.65 s, same output.

## Dream
The ungated ideal (does not check today):

```cp
var unreached = PointSet<City>.of(at)
for (u, d) in dijkstra(At(City.first!), next: u => match u:
        At(c): out[c].map(p => (Via(p), portal[p].t))
        Via(p): unreached.drain(portal[p].xl..=portal[p].xr, portal[p].yl..=portal[p].yr).map(c => (At(c), 0))):
    if let At(c) = u: dist[c] = d
```
