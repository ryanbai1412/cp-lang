# atcoder_dp_e

Score: 5/5
Dream: 5/5

Cold review of `port.cp` and `dream.cp` (the same program), without the C++. The header derives
the swap to value-indexing. `lightest: [Value]?Int` makes the unreachable values absent, and
`Value.at(t - v)` covers the left edge. The answer line reads as the statement says it.
Bounds are typed, including `Value = 0..=sum(v ...)`.

## Checklist failures
None.

## Owed rewrites
None.

## Language gaps
None.

## Dream notes
An exemplar of an optional DP table.
