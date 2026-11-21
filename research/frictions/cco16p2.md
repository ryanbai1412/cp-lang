# cco16p2 — Splitting Hares

The port pivots around each weighted point and sorts the other points by polar
angle in the upper half-plane. Adopting `std.geometry.{ByAngle, turn}` and `Pt`
removed the local `Vec` alias, local `cross` helper, and `Pt(s.dir.x, s.dir.y)`
adapter while making the collinearity check `I128`-safe.

## `Pt` cannot carry coordinate-range bounds on `read()`
- **Severity:** low
- **Goal it hurts:** express
- **Where:** `proto/ports/cco16p2.cp`, lines 9–11:
  ```cp
  type Spot = (x: -1e6..=1e6, y: -1e6..=1e6, w: -1e5..=1e5)
  fn pos(s: Spot) -> Pt = Pt(s.x, s.y)
  ```
- **What felt wrong:** `Spot` declares `-1e6..=1e6` coordinate bounds for input
  validation and generator bounds, so it cannot store a `Pt` field directly and
  needs a `pos(s) -> Pt` helper. Parameterized or ranged points (e.g.
  `Pt<-1e6..=1e6>`) would let `Spot = (p: Pt<-1e6..=1e6>, w: -1e5..=1e5)` carry
  both the geometric type and its overflow/range proof.
