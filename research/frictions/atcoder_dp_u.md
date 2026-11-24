# atcoder_dp_u: friction report

Port: `corpus/problems/atcoder_dp_u/port.cp`, dream: `corpus/problems/atcoder_dp_u/dream.cp`
(identical code).

## Measurements

- Tokens: port 138, accepted C++ 449.
- Emitted C++: 17819 bytes.
- `--max` (n = 16, median of 15): port 32.9 ms / 3.5 MB, accepted 52.8 ms / 3.6 MB (0.62x time).
- Output: identical to the accepted C++.

## Frictions

No code gaps. `Subset<R>` tables, `s.subsets` and `s - g` say the O(3^n) submask
recurrence without a single bit operation, and the port is faster than the bit-twiddled
accepted C++.

The header has no `limits:` line though meta.json records 2000 ms / 1024 MB. Header
edit owed by idioms; the lint that requires it is with Consumers (see atcoder_dp_g).

## Addendum: pairs inside a subset
- **Where:** `score[g] = sum(a[i, j] for i in g for j in g if i < j)` visited every ordered pair
  to keep half of them.
- **Fix (this port and its dream):** `score[g] = sum(a[i, j] for (i, j) in g.pairs())`.
  `Subset.pairs()` already existed. The friction left is discoverability: a lint that turns
  `for i in X for j in X if i < j` into `X.pairs()` would catch the same shape in every pairwise
  sum. Owner: lint-fmt.
- **Measurements after the rewrite:** port 132 tokens, 18,276 bytes of C++; `--max` best of 3:
  port 0.06 s / 4.1 MB, accepted 0.09 s / 4.5 MB, same output. `corpus.py verify`: 0 findings.
