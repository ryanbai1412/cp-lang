# Friction report: dmopc21c8p1

No gap. The input is sorted in place and the unused numbers are the window `a[lo..hi]`, so no sorted copy or popped list is needed. The bounds are types.

## Measurements

gen.py --max (n = 1e6): port 0.19 s, accepted C++ 1.08 s.
