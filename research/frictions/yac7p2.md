# Friction report: yac7p2

There is no dream.cp; the ideal is below. The port numbers the cells along a fixed tour: a closed snake when a side is even, and a row/column/snake/zigzag path when both sides are odd. It transposes by hand so that the even side is the height.

## Dream

```cp
domain Row = 0..h
domain Col = 0..w
fn tour() -> List<(Row, Col)>: ...
let number = [Row, Col] from tour().items
print(if flip: number.transposed() else: number)
```

## Coordinates are bare `Int`s because the grid may be transposed
- **Where:** `tour(h: Int, w: Int) -> List<(Int, Int)>`, `number: [0..h, 0..w]Int`, and the print line's `if flip: number[j, i] else: number[i, j]`.
- **What felt wrong:** rows and columns are two index spaces (R3), but swapping them forces both to plain integer ranges, and the transpose is an index swap inside the print.
- **Goal it hurts:** hard-to-get-wrong.
- **Ideal code:** `Row`/`Col` domains for the tour, and `number.transposed()` at the end.
- **General fix:** a `transposed()` on 2-D arrays that swaps their index domains. Owner: stdlib.
- **Severity:** low.

## Filling an array from a path is a loop
- **Where:** `for (i, (r, c)) in tour(h, w).items: number[r, c] = i + 1`, over an array first filled with `0`.
- **What felt wrong:** the `0` is a placeholder every cell overwrites.
- **Goal it hurts:** express.
- **Ideal code:** build the array from the tour's positions as its inverse: `tour.inverse_into([Row, Col])`.
- **General fix:** an array built from a permutation listing, matching `List.inverse()` for 1-D. Owner: stdlib.
- **Severity:** low.

## Measurements

- **Tokens:** port 359, accepted C++ 970 (comments excluded).
- **Runtime and memory on gen.py --max:** port 0.05 s / 3 MB, accepted C++ 0.03 s / 3 MB, with the same output.
- **Emitted C++:** 16839 bytes.
- **Agreement:** `cpl stress` against the accepted C++ agrees on 30 gen.py cases at size 30.
