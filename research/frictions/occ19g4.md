# Friction Report: occ19g4

## Dream

```cp
let order = Goose.sort(by: g => desc(radius[g]))
// count, for every goose, the larger-radius points in its square and the
// smaller-radius squares over it: two offline dominance counts
let inside = order.cdq(points: g => (u(g), y[g]), boxes: g => square(g))
```

## Gaps

- **CDQ divide and conquer is written by hand.** Both passes repeat the
  split, the sort by `u` and the Fenwick sweep with undo; a std offline
  dominance-count helper would take the points and boxes. Severity: medium.
- **No accepted C++.** Both DMOJ submissions are TLE, so the package has no
  reference to write expected outputs from; the port is checked against the
  old port and the TLE C++ on generated inputs. Severity: medium.
