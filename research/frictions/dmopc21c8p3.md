# Friction report: dmopc21c8p3

No gap. `most` is a `[0..=n]` table capped at `k + 1`. The free positions are a `Fenwick` of ones, and `first_gt(offset)` picks the chosen one, so the old `Set.nth` note no longer applies. The permutation is `[Pos]Val`.

The earlier accepted submission 4522760 prints a permutation that is not the lexicographically smallest one (on `10 32` it prints `1 2 9 7 ...` while `1 2 7 9 ...` also has 32 valleys). This port agrees with the header's original, 4537755, on 30 generated cases and at max.

## Measurements

gen.py --max (N = 1e6, K near 1e18): port 0.01 s, accepted C++ 0.00 s.
