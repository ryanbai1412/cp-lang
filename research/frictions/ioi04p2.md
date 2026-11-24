# ioi04p2: Hermes

## Dream

```cp
var on_x: [Coord]?Int = None
...
for (x, y) in lights:
    let turn_to_y = on_x.min_by((v, w) => w + dist(v, y))
    on_x += dx  // an elementwise add that skips None
    ...
```

## Distances between coordinates go through `.ord`
- **Where:** `abs(x.ord - px.ord)` and `abs(v.ord - y.ord)`: subtracting two
  values of one domain isn't spelled directly (shared with ioi14p1).
- **Severity:** low.

## Shifting every present entry of an optional array is a loop (fixed)
- **Status:** fixed. The port now keeps plain `[Coord]Int` arrays seeded with
  `abs(v.label)` (port lines 20-21), so no entry is absent. It updates them with a
  `for v in Coord:` loop (lines 26-28), but rebinding with the builder,
  `on_x = [Coord] v => min(on_x[v] + dx, ay + abs(v.ord - py.ord))`, runs today:
  a fresh `cpl run` of the port with both lines rewritten gives the same output on
  tests.toml. That leaves a port rewrite, not a language gap.
- **Severity:** low.
