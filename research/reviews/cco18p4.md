# cco18p4

Score: 4/5
Dream: 4/5

Rows and columns are domains, the search is std `ternary_min` down column 1 and then along the best row, and the header counts the questions (about 73 of 75). The cache of asked scores is by hand because a memo may not ask.

## Checklist failures

- idiomatic-code/R5: the reply is typed `0..=2e9` on line 22, but the cache and `score` return plain `Int` (lines 19-20).

## Owed rewrites

- One `type Score = 0..=2e9` for the reply, the cache value and the return type.

## Language gaps

- A memo may not ask (spec 6.4), so the cache is spelled by hand: research/frictions/cco18p4.md.

## Dream notes

- `dream.cp` is the port with `@memo` on `score`; its header states the separation. `let s: 0..=2e9 = ask(...); s` could be the body itself. The friction says the dream uses `argmin_convex` and `wire.ask`; it does not.
