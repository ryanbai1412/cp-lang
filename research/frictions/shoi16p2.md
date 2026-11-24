# Friction report: shoi16p2

There is no dream.cp; the ideal is below. The port uses inclusion-exclusion. It counts every square through its i x i box, subtracts the squares through each deleted point with the four-sided `corner_squares` closed form, and adds back a 2/c share at every pair of deleted corners.

## Dream

```cp
type Point = (X, Y)
let gone: [Gone]Point = read()
fn corner(p: Point, d: (Int, Int)) -> ?Point = p + d      // off the lattice is None
```

## Off-lattice corners go through `Int` pairs and a hand `point()`
- **Where:**
  - line 19: `gone` is read as `(x: 0..=n, y: 0..=m)` `Int`s even though the `X` and `Y` domains exist;
  - lines 29-30: `point(x, y)` checks both coordinates with `X.at`/`Y.at`;
  - line 47: each corner is spelled `x + dx - dy, y + dx + dy` and the like.
- **What felt wrong:** two things:
  - the points are domain values only after a lookup;
  - "the point at offset d, if it is on the lattice" is written once per call site as `Int` arithmetic.
- **Goal it hurts:** express.
- **Ideal code:** the dream's `Point + (dx, dy) -> ?Point` on a product domain.
- **General fix:** a checked offset on product-domain values that returns `None` off the grid, the 2-D form of `offset`. Owner: stdlib.
- **Severity:** low.

## Measurements

- **Tokens:** port 620, accepted C++ 970 (comments excluded).
- **Runtime and memory on gen.py --max:** port 0.46 s / 3 MB, accepted C++ 1.09 s / 67 MB, with the same output.
- **Emitted C++:** 29447 bytes.
- **Agreement:** `cpl stress` against the accepted C++ agrees on 30 gen.py cases at size 30.
