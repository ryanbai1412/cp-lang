# Friction Report: ioi07p2

## Dream

```cp
enum Dir = East | North | West | South
fn turn(d: Dir, k: Int) -> Dir = d.wrap(k)          // cyclic step on an enum
let faces = PlanarFaces(pts, walls)                  // faces of a planar straight-line graph
```

## Gaps

- **Enums have no cyclic step.** `Dir.wrap` exists on domains but not on an
  enum, so a quarter turn is `Dir.nth((d.ord + k) % 4)`. Severity: low.
  Fresh run (`cpl run`):
  ```
  enum Dir = East | North | West | South
  let d = North
  print(d.wrap(3).ord)
  ```
  ```
  error[E0203]: Dir has no field or method `wrap`
  ```
  General fix: give enums the domain stepping methods (`wrap`, `next`, `prev`); owner: stdlib.
- **Faces of a planar graph are built by hand.** The DSU over wall sides,
  joined round each point by the next wall counter-clockwise, is the standard
  face walk; std has no planar-face helper. Severity: medium.
