# ioi22p2

Score: 4/5
Dream: 4/5

Baseline review, cold, on main 96359635. Reads like the editorial. The header (lines 11-18) derives the 21-state tree, and `Step` plus the `path` walk carry the idea. Remaining issues are small and spelling-level. No dream exists, so the score is capped at 3.

## Checklist failures

- Free `all(...)` (line 75) breaks §53; write `pairs.all(...)`. Withdrawn: §53.1 keeps the free generator form canonical.
- `where N <= 60` on the checker (line 74): check it against §70.
- `size` built in a hand reverse loop (lines 27-28) where a `scan` over `FANOUT.rev` says it.
- `for _ in 0..500` (line 79) is an unexplained step bound; the protocol bound is `x + 1` moves.

## Rewrites owed with today's language

- Method `all`; `scan` for `size`; derive the play bound from the state count.

## Gaps that need the language

- The grader's `-1`/`-2` action codes are fixed by the interface, so they are not a gap.

## Cold review (HEAD, after rewrite)

Dream 4/5 (new), port (`today/ioi22p2.cp` = dream) 3 → 4/5, rescored cold on current main. Both check with zero errors. The header (lines 6-14) derives the 21 states and the `DEPTH + 1` bound, and `Step` with `step_action` carries the walk.
- Owed: `proto/ports/scout_batch_25/ioi22p2.cp`, the statement-first sketch, still fails with 5 errors: `where` on a `fn` (line 74), `unreachable` (line 40), and `match` as a value (lines 49, 59). Delete it, or rewrite it to the `today` form.
- Minor: `rsize` is built reversed and then re-indexed (lines 22-25), where a backward fill into `size` says it directly. `first_state` (lines 28-29) is a hand prefix sum. `path(v)` is recomputed for every state (line 60) instead of once per `v`.
