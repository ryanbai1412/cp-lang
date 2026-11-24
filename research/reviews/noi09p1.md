# noi09p1

Score: 3/5
Dream: none

Port 3/5, dream none. The friction report has no `## Dream` section and there is no `dream.cp`, so the port is capped at 3. The friction's entry says "Ideal code: as written", outside a `## Dream`. Positions are edges of a std graph between their two candidate values, forced values are peeled from a worklist (lines 31-38), and each remaining cycle is oriented from its smaller value (lines 40-49), as in the header.

## Checklist failures

- `deg` (line 24) is kept by hand beside the graph and decremented on both ends (lines 36-37).
- `take` (lines 25-29) both tests and assigns, and every caller turns its `false` into `return None` (lines 35, 43, 47).

## Owed rewrites

- Add a `## Dream` to the friction (the port as written, if that is the claim).

## Language gaps

None.

## Dream notes

None.
