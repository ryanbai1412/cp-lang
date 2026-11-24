# apio07p2

Score: 3/5
Dream: 5/5

Cold review of `port.cp` against the ideal under `## Dream` in `research/frictions/apio07p2.md`
(there is no `dream.cp`), without the C++. The header gives the reduction to "k non-adjacent
gaps of least sum", its convexity in k, and the penalized linear DP behind `aliens`. `gaps`
comes from `at.adjacent`, and `aliens(k, priced, lambdas: ...)` is std.

## Checklist failures
- idiomatic-code/R8: `priced` returns `(val:, count:)` but keeps its state in positional pairs
  (`before.0`, `before.1`) and converts at the end.

## Owed rewrites
- Named state, as in the friction's dream.

## Language gaps
None.

## Dream notes
The editorial in 10 lines.
