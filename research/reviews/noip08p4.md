# noip08p4

Score: 3/5
Dream: 4/5

Cold review: the clash graph, `ParityDsu` for bipartiteness and the due-pop simulation follow the header, but `clash` keeps a symmetric branch no caller uses, the leaders are a push-first loop, and the friction's graph section describes code the port no longer has.

## Checklist failures

- R13: `if j < i: return clash(j, i)` (line 27) is dead; the only caller passes `i < j` (line 31).
- R2: `lead` (lines 33-36) is filled by a first-seen loop over `[Pos]?Pos`.
- R10: the port is 0.011 s against the C++'s 0.004 s on the max case; the friction gives the numbers but no cause.

## Owed rewrites

- Drop line 27.

## Language gaps

- The friction's section 2 (`apart` built from a predicate) describes the earlier graph; the port now unites into `ParityDsu`. The max-case time is filed in research/frictions/noip08p4.md.

## Dream notes

- None.
