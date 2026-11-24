# sac22cc1p3

Score: 4/5
Dream: none (no dream.cp; the ideal is in the friction report)

Scored cold against the checklist and the port.

Cold review: the grid is [Row, Col]Char with offsets instead of index arithmetic; each W slides with a parallel cell swap, and the drop pass collects its moves against the old board before applying them, which is what the statement asks. Bounds now match the problem line (100, not 50).

## Checklist failures

- `slide_left` walks each `W` with a cursor `at` and swaps one cell at a time, as a direct simulation. It is O(nm) per step at n, m <= 100, so not a cost, but a row-wise "pack the W's left until a non-dot" would state the move.
- Cells are compared to the characters `'W'` and `'.'` throughout, where a small enum read from the grid would name them.
- `kind: 1..=2` is dispatched with `if kind == 1` and an implicit else.
