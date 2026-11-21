# ioi22p2

Score: 3/5

Baseline review, cold, on main 96359635. Reads like the editorial. The header (lines 11-18) derives the 21-state tree, and `Step` plus the `path` walk carry the idea. Remaining issues are small and spelling-level. No dream exists, so the score is capped at 3.

## Checklist failures

- Free `all(...)` (line 75) breaks §53; write `pairs.all(...)`.
- `where N <= 60` on the checker (line 74): check it against §70.
- `size` built in a hand reverse loop (lines 27-28) where a `scan` over `FANOUT.rev` says it.
- `for _ in 0..500` (line 79) is an unexplained step bound; the protocol bound is `x + 1` moves.

## Rewrites owed with today's language

- Method `all`; `scan` for `size`; derive the play bound from the state count.

## Gaps that need the language

- The grader's `-1`/`-2` action codes are fixed by the interface, so they are not a gap.
