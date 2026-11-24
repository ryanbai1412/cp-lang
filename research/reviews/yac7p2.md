# yac7p2 review

Score: pending
Dream: pending

Cold review: the old port drove a cursor with go/right/left/down/up through globals and wrote
into a flat list. Now `tour(h, w)` returns the path cells as comprehensions chained with std
`chain`, one per stretch (top row, snake, return column; or top row, side, column snake,
two-column zigzag), and the numbering is `number[r, c] = i + 1` over `tour(...).items`. Reads
carry the statement bounds (T, n, m <= 100). The answer isn't unique, so the package gains a
checker.py that checks the permutation and two coprime neighbours per cell.
