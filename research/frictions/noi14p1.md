# Friction Report: noi14p1

No friction: the gates are a typed `[0..n](op: Str, t: 0..=1e9)` read folded once from all-zeros
and once from all-ones, then a 30-bit greedy.

## Measurements
Stress: 30/30 agree with the v0.2 port and with AC; max (n = 1e5) 0.015 s vs C++ 0.012 s.
