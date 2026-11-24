# cco20p4

Score: 2/5
Dream: 3/5

The per-start two-run construction from the header, with colours as an enum. The case split reads as the proof, but pops discard what was peeked, and the colour flip is inline because the friction says enums cannot have methods, which they can.

## Checklist failures

- idiomatic-code/R7: `first.pop()` (lines 41, 45) and `second.pop()` (line 49) discard the city already read with `.last`.
- idiomatic-code/R13: `x = if x == Red: Blue else: Red` (line 36): an enum declares a method today (`enum Colour:` with `fn flip(self) -> Colour = ...` runs under 0.2), so the friction's "an enum cannot declare a method" does not hold.

## Owed rewrites

- Move the popped city (`second.push(first.pop()!)`-style) instead of popping and reusing the peek.
- `fn flip` on `Colour`; drop the friction bullet.
- Fix the friction's `## Dream`: `dream.cp` is the same construction with `Bool` colours, not an Euler tour.

## Language gaps

- No lower-triangle reader: research/frictions/cco20p4.md.

## Dream notes

- `dream.cp` keeps `Bool` colours ("red when true"), `if u == s: continue` and a wrapped `red[...]` index across four lines (idiomatic-code/R2, caps at 3). The port improved on it.
