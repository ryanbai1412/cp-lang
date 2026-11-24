# sac22cc4jp3 — closest direction

## No `%` or `rem_euclid` on F64
- **Where:** `port.cp`, `gap`.
- **What felt wrong:** reducing an angle into `0..360` is `(d - aim).rem_euclid(360.0)`; F64 has
  neither, so the port folds `abs(d - aim)` by hand, using that atan2 is in `-180..=180`.
- **General fix:** `F64.rem_euclid` (and `%`) in the prelude.
- **Goal it hurts:** express.
- **Severity:** low.

## Dream

```cp
let points = read<[read<1..=1e5>()](x: -1e9..=1e9, y: -1e9..=1e9)>()   // after the aim
fn gap(d: F64) -> F64 = let t = (d - aim).rem_euclid(360.0) in min(t, 360 - t)
print(points.argmin(by: p => gap(degrees(atan2(p.y, p.x))))!)
```
