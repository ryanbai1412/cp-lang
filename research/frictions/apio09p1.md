# apio09p1

Ported from `corpus/submissions/5536569.cpp`; `orig.cpp` is that submission with its out-of-bounds
read fixed, and it is the reference. No `dream.cp`; the dream is below.

## Dream
`port.cp`, except that a square off the grid is absent instead of 0:
```cp
fn square(x: Int, y: Int) -> ?Haul:
    let r = Row.at(x)? ; let c = Col.at(y)?
    let r2 = Row.at(x + k - 1)? ; let c2 = Col.at(y + k - 1)?
    sums.fold(r..=r2, c..=c2)
```
and `square(...) ?? 0` at the four corner tables, which is where "no square fits" means 0.

## "No square" is spelled 0
- **Where:** `port.cp` `fn square`: `if x < 0 or y < 0 or x > n - k or y > m - k: return 0`.
- **What felt wrong:** a square that does not fit is a sentinel 0. The comment ("no square is
  negative") explains why 0 is safe for `max`, but the reader has to check it.
- **Goal it hurts:** absent values are types (idiomatic-code/R5).
- **Ideal code:** the dream's `?Haul` with `Row.at`/`Col.at` (both exist today), and `?? 0` at
  the use.
- **General fix:** a port rewrite.
- **Severity:** low.
- **Owner:** idioms (this port); lands with the next port-edit commit.

## The read table is stored as 64-bit
- **Where:** emitted C++ for `let oil: [Row, Col]0..=500 = read()`: `cpl::Grid<ll, 2> oil;`,
  while the `var ul: [RowCut, ColCut]Haul` tables right below become `cpl::Grid<uint32_t, 2>`.
- **What felt wrong:** a `var` table with a range element type gets the narrow storage its range
  allows, but a `read()` table with a range element does not. At n = m = 1500 that is 18 MB
  where 4.5 MB would do (`uint16_t`). The port peaks at 71.5 MB against `orig.cpp`'s 37 MB.
- **Goal it hurts:** zero-cost types (handbook/principles.md); R10 (memory against the C++).
- **Ideal code:** unchanged: `let oil: [Row, Col]0..=500 = read()` should store as `uint16_t`.
- **General fix:** codegen narrows the storage of a read table by its element range, as it
  already does for `var` tables.
- **Severity:** medium (the limit is 128 MB; a tighter memory limit would fail).
- **Owner:** Core (codegen). Repro: `cpl build corpus/problems/apio09p1/port.cp -o a.cpp; grep
  'Grid<' a.cpp`.

## Measurements
- Tokens: port 767, original 1,124.
- `--max` (1500 x 1500, median of 15): port 116.8 ms / 71.5 MB, `orig.cpp` 182.2 ms / 37.0 MB
  (0.64x time, 1.93x memory). Same output.
- Emitted C++: 30,711 bytes.
