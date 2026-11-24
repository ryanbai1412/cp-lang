# Friction Report: ioi04p2hard

## Dream

```cp
var x_below = Staircase<Int, Int>(min)   // prefix-min map: keep only keys that beat all smaller
dx min= y_below.upto(prev.x) + prev.x - walk_y[i]
```

## Gaps

- **No prefix-min map in std.** The four "best value over keys <= x" maps are
  hand-kept staircases (`insert`/`upto` over a `Map`). A min `SegTree` over
  `compress`ed coordinates does the same job, but at n = 1e6 it ran 2.0 s
  against 0.54 s for the staircase, so the port keeps the staircase.
  Severity: medium.
- **Stop 0 (the start) is glued onto the read stops.** `stop: [0..=n]` is
  built from `junction` with an `if i == 0`. Severity: low.
