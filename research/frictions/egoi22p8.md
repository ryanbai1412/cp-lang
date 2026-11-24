# Friction report: egoi22p8

Rethought around the structure the v0.2 port hid. Dots are `(r, c)` tuples and segments are ordered dot pairs, not `3x + y` ids and a `min * 9 + max` key into an 81-cell array. The 28 segments are 7 orbits of a quarter turn, built by turning the orbit list. A drawn card is a `Set` of segments, and the least rotation is a `min` over the four digit rotations. `@!multitest` replaces the hand-written test loop. The `with1..with4` counts are the accepted C++ case analysis, unchanged.

No language gap. The accepted C++ is grader-only (`BuildPattern`, no main), so the port is compared against the v0.2 port, plus a round trip: decode of a randomly turned encode returns n, on 203 values including 1 and 6.7e7.

## Measurements

gen.py --max (1e5 tests): port 0.30 s, v0.2 port 0.24 s.
