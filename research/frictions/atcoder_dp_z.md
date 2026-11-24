# atcoder_dp_z: friction report

Port: `corpus/problems/atcoder_dp_z/port.cp`, dream: `corpus/problems/atcoder_dp_z/dream.cp`
(identical code).

## Measurements

- Tokens: port 152, accepted C++ 739.
- Emitted C++: 20593 bytes.
- `--max` (n = 2e5, median of 15): port 23.8 ms / 7.2 MB, accepted 28.9 ms / 6.9 MB (0.82x time).
- Output: identical to the accepted C++.

## Frictions

None. `LiChao(min, over: 1..=1e6)` with `Line(-2 * h[s], cost[s] + h[s] * h[s])` is the
convex-hull-trick editorial written once, and `lines.at(h[s])!` is safe because a line is
added before the first query; the `!` is visibly guarded by `s != Stone.first!`.
