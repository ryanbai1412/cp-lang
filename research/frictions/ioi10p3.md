# ioi10p3

## Prefix built from a grid table is 1.63x the C++ at --max
- **Where:** `some_median_at_most`, lines 17-19: the port fills `Prefix<Row, Col, Int>.filling()` with `set` cell by cell.
- **What felt wrong:** the check is one prefix table of `rank <= x`; `Prefix([Row, Col] (i, j) => Int(rank[i, j] <= x))` says it in one line, but it materializes the `[Row, Col]` table before summing it. On the 3000 x 3000 --max case that is 2.745 s of CPU against the accepted C++'s 1.685 s (1.63x, over the 1.5x gate), so the port keeps the fill loop.
- **Goal it hurts:** fast (the obvious spelling is the slow one).
- **Ideal code:** the dream below.
- **General fix:** `Prefix.new` fed a table literal should fuse the table into the summing loop instead of allocating it. Owner: Perf B (0dea8747).
- **Severity:** medium.

## Dream

`first_true` over the answer, and each check is one `Prefix` built from the table `[Row, Col] (i, j) => Int(rank[i, j] <= x)`, tested over every `windows(h)` by `windows(w)` pair:

```cpl
fn some_median_at_most(x: Int) -> Bool:
    let low = Prefix([Row, Col] (i, j) => Int(rank[i, j] <= x))
    any(2 * low.fold(rows, cols) > h * w for rows in Row.windows(h) for cols in Col.windows(w))
```
