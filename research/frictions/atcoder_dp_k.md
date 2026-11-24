# atcoder_dp_k

Ported from `corpus/submissions/4131775.cpp`. Dream: `corpus/problems/atcoder_dp_k/dream.cp`.

## No gaps
The port and the dream are the same. `wins[s] = any(Stones.at(s - x).is_some_and(rest => not
wins[rest]) for x in a)` is the editorial's definition: a move exists that leaves a losing
position. `Stones.at` takes the place of the `s >= a_i` guard.

## Measurements
- Tokens: port 109, original 308.
- `--max` (k = 1e5, n = 100, best of 3): port 7.6 ms / 3.1 MB, original 15.6 ms / 2.6 MB. Same
  output.
- Emitted C++: 11,455 bytes.
