# Friction Report: noi16p4

No friction left: the v0.2 port's hand-written input generator (`read(gen: tangle)`) and its
`assert l <= r` are gone, the bounds are one typed read, and the endpoints compress in one
comprehension. The window and the max/add SegTree over compressed coordinates are unchanged.

## Measurements
Stress: 30/30 agree with the v0.2 port and with AC; max (n = 5e5, m = 2e5) 0.77 s vs C++ 0.51 s,
same output.
