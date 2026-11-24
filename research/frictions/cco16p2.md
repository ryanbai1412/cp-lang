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

## Re-port (v0.3)
The read-time `gen: crowded` generator is gone (gen.py owns inputs), the coordinate bound is one
`Coord` alias, and the answer is `max(0, max(best_through(h) for h in Hare))`.

## Dream
```cp
use std.geometry.{ByAngle}
let n: 2..=4000 = read()
domain Hare = 0..n
let hares: [Hare](p: Pt<-1e6..=1e6>, w: -1e4..=1e4) = read()
fn best_through(pivot: Hare) -> Int:
    let lines = [h for h in Hare if h != pivot].runs_by_angle(around: hares[pivot].p, folded: true)
    hares[pivot].w + lines.sweep_halfplanes(weight: h => hares[h].w).max()!
print(max(0, max(best_through(h) for h in Hare)))
```

- Tokens: port 401, original 972 (a WA submission; none is accepted)
- Max input (`gen.py --max`): port 2.22 s, 3412 KB, original 3.47 s, 3900 KB (limit 7 s)
