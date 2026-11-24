# Friction report: egoi23p1

Rethought. The v0.2 port counted prices in a `Map` by hand and read untyped `Int`s. The v0.3 port builds a `Counter(prices)`, moves a whole count with `at.remove(x - inflation)` (an optional, so a missing price needs no `?? 0` sentinel) and `at.add`, and types every bound (`n`, `prices`, `q`, `x`, `y`).

No language gap.

## Measurements

gen.py --max (n = 3e5, q = 1e5): port 0.04 s, accepted C++ 0.21 s, same output.
