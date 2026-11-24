# Friction Report: noip18p4

No gaps: `Graph.undirected` with edge ids gives the edge to leave out as `?E`, and the best walk is `min` over the walks that reach every town.

## Measurements
Stress: 30/30 agree with the v0.2 port and with AC 4004140; max (n = 5000 with a cycle) 0.18 s vs C++ 0.26 s, same tokens.
