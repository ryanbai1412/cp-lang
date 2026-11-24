# Friction Report: ioi07p3

## Dream

The port is the dream: a range-add min `SegTree` over heights, where
`right_while` finds the run of heights at the pivot count and the run below it.

## Gaps

- **`k <= h` is not in the read type.** A struct field's bound can't name a
  sibling field, so `masts: [n](h: 1..=1e5, k: 1..=1e5)` reads `k` wider than
  the statement. Severity: low.
- **Moving a cut by a count goes through `Int`.** "`rest` heights from `level`"
  is `Height.cuts(Int(level) + rest)`, and the run below `v` is clamped to the
  mast top with `min`. Severity: low.
