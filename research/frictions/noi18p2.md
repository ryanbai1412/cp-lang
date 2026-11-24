# Friction Report: noi18p2

No friction left: `@multitest` reads each case, so the v0.2 port's buffered `List<List<Int>>`
and its `max_n` pass are gone. The permutation is typed `[1..=n]1..=n`, `Comb.choose` is already
0 outside the triangle (no `safe_choose`), and the smallest unused value is a `find`.

## Measurements
Stress: 30/30 agree with the v0.2 port and with AC; max (T = 5, sum n = 2e6) 0.06 s vs C++
0.67 s, same output.
