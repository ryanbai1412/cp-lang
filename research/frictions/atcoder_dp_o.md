# atcoder_dp_o

Ported from `corpus/submissions/4131780.cpp`. Dream: `corpus/problems/atcoder_dp_o/dream.cp`.

## No gaps
The port and the dream are the same. `ways: [Taken]Z` over `Subset<Woman>` is the mask DP with
its domain kept. The man is `Man(t.len - 1)`, so the men's order needs no extra state, and
`sum(ways[t.without(w)] for w in t if likes[m, w] == 1)` is the editorial's recurrence. Two
nits that are not gaps: `if t != Taken.empty:` could be a filter on the `for`, and
`const P = 1000000007` lacks the `_` separators other ports use.

## Measurements
- Tokens: port 142, original 382.
- `--max` (n = 21, median of 15): port 74.8 ms / 10.5 MB, original 153.9 ms / 10.6 MB (0.49x).
  Same output.
- Emitted C++: 18,785 bytes.
