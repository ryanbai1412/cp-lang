# ucup5s06g

Score: 4/5
Dream: 4/5

Rescore of the rewrite, cold, on main e1345392. Port and dream are the same file and check with zero errors. `use` is at the top, and the header (lines 3-7) argues why peeled vertices can't be free and unpeeled ones are.

## Checklist failures

- Minor: line 26 recovers edge direction with `edges[a.e].v == a.to`; a directed-pair cost on the arc would say it directly.
- Minor: line 28 checks `dist.reached(v)` and then unwraps `dist[v]!` for the same vertices.

## Rewrites owed with today's language

- None beyond the minor notes.

## Gaps that need the language

- None found.
