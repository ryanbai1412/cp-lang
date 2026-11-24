# Friction Report: ioi05p1

## Dream

```cp
for w in C.stretches(w => strip[w].sum() >= k):   // the narrowest window from each left column
    if strip[w].sum() == k: least min= 2 * (bottom - top + 1 + w.len)
let down = above(garden.flipped_rows())
```

## Gaps

- **The narrowest window per left end is a hand two-pointer.** There is no
  std "shortest window from each start whose fold reaches k", so the strip scan
  keeps `right` and `inside` by hand, walking `C.nth(right)` by position.
  Severity: low.
- **No row-flipped view of a grid.** `transposed()` exists in `std.grid`, but
  the garden read bottom-up is spelled `[R, C] (r, c) => garden[R.nth(R.len -
  1 - r.ord), c]`, and the cut loop matches rows by position through `nth`.
  Severity: low.
