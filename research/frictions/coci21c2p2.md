# Friction report: coci21c2p2

Already close to the bar. The v0.3 port types n, m and q from the statement, builds the Dsu with `Dsu<Box>()`, and reads each query as one `(Box, Box)` pair.

No language gap.

## Measurements

gen.py --max (n = m = 1000, q = 5e5): port 0.03 s, accepted C++ 0.52 s, same output.
