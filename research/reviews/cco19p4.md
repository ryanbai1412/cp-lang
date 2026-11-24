# cco19p4

Score: 3/5
Dream: 4/5

The DP from the header with a std `LiChao` per value over how many of its cards are ahead. The curve is extended past its domain with a clamp instead of limited to it, and the friction describes the other version.

## Checklist failures

- idiomatic-code/R2: `score[max(m - x, 0)]` (line 24) makes each curve defined everywhere; std `LiChao.add(line, on: ?Range<Int>)` limits it to the ranks where it is defined, as the friction says the port does. It does not (line 33 has no `on:`), so the clamp and its effect on the single-crossing argument are unexplained.

## Owed rewrites

- `runs[v].add(Run(ahead[v] + 1, best), on: 0..=ahead[v] + 1)` and `score[m - x]`, or explain the clamp in the header; fix the friction's description either way.

## Language gaps

None open: the friction reports no remaining gap.

## Dream notes

- The dream fragment is the port with `add(on:)` and `LiChao(max, over: 0..=count[v])`: the header's argument. No header of its own.
