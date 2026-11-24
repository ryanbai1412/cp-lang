# Friction report: dmopc19c4p2

The port is std `bfs` over a product domain `State = (marble: Cell, hole: Cell)` with a
generator of moves. One gap: reading a product-domain value.

## `read<Cell>()` for a product domain reads one flat index

- **Where:** `let marble = Cell(read<(Row, Col)>())` (and goal, empty cells)
- **What felt wrong:** with `domain Cell = (Row, Col)`, `let c: Cell = read()` reads a single
  integer as the flat index, not a row and a column. The input `1 2` (row 1, column 2) became
  the cell numbered 1, so the first version silently answered 0 instead of 82.
- **Goal it hurts:** correct (silent misread)
- **Ideal code:** `let marble: Cell = read()`
- **Repro (fresh build):**
  ```
  domain R = 1..=3
  domain C = 1..=3
  domain Cell = (R, C)
  let c: Cell = read()
  print(c)
  ```
  with input `2 3` prints `1 3`, and leaves `3` unread; reading `(R, C)` prints `2 3`.
- **General fix:** io: a product domain reads and prints as its factors, like the tuple it names.
- **Severity:** high (wrong answer, no diagnostic)

## Measurements

gen.py --max (n = 15): port and accepted C++ both under 0.01 s.

## Dream

Fixed by Core in 50ed27f43; the port now reads `let marble: Cell = read()` and `let empty: [2]Cell = read()` and is the dream.
