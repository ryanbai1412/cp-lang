# Friction Report: noi18p4

No friction left: `@multitest` replaces the test loop, every array is a typed read, the weakest
sword is `swords.first`, and the congruence merge is one `if let ... and let` instead of
`all == None or one == None` with four unwraps.

## Measurements
Stress: 30/30 agree with the v0.2 port and with AC; max (T = 5, n = m = 1e5, planted answers)
0.76 s vs C++ 0.69 s, same output.
