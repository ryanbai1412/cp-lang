# ucup5s05b

Score: 3/5

The header (lines 1-8) is editorial quality: it states the folded diamond, why each piece is a straight segment, and the x-sweep. The body is still coordinate arithmetic over raw `Int` with a hand interval-union sweep. No dream exists, so the score is capped at 3. Scored cold against `design/EXEMPLARS-PORTS.md`.

## Checklist failures

- Hand interval-union length (lines 60-64): `open`/`prev`/`ans` sweep over `(x, ±1)` events.
- Cyclic side walk by index (lines 33-36): `corners[(z + 1) % 4]`; `pairs()` has no cyclic form.
- `sign` via `Int(bool)` arithmetic (line 9).
- Swap-to-nonzero-`b` packed into a four-tuple conditional (line 30).
- Unbounded query reads (line 68): `(Int, Int, Int, Int)` while line 23 bounds its reads.

## Owed rewrites

- One `sides` list of `(start, end)` for the diamond; bounded query types; push intervals and take a union length.

## Language gaps

No friction report exists. Missing and owed as a friction: union length of half-open intervals, and a cyclic `pairs`, in the stdlib.
