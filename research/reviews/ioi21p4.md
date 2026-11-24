# ioi21p4

Score: 4/5
Dream: 4/5

Cold review: prefix counts of the nine (a, b) letter pairs, a balance test, then pair swaps plus threes (lines 28-36), exactly the header's argument.

## Checklist failures

- None beyond the reported gaps.

## Owed rewrites

- None.

## Language gaps

- `Base` as `0..3` with a hand `base(c)`, nine indicator `Prefix`es, row/column sums spelled as generators, and the grader's `-1`: all in research/frictions/ioi21p4.md.

## Dream notes

- `have.off_diagonal.sum` is invented without a note; the port's `sum(have[s, t] for s in Base for t in Base if s != t)` is already clear, so the dream gains little there.
