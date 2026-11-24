# Friction report: coci21c6p2

Rethought lightly. The v0.2 port read untyped `Int`s and folded the cost through an `offer` helper that hid the `min=`. The v0.3 port types the grid and every bound, names the cost `cost(m)` and folds with `best min= cost(..)` into a `?Int`. The two-pointer state is one `(right, sum)` pair.

No language gap.

## Measurements

gen.py --max: port 0.20 s, accepted C++ 0.19 s, same output.
