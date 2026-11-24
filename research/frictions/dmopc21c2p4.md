# Friction report: dmopc21c2p4

No semantic gap: `height.runs()` gives the equal-height runs and three optional counts carry the DP (on its own, spilling into the next run, covered from the next run), so no state domain or sentinel height is needed.

## Measurements

gen.py --max (N = 1e6): port 0.02 s, accepted C++ 0.02 s.
