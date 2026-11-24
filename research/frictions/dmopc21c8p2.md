# Friction report: dmopc21c8p2

No gap. Parity classes and column sums are domains (`Parity`, `Sum = 2..=2M + 1`). The per-class counters are `[Parity]` tables and the difference array is one `[Parity, Sum]` table, in place of `List`s indexed by `0`/`1`. Like the accepted C++, the count assumes N >= 2, so both parity classes have a column.

## Measurements

gen.py --max (N = M = 1e6): port 0.07 s, accepted C++ 0.10 s.
