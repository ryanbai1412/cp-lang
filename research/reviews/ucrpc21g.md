# ucrpc21g

Score: 4/5
Dream: 4/5

Cold review on main 377955a60, against the idiomatic checklist. Port and dream are the same file (dream added in this commit). `held` is a `[Row, Col]?Int` table rebuilt per tick, so unreachable cells are `None`, not a sentinel, and the header (lines 3-10) proves why keeping only the most popcorn per cell is safe; 37x faster than the accepted C++.

## Checklist failures

- `p.adj4 + [p]` builds a list per cell per tick; in the friction report (stdlib).

## Rewrites owed with today's language

- None.

## Gaps that need the language

- None found.
