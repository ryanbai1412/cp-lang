# Friction Report: noi19p5

No friction left: the coefficients `a, b, c`, the hand-expanded case analysis for positions 1, 2
and n, and the n <= 2 special cases (the statement has n >= 3, now a type) are one
falling-factorial sum per position 0, 1, 2 refit with std `interpolate` and read with `eval`;
the cuts and queries are read where they are used.

## Measurements
Stress: 30/30 agree with the v0.2 port and with AC; max (n = 1e7, m = Q = 5e5) 0.26 s vs C++
0.19 s, same output.
