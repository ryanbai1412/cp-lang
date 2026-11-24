# noip18p2

Score: 4/5
Dream: 3/5

Cold review: a coin is needed iff the smaller ones cannot pay it: sort, count the unpayable coins and add each to an unbounded reachable-sums table (lines 16-24), the header's argument. Port score provisional until the dream is rewritten.

## Checklist failures

- None beyond the reported gap.

## Owed rewrites

- None.

## Language gaps

- The unbounded reachable-sums step by hand (lines 22-23): in research/frictions/noip18p2.md, now with the `cpl run` error for `Bitset.unbounded`.

## Dream notes

- `count(for c in coins.sorted(): if not payable[c]: payable.unbounded(c); true)` mutates inside `count` and is not an expression the language has; the dream should be a plain loop that counts and adds, and `Bitset.of` takes one argument.
