# yac6p5 — No More Coyotes

## Look direction as `0..=3` with bit tests
- **Where:** `port.cp`, `s.v >= 2` and `s.v == 1 or s.v == 2`.
- **What felt wrong:** the quadrant is a pair of axis directions; an enum read from `0..=3` with `.x_down` / `.y_down` would name them.
- **General fix:** reading an `enum` from its ordinal (`read<Quadrant>()`) with methods on it.
- **Goal it hurts:** express.
- **Severity:** low.

## Dream

The port, with `enum Quadrant(up_right, down_right, down_left, up_left)` read by ordinal and `axis(flock.map(s => (s.x, s.v.looks_left)))`.
