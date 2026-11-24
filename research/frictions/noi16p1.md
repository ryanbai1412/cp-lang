# Friction Report: noi16p1

No friction left: the square counts are two `Diff<Pos, Int>` difference arrays fed by
`lyndon_runs`, with each run's halves `(period..=len / 2).step(period)`.

## Measurements
The v0.2 snapshot no longer checks (2 errors), so stress ran against AC only: 30/30 agree; max
(10 strings of 30000) 0.11 s vs C++ 0.24 s, same output.
