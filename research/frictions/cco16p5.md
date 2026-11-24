# Friction Report: cco16p5

The v0.2 port hand-rolled a cover-count segment tree on `2p`/`2p + 1` node indices with global
`ys`/`cnt`/`cov` lists. K <= 2000 makes O(K^2) per area fast (0.01 s at the max), so the port now
cuts the columns at square edges and measures each strip's covered rows in one pass over the
squares sorted by top row: no tree, no node indices.

## Union length of intervals is spelled by hand
- **Where:** the `(rows, reach)` loop inside `covered`.
- **What felt wrong:** "the length of the union of these sorted intervals" is a std-shaped fold,
  written as a running pair.
- **Ideal:** `union_length(ranges)` in `std.interval` (or `SparseIntervalSet.of(ranges).covered`
  without per-insert logs).

## No Klee's measure in std
- **Ideal:** `union_area(rects)` (cover-count segment tree sweep) in `std.geometry`, so the whole
  program is `print(union_area(squares(q)) - union_area(squares(q - 1)))`.

## Dream
```cp
use std.geometry.{union_area}
let n: 1..=1e9 = read()
let m: 1..=1e9 = read()
let k: 1..=2000 = read()
let zombies: [k](r: 1..=n, c: 1..=m) = read()
let q: 0..=n + m = read()
fn within(rad: Int) -> Int =
    if rad < 0: 0 else: union_area([(max(1, z.r - rad)..min(n, z.r + rad) + 1,
                                     max(1, z.c - rad)..min(m, z.c + rad) + 1) for z in zombies])
print(within(q) - within(q - 1))
```

- Tokens: port 308, original 2081
- Max input (`gen.py --max`): port 0.01 s, 3328 KB, original 0.23 s, 4096 KB
