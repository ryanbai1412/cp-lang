# atcoder_dp_x: friction report

Port: `corpus/problems/atcoder_dp_x/port.cp`, dream: `corpus/problems/atcoder_dp_x/dream.cp`
(identical code).

## Measurements

- Tokens: port 161, accepted C++ 395.
- Emitted C++: 15258 bytes.
- `--max` (n = 1000, median of 15): port 6.8 ms / 3.1 MB, accepted 5.5 ms / 2.8 MB (1.24x time).
- Output: identical to the accepted C++.

## Friction 1: 1.24x on a 7 ms run

- Where: the knapsack loop `if let v = best[x]: best[Weight(x.label + b.w)] max= v + b.v`.
- What felt wrong: nothing in the source. The difference is about 1.3 ms of a run that is
  mostly process start-up and reading 3000 numbers; the `?Int` table is a pair per cell
  where the accepted C++ uses `-1` in a plain `long long[]`. The table is 2e4 + 1 cells,
  so the extra 0.3 MB is noise.
- Goal it hurts: R10 (formally).
- Ideal code: unchanged. `?Int` for "no tower of this weight" is the correct type; a
  `-1` sentinel would be a regression.
- General fix: store `?T` for an integer `T` with a niche (the bound `1..=1e9` sums leave
  negatives unused) as one word. That is a general codegen layout change.
- Severity: low.
- Owner: Core (codegen, niche layout for bounded `?Int`).

The exchange-argument sort `sorted(by: b => b.s + b.w)` and the `..=Weight(b.s)` scan
read as the editorial.
