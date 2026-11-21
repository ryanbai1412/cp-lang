# ucup5s05b

Score: 4/5
Dream: 4/5

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

## Cold review (HEAD)

Dream 4/5 (new; port = dream), port → 4/5, rescored on current main. It checks with zero errors. The header derives the folded diamond and the x-interval union.
- Reported gaps: `union_len` (lines 24-31) and the per-axis `signs` (line 33).
- Minor: `corners[(z + 1) % 4]` (line 52) is a cyclic pairing written by hand. The `d % m` trick in `to_mirror` (lines 19-22) isn't explained. `(a, b, c, t)` (line 77) are unbounded `Int`s.
