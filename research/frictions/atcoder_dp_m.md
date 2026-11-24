# atcoder_dp_m

Ported from `corpus/submissions/4131778.cpp`. Dream: `corpus/problems/atcoder_dp_m/dream.cp`.

## No gaps
The port and the dream are the same. The sliding window `window += ways[g]`, minus
`ways[g - most - 1]` when `Given.at` finds it, is the editorial's prefix-sum window with the
left edge as an optional, not an `if (g - a - 1 >= 0)`. `ways: [Given]Z` keeps the modulus in
the type, so the subtraction cannot go negative by mistake.

## Measurements
- Tokens: port 147, original 425.
- `--max` (n = 100, k = 1e5, best of 3): port 31.7 ms / 3.4 MB, original 63.9 ms / 4.1 MB. Same
  output.
- Emitted C++: 13,495 bytes.
