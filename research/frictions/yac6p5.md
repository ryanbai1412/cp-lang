# yac6p5 — No More Coyotes

## Look direction as `0..=3` with bit tests
- **Where:** `port.cp`, `s.v >= 2` and `s.v == 1 or s.v == 2`.
- **What felt wrong:** the quadrant is a pair of axis directions; an enum read from `0..=3` with `.x_down` / `.y_down` would name them.
- **General fix:** reading an `enum` from its ordinal (`read<Quadrant>()`) with methods on it.
- **Goal it hurts:** express.
- **Severity:** low.
- **Status (2026-11-24):** still open. `let q: Quadrant = read()` is E0302 "reading a `Quadrant` needs to know how its variants are written in the input". `Quadrant.at(read<0..=3>())` checks as `?Quadrant`, but `Quadrant.at(read<0..=3>())!` is E0302 "`!` takes the value out of an optional, but this is a `Quadrant`": the diagnostic contradicts the type `at` returns (sent to Core).

## Dream

The port, with `enum Quadrant(up_right, down_right, down_left, up_left)` read by ordinal and `axis(flock.map(s => (s.x, s.v.looks_left)))`.
