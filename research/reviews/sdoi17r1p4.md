# sdoi17r1p4

Score: 4/5
Dream: 4/5

Cold review on main 377955a60, against the idiomatic checklist. Port and dream are the same file (dream added in this commit, recorded by `corpus.py verify`). The header (lines 3-9) proves why Dinkelbach's iteration stops at the optimum, and the loop (lines 18-22) is exact in integers: no epsilon, no binary search. 230x faster than the accepted C++ at max.

## Checklist failures

- Minor: the two sums on line 22 repeat `best.mate[i]`; a `let pairs = Boy.map(i => (i, best.mate[i]))` or a `best.pairs` would say it once.

## Rewrites owed with today's language

- None.

## Gaps that need the language

- None found.
