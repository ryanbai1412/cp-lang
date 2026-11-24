# Friction report: dmopc21c9p4

No semantic gap. A line keeps its heads as read plus a `flipped` mask, so `peel` only reads its argument and nothing is copied (the old port mutated a copy of it).

## Measurements

gen.py --max (999 x 999): port 0.14 s, accepted C++ 0.18 s.
