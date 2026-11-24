# Friction Report: noi19p1

No friction left: the hand-rolled dynamic Li Chao tree (four parallel lists and `new_node`) is a
`LiChao(min, over: 0..=1000)` per station, the trains are one typed read plus the dummy in a
`Train` domain, and the event kind is a Bool so departures sort before arrivals.

## Measurements
Stress: 30/30 agree with the v0.2 port and with AC; max (n = 1e5, m = 2e5, hub stations) 0.11 s
vs C++ 0.08 s, same output.
