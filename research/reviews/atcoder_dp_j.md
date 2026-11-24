# atcoder_dp_j

Score: 4/5
Dream: 5/5

Cold review of `port.cp` against `dream.cp` (the same code), without the C++. The header
derives the `(a, b, c)` state, the expectation equation and the fill order. `if let a1 =
a.prev` takes each guard, and `print(..., precision: 10)` states the output format.

## Checklist failures
- idiomatic-code/R1: no `limits:` header line.

## Owed rewrites
- The header line.

## Language gaps
None.

## Dream notes
`Count(dishes[Pieces(1)])` at the end is a little indirect; a `let (a, b, c) = ...` would name
the start state. A nit, not a gap.
