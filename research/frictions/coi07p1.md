# Friction report: coi07p1

Already at the bar: a single greedy sweep over the typed positions, with no index arithmetic. Only header lines were added (limits, `// Bounds:`).

## Gap

K and the positions are reals, so they read as `F64` with no range type: F64 values have no range types.

## Measurements

gen.py --max (n = 1e5): port 0.18 s, accepted C++ 0.03 s, same output; the limit is 600 ms.
