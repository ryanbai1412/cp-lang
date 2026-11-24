# ucrpc21f

Score: 4/5
Dream: 4/5

Cold review on main 377955a60, against the idiomatic checklist. Port and dream are the same file (dream added in this commit). A weighted `dijkstra` over the grid with `adj4` moves, three lines of logic. Agrees with the reference at max; 1.06x its time.

## Checklist failures

- Memory is 1.8x the original's at max (20.3 MB against 11.4 MB); in the friction report (perf).

## Rewrites owed with today's language

- None.

## Gaps that need the language

- None found.
