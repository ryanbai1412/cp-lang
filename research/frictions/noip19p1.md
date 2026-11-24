# Friction Report: noip19p1

No gaps: `U64` holds k < 2^64, so the v0.2 port's two 32-bit halves are gone; the answer is `k ^ (k >> 1)` printed bit by bit.

## Measurements
Stress: 30/30 agree with the v0.2 port and with AC 5127723; max (n = 64, k = 2^64 - 1) same output.
