# Friction Report: etsr23r3p2

## Dream

The port is the dream: unfold the folds backwards from `Fold.end`, keeping a
point or adding its mirror image across each fold line, and count the copies
that land on the 100 x 100 sheet.

## Gaps

None found. Coordinates are `F64` vectors from `std.geometry` because the
reference reads doubles; `reflect` and `cross` are the std spellings.
