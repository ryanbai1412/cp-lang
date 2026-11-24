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
