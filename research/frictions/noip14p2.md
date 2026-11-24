# Friction Report: noip14p2

No gaps: `Graph.undirected` for the tree, `max_k(2)` for the top two neighbour weights, `Mod<10007>` for the sum.

## Dream

The port as written. Per middle vertex, `max_k(2)` of the neighbour weights and (sum)^2 - sum of squares in `Mod<10007>`.

## Measurements
Stress: 30/30 agree with the v0.2 port and with AC; max (n = 2e5) 0.041 s vs C++ 0.051 s, same output.
