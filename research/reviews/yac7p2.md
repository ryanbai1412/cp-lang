# yac7p2 review

Score: 3/5
Dream: 3/5

Cold review: the old port drove a cursor with go/right/left/down/up through globals and wrote
into a flat list. Now `tour(h, w)` returns the path cells as comprehensions chained with std
`chain`, one per stretch (top row, snake, return column; or top row, side, column snake,
two-column zigzag), and the numbering is `number[r, c] = i + 1` over `tour(...).items`. Reads
carry the statement bounds (T, n, m <= 100). The answer isn't unique, so the package gains a
checker.py that checks the permutation and two coprime neighbours per cell.

## Cold review (HEAD)

Port 3/5 (no dream.cp), dream 3/5. `tour(h, w)` returns the path as chained comprehensions, one per stretch (lines 15-23), and numbering is `number[r, c] = i + 1` over `.items` (line 35). A checker covers the non-unique answer.
- Lines 31-36: the even side is forced to be the height by hand (`flip`, `(h, w) = ...`, `number[j, i]`), so cells are bare `(Int, Int)` instead of domain positions (Owner: stdlib, `transposed()`, in the friction).
- Line 35 fills an array from a path with a loop (Owner: stdlib, in the friction).
- `print(-1); continue` (lines 28-30) could be `if ...: print(-1) else:`. Minor.
