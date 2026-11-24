# noi14p1

Score: 3/5
Dream: none

Port 3/5, dream none. The friction report has no `## Dream` section and there is no `dream.cp`, so the port is capped at 3. All-zeros and all-ones through the gates once (lines 12-17), then the greedy from the top bit (lines 19-25): the header's argument.

## Checklist failures

- The gate is read as `(Str, ...)` and dispatched with `if`/`elif`/`else` (lines 14-17), so `else` silently means `XOR`; the friction describes a typed `[0..n](op: Str, t: ...)` read that the port no longer has.

## Owed rewrites

- Add a `## Dream`; read the gate as a tagged enum and `match` it.

## Language gaps

None.

## Dream notes

None.
