# Friction Report: noi10p4

No friction left: the deadlines tighten along `topo(g)` in reverse, and the planes that must
precede u are `reachable(g.transpose(), [u])` (once O(n + m) per call, now O(reached), fixed in
std).

## Measurements
The schedule is not unique, so `checker.py` checks line 1 against k and the rules and line 2
exactly. Stress: 30/30 agree with the v0.2 port and with AC; max (n = 2000, m = 1e4) passes the
checker, 0.03 s vs C++ 0.06 s.
