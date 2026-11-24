# noi09p5

Score: 3/5
Dream: none

Port 3/5, dream none. The friction report has no `## Dream` section and there is no `dream.cp`, so the port is capped at 3. The friction's entry says "Ideal code: as written", outside a `## Dream`. Two games in lockstep over `Mod<1024523>` with one layer kept (lines 19-32), as the header derives.

## Checklist failures

- idiomatic-code/R6: `[0..=n, 0..=n]Z` (lines 19, 22) are anonymous axes for "taken from pipe one by game one/two".
- The four transitions (lines 28-31) repeat the same bounds-and-compare shape; a `next_of(game, taken)` would state "the next marble of a game" once.

## Owed rewrites

- Add a `## Dream` to the friction; a `domain Taken = 0..=n` axis.

## Language gaps

None.

## Dream notes

None.
