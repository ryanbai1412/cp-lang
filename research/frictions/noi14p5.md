# Friction report: noi14p5

spec.cp used `board.inverse()` on a `[Grid]Value` (Grid a product domain),
domain `.items` as `(ordinal, position)` and `r.next..`; the first is a
checker gap, the other two were stale (`.items` of a domain gives keys,
R04a.9.7; `r.next` is `?Row`). It now keys the board by a flat
`Cell = 1..=n * m`, which is also what the swap queries name, reads the
queries as `Cell`s, and walks rows with `r.before` / `r.after`.

## `inverse()` of a product-keyed array is rejected

- **Where:** `let at = board.inverse()` with `board: [Grid]Value`,
  `domain Grid = (Row, Col)`.
- **What felt wrong:** R04.2.11 makes `[P]T` and `[A, B]T` the same type
  and `inverse<I, V>(self: [I]V) -> [V]I` takes any key, but `cpl check`
  says E0302 "`inverse()` requires elements of an index domain or bounded
  range, not `Value`", blaming the elements.
- **Goal it hurts:** coherence, express.
- **Ideal code:** `let at = board.inverse()  // [Value]Grid`, then
  `let (r, col) = at[v]`.
- **General fix:** `inverse()` accepts any domain key, product domains
  included (one `I` for the whole key tuple), and the diagnostic names the
  part that is actually unsupported. Routed to compiler.
- **Severity:** low (a flat `Cell` domain plus `Grid.nth(cell.ord)` works).

Measurements: spec.cp agrees with the AC submission (3601452) on the 8
tests and 60 generated inputs (`gen.py`, n, m up to 300), plus a
932 x 520 grid (0.19 s in `cpl run`).
