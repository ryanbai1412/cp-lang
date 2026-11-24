# atcoder_dp_e

Ported from `corpus/submissions/4131644.cpp`. Dream: `corpus/problems/atcoder_dp_e/dream.cp`.

## No gaps
Port and dream are the same program. Indexing by value instead of weight is in the header, and
`lightest: [Value]?Int` makes "no subset reaches t" absent instead of an INF. `Value.at(t - v)`
handles the left edge without a guard, and `min=` on an optional keeps the present side. The
answer reads as the statement says it ("the largest t whose lightest weight fits"). The
accepted C++ needs an INF constant and a `min` against it.

## Measurements
- Tokens: port 163, original 334.
- `--max` (n = 100, best of 3): port 15.7 ms / 3.4 MB, original 15.9 ms / 3.3 MB. Same output.
- Emitted C++: 14,361 bytes.
