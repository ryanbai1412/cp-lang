# Friction Report: noip07p2

No gaps: one pass with `c.upper` and a reversed fill is the dream.

## Dream

The port as written. One pass: an inner `-` whose neighbours qualify becomes the run between them through `shown`, reversed for p3 = 2, each character pushed `repeat p2` times.

## Measurements
Stress: 30/30 agree with the v0.2 port and with AC 5184882; max (100 chars, p2 = 8) 0.004 s vs C++ 0.003 s, same output.
