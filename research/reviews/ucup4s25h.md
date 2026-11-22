# ucup4s25h

Score: 4/5
Dream: 4/5

Rescore of the rewrite, cold, on main e1345392. Port and dream are the same file and check with zero errors. The header (lines 3-13) now explains `go_right` and why `s[i - 1] > s[i]` decides the list.

## Checklist failures

- Minor: the `|s| <= 500_000` bound lives in the header (line 3), not the type on line 16.
- Minor: line 25 concatenates both lists to read one index.

## Rewrites owed with today's language

- None beyond the minor notes.

## Gaps that need the language

- None found.
