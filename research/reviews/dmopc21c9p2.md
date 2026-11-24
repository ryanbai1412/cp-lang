# dmopc21c9p2

Score: 4/5
Dream: none (no dream.cp; the ideal is in the friction report)

Scored cold against the checklist and the port.

A stack of letters where two equal letters merge into the next one (up to `z`), with matched letters of b popped from the front as they appear.

## Checklist failures

- `pending.last! == pending[pending.len - 2]` peeks two from the end by index.
- `Char(Int(x) + 1)` steps a letter through `Int`.
- `matched` is an index into `b` advanced by hand; a cursor over `b` would hold it.
