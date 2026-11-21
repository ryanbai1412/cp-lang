# dpz — Frog 3

Monotone-slope convex hull trick over a `Deque<Pt>`. Adopting
`std.geometry.{dot, turn}` and `Pt` removed the local `Line` type alias, `at`
helper, and hand-rolled `useless` cross-product predicate, and upgraded the hull
orientation check from 64-bit `Int` multiplication (which sits near `1e18` for
`h <= 1e6, c <= 1e12`) to exact `I128` via `turn`.

## No `dot128` / `Line` slope-intercept wrapper needed, but CHT queries use `Pt(x, 1)`
- **Severity:** low
- **Goal it hurts:** express
- **Where:** `proto/ports/dpz.cp`, lines 13–17:
  ```cp
  let q = Pt(x, 1)
  while hull.len >= 2 and dot(hull[1], q) <= dot(hull[0], q): hull.pop_front()
  ...
  let line = Pt(-2 * x, cost + x * x)
  while hull.len >= 2 and turn(hull[hull.len - 2], hull.last!, line) >= 0: hull.pop()
  ```
- **What felt wrong:** representing a line `k * x + b` as `Pt(k, b)` and
  evaluating with `dot(line, Pt(x, 1))` is concise and reuses exact `I128`
  `turn`, though a dedicated `std.dp` / CHT line container (e.g. `LiChao` or
  `LineHull`) would read even more directly for non-monotone queries.
