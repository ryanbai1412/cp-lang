# atcoder_dp_y: friction report

Port: `corpus/problems/atcoder_dp_y/port.cp`, dream: `corpus/problems/atcoder_dp_y/dream.cp`
(identical code).

## Measurements

- Tokens: port 268, accepted C++ 695.
- Emitted C++: 19304 bytes.
- `--max` (h = w = 1e5, n = 3000, median of 15): port 34.9 ms / 4.5 MB, accepted 53.0 ms / 5.8 MB (0.66x time).
- Output: identical to the accepted C++.

## Frictions

None. `type Cell = (r: 1..=h, c: 1..=w)` makes start, goal and walls one type, so
`paths(a, b)` serves all three uses and the sort is the record order. `paths` returning
0 for a cell not down-right of the other is the number of paths, not a sentinel.

The header lacks `original:` (header edit, idioms).
