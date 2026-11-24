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

## Shifting every present entry of an optional array is a loop
- **Where:** the `for v in Coord: if let w = on_x[v]: ...` pair; an
  elementwise map over present entries would be one line each.
- **Severity:** low.
