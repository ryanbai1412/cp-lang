# ucup4s25e friction report

## Missing stdlib: `best_dot` on a monotone upper/lower hull in `std.geometry`
- **Where:** `proto/ports/ecnerwala/dream/ucup4s25e.cp` line 40 (`proto/ports/ecnerwala/ucup4s25e.cp` lines 37–43).
- **What felt wrong:** `upper_hull` returns a left-to-right (CW) chain, while `hull_extreme` requires a CCW convex polygon and returns an index, forcing the caller to `.reverse()` the hull, bind `dir = Pt(v, 1)` to avoid duplicating it, index back into `hull`, and call `dot(hull[idx], dir)`.
- **Goal it hurts:** express.
- **Ideal code:**
  ```cp
  let hull = upper_hull([Pt(i + 1, y) for (i, v) in best.items if let y = v])
  print([best_dot(hull, Pt(v, 1)) for v in qs])
  ```
- **General fix:** Add `best_dot(hull, dir)` (or a `best_dot` helper accepting a left-to-right upper/lower hull or CCW polygon) to `std.geometry` that returns the maximum `dot(p, dir)` directly via binary search on slopes, without requiring the caller to reverse `upper_hull` output into CCW order.
- **Severity:** low.
