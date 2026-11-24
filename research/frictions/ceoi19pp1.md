# ceoi19pp1 frictions

## Dream

The committed `dream.cp` is the target; the port matches it except for the coordinate type.

## Coordinates stay `Int`

The statement has increasing coordinates in `0..=2^30`, but the package's inline cases use
negative and unsorted values, so `a` and `b` are read as `[n]Int` to keep those cases.
