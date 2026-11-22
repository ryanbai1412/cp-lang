# ucup4s25h

Score: pending cold rescore (added dream, renamed state -> go_right with full derivation of right-to-left deletion ordering and s[i-1] > s[i] transition, stated |s| <= 500_000 bound in header)
Dream: pending cold rescore (new dream: go_right ordering derivation and |s| bound)

Baseline review, cold, on main 6566cb18. The port checks with zero errors. The header (lines 1-2) names the ordering, but `state` (lines 8-12) isn't explained. No dream exists, so the port is capped at 3.

## Checklist failures

- Say what `state` means and why `s[i - 1] > s[i]` decides which list `i` joins.
- `s` has no length bound (line 4).

## Rewrites owed with today's language

- Write a dream.

## Gaps that need the language

- None found.
