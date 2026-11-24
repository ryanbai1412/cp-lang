# cco18p2

Score: 4/5
Dream: 4/5

An output-only construction: costs 100 except 1s on two interleaved chains, printed as the upper triangle. `cheap` names the chains and the header says why the greedy falls off them.

## Checklist failures

- idiomatic-code/R1: the header states `X = 9702` without the count behind it, and line 15 also makes 0-2 cheap, which the two listed chains do not include.
- idiomatic-code/R13: `and r.ord + 2 < N` (line 17) always holds for an odd `r` in `V.all.drop_last(1)` with N = 100.

## Owed rewrites

- Derive X in the header (which moves the greedy pays 100 for) and mention the 0-2 edge.
- Drop the redundant guard.

## Language gaps

- `for c in V if c > r` filters every row: research/frictions/cco18p2.md (`V.after(r)`).

## Dream notes

- The dream is the port with `V.after(r)`; it keeps the redundant guard. No header.
