# cco23p2

Score: 3/5
Dream: 4/5

Cold review against the friction's `## Dream`; 3 because the 1.22x slowdown is measured but has no perf friction. Caps come from `h.scan(max)` and `h.rev.scan(max)` (lines 19-20), columns are bucketed by compressed level (lines 21-22), the still-low columns live in a `SortedSet<Col>` (lines 29, 34-36) and one `least_above` sweep serves both directions (lines 49-57). The header derives the per-level cost used at lines 42 and 45.

## Checklist failures

- R3: `for l in Level:` with `let Some(u) = l.next else: break` (lines 31-32) and `seen.range(q.x + 1..).first!` (line 54); both reported.
- R10: the friction measures 3.15 s against the C++'s 2.58 s (1.22x), over the 1.1x bar, and names no cause or perf owner.

## Owed rewrites

- `least_above(order: Seq<Col>, ...)` called with `Col` and `Col.rev` (line 57), dropping both `.collect()` copies.
- Add a performance friction for the 1.22x with a profile (the two `Col.collect()` copies on line 57 and the per-ask `SortedSet.range` are the candidates).

## Language gaps

- `Level.adjacent` fails in the backends (E0501): in the friction.
- `SortedSet.first_gt`: not a gap; std spells it `range(x + 1..).first` (fresh run in the friction).
- A walk over a domain as a parameter: fixed, `Seq<Col>` takes `Col` (fresh run in the friction).

## Dream notes

- A fragment over the port (`Level.adjacent`, `first_gt`, `least_above(Col, ...)`), judged as the port with those changes; that reads like the editorial.
