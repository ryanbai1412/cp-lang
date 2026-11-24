# dmopc21c9p2

Score: 4/5
Dream: 4/5 (the friction's `## Dream`)

Scored cold against the checklist and the port.

A stack of letters where two equal letters merge into the next one (up to `z`), with matched letters of b popped from the front as they appear.

## Checklist failures

- `pending.last! == pending[pending.len - 2]` peeks two from the end by index.
- `matched` is an index into `b` advanced by hand; a cursor over `b` would hold it.
