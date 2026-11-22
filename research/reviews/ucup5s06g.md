# ucup5s06g

Score: pending cold rescore (added dream, moved use std.graph.{bfs01} above @!multitest, derived why peeled vertices have DAG ancestors and cannot be free while unpeeled vertices lie on/below cycles)
Dream: pending cold rescore (new dream: top-level use, peeling cycle-reachability derivation)

Baseline review, cold, on main 614a9f1b. The port checks with zero errors. Clean: `peel` and `bfs01` say the algorithm, and both steps are commented. No dream exists, so the port is capped at 3.

## Checklist failures

- No dream; the dream may be this port. `use` sits after `@!multitest` (lines 4-5); move it to the top.
- Why peeled vertices can never be free (lines 11-13) is only asserted.

## Rewrites owed with today's language

- Fix the items above and write a dream.

## Gaps that need the language

- None found.
