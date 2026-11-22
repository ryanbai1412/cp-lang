# wtf26b

Score: 4/5
Dream: 4/5

Rescore of the rewrite, cold, on main 70a4aec4. Port and dream are the same file and check with zero errors. Suffix sums over `y` (lines 28-33) remove the inner `y` loop; the header states O(n^4), about 1.3e8 at `n = 200` (line 16), and explains the seed (lines 7-8).

## Checklist failures

- Minor: `suf` is built by a hand suffix scan over one axis of a 3-D table (lines 28-33).

## Rewrites owed with today's language

- None beyond the minor notes.

## Gaps that need the language

- None found.
