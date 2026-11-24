# Friction Report: noip10p4

## 1. The span of a cell is a fold the port writes
- **Where:** `type Span`, `widen`.
- **What felt wrong:** "the union of child intervals" is an interval-hull monoid; std has no `Span` with an
  empty value.
- **Goal it hurts:** express.
- **Ideal code:** `span[r, c] += span[r2, c2]` on `Hull<C>`.
- **General fix:** an interval-hull monoid (`Hull<D>`: empty, `+`, `lo`, `hi`).
- **Severity:** low.

## 2. Interval cover is a std idea
- **Where:** the greedy `while covered < m` loop.
- **What felt wrong:** "fewest intervals covering 0..m" is a classic; the port writes the O(m^2) greedy.
- **Ideal code:** `min_cover(spans, 0..m)!`.
- **General fix:** `min_cover` in `std.interval`.
- **Severity:** low.

## Measurements
Stress: 30/30 agree with the v0.2 port and with AC 5205786; max (500 x 500 descending snake) 0.011 s vs C++ 1.13 s, same output (both with an unlimited stack).

## Dream
The ungated ideal (does not check today):

```cp
var span: [R, C]Hull<C> = Hull.empty
@memo
fn flow(r: R, c: C) -> Hull<C> =
    (if r == R.last!: Hull.of(c) else: Hull.empty) + sum(flow(q) for q in (r, c).steps(Dir4) if h[q] < h[r, c])
let tops = [flow(R.first!, c) for c in C]
if let k = min_cover(tops, C): print(1); print(k)
else: print(0); print(count(c not in union(tops) for c in C))
```
