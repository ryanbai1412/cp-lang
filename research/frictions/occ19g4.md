# Friction Report: occ19g4

## Dream

```cp
let rank = Goose.sort(by: g => desc(radius[g]))
// one offline pass: for every goose, the earlier-ranked points in its square
// and the later-ranked squares over it, as two dominance counts
let fights = rank.cdq(points: g => (u[g], y[g]), boxes: g => square(g))
```

## Gaps

- **CDQ divide and conquer is written by hand.** The port keeps the three u
  orders sorted once and splits them stably with `partition` down the
  recursion, then runs four Fenwick sweeps with undo per split. A std
  offline dominance-count helper would take the points and boxes and own the
  splitting, the sweeps and the undo. Severity: medium.
- **No accepted C++.** Both DMOJ submissions are TLE, so `orig.cpp` is an
  O(N^2) brute force written for the package; it is the reference for the
  expected outputs, and `--max` was checked against the previous port.
  Severity: low.
