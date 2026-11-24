# Friction Report: noi15p2

No friction: `Rooted.from_parents`, `tree.chains` (root-to-x as heavy-chain ranges) and
`tree.sub` (subtree range) on a `SegTree(+, act: assign)` say the whole solution; the re-port
types the bounds (`n, q: 1..=1e5`).

## Measurements
Stress: 30/30 agree with the v0.2 port and with AC; max (n = q = 1e5, a path) 0.09 s vs C++
0.08 s.
