# Friction Report: ccc23s3

## A transpose is a coordinate swap at every write (not a gap: `transposed()` exists)
- **Where:** `flip`, `(lines, len, pal, cross)` and `if flip: letter(j.ord, i.ord) else: letter(i.ord, j.ord)`.
- **What felt wrong:** half the cases are the transpose of the other half, and the port builds the
  poster from a cell function on plain `Int` coordinates and swaps them. That is an owed rewrite,
  not a language gap: std's `transposed()` (`std/grid.cp`) turns a `[Col, Row]` array into a
  `[Row, Col]` one. `cpl run` of
  `let g: [Col, Row]Char = [Col, Row] (c, r) => ...; let p: [Row, Col]Char = g.transposed()`
  compiles and prints the transposed grid.
- **Goal it hurts:** types carry the proofs (the swapped coordinates lose their domains).
- **Ideal code:** `let poster = if flip: frame<Col, Row>(c, r).transposed() else: frame<Row, Col>(r, c)`.
- **General fix:** none in std; rewrite the port over domain-typed `frame<A, B>`.
- **Severity:** low

## Dream

```cp
fn frame<A, B>(pal: Int, cross: Int) -> ?[A, B]Char  // all A-lines or none palindromic
let poster = if r > 0 and r < n and c > 0 and c < m:
                 Some([Row, Col] (i, j) => if i.ord >= r and j.ord >= c: 'b' else: 'a')
             elif r != 0 and (c == 0 or r != n): frame<Col, Row>(c, r).map(p => p.transposed())
             else: frame<Row, Col>(r, c)
if let p = poster: print(p) else: print("IMPOSSIBLE")
```

## Measurements

- Tokens: port 387, original 1549
- Max input (`gen.py --max`): port 0.02 s, 6784 KB, original 0.08 s, 11264 KB
- Emitted C++: 11903 bytes
