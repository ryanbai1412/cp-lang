# apio07p1

Score: 3/5
Dream: 5/5

Cold review of `port.cp`, without the C++; there is no `dream.cp`, and the friction's dream is
the port as written. The header states the depth-span test, keep or swap, and the root check.
`read(none: -1)` gives `kids: [Bar](l: ?Bar, r: ?Bar)`, and `span(u, d) -> ?Span` returns
`None` for an unsortable subtree, with `let ... else: return None` stopping early.

## Checklist failures
None found. Capped at 3 for the missing `dream.cp`.

## Owed rewrites
- Commit the dream (the port) as `dream.cp` after `corpus.py verify` passes.

## Language gaps
None.

## Dream notes
Every editorial object is a named value, and there are no sentinels or flags.
