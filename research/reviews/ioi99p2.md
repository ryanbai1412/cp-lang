# ioi99p2

Score: 3/5
Dream: 3/5

Cold review: the scan over text letters updating each word prefix's latest start, and a DP over `T.cuts`, read as the header says. The completion test repeats its formula and the walk back is a hand loop over cuts.

## Checklist failures

- R4: `words[s.w].len + best[from.l]` is written twice, in the test and the assignment (lines 41-42).
- R2: the walk back (lines 46-52) is a `while` over cuts with `took` kept beside `best`.

## Owed rewrites

- `let gain = words[s.w].len + best[from.l]` once, then compare and assign.

## Language gaps

- The ragged `[Word]List<?T>` start table, `T.cuts.at(Int(at) - 1)!` for the previous cut, and a DP that records its choice: all in research/frictions/ioi99p2.md.

## Dream notes

- The dream is a sketch: `start` is `...` and the DP body is pseudocode ("for each word w ending at c.prev"), so the start update, the core of the solution, is never written. It should give the update and name the span test.
