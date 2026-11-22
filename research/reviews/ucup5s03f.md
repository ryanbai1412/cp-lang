# ucup5s03f

Score: 4/5
Dream: 4/5

Rescore of the rewrite, cold, on main 70a4aec4. Port and dream are the same file and check with zero errors. The header (lines 3-14) derives the WHT formula, the Krawtchouk steps and the `/ 2^n`, and confirms the `m + 1` values.

## Checklist failures

- Minor: `kraw` is advanced in place by two passes (lines 29-30); a named `step` would say what one `k -> k + 1` move is.

## Rewrites owed with today's language

- None beyond the minor notes.

## Gaps that need the language

- None found.
