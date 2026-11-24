# Friction Report: noip00p4

## 1. The anti-diagonal walk is index arithmetic
- **Where:** `rows = Row.at(max(1, k - n + 2))!..=Row.at(min(n, k + 1))!` and `col(r) = Row(k + 2 - Int(r))`.
- **What felt wrong:** "the cells of diagonal k" is a grid idea; the port derives the row range and the
  column by hand, and the predecessor pairs as `[a.prev, Some(a)]` products.
- **Goal it hurts:** express.
- **Ideal code:** `for (p, q) in grid.diagonal(k)` and `best[a, b] = max(best[pa, pb] for pa in a.back(0..=1) for pb in b.back(0..=1))`.
- **General fix:** `[R, C]T.diagonal(k)` yielding cells, and optional max over a product of optional steps.
- **Severity:** low.

## Measurements
Stress: 30/30 agree with the v0.2 port and with AC 5067920; max (full 9 x 9 grid): port 845 equals the v0.2 port, while the AC enumerates subsets of the numbers and crashes on a full grid (the judge data are sparse); the committed full-grid test passes.

## Dream
The ungated ideal (does not check today):

```cp
let best = [Row, Row] (a, b) on diagonal k =>
    max(best[pa, pb] for pa in a.back(0..=1) for pb in b.back(0..=1)) + grid.taken(a, b)
print(best[Row.last!, Row.last!])
```
