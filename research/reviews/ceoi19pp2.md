# ceoi19pp2

Score: 4/5
Dream: 3/5

Cold review against the friction's `## Dream`. The port is the header's stack argument: decode with the previous answer (`kept.len`, line 18), push a new prefix maximum (lines 19-21), otherwise `pop_while` the undercut ones (line 23).

## Checklist failures

- R5, minor: `var high: ?Int` (line 15) drops the `0..M` bound that `kept: List<0..M>` keeps.
- Minor: `high == None or x > high!` (line 19) is `high.is_none_or(h => x > h)`, which `std/opt.cp` has.

## Owed rewrites

- `var high: ?(0..M)` and `if high.is_none_or(h => x > h):`.

## Language gaps

- None demonstrated; the friction's optional prefix maximum is expressible with `is_none_or`.

## Dream notes

- `for (b, high) in bs.decoded(...).with_prefix_max()` (the whole dream) contradicts its own friction: the decoded value depends on the previous answer, so no input pipeline yields it. The dream should be the port's loop with `is_none_or`; the port is judged against that.
