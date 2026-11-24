# egoi22p8

Score: 2/5
Dream: none

Cold review of `port.cp` (no `dream.cp`, no friction `## Dream`, so capped at 3). Dots and segments are tuples, the 28 segments are 7 quarter-turn orbits (lines 56-65), a drawn card is a `Set`, and the least rotation is a `min` over four turns: the geometry reads well.

## Checklist failures
- idiomatic-code/R1: `with1`..`with4` (lines 29-42) are "the four cases of the accepted C++"; neither the comment nor the header derives `f * f * f + f * f + f + 1 + f * (f + 1) / 2` or the `with4` conditions.
- idiomatic-code/R4: the digit width is written as `DIGIT = 128`, `127 - a` (lines 30, 33, 36), `128 - b` (lines 34, 37) and `7 *` shifts (lines 25, 54, 82); `BITS = 28` and the `0..4`/`0..7` tables repeat the same 4 x 7 shape. Not in the friction.

## Owed rewrites
- Name the shape once (`const TURNS = 4`, `const ORBITS = 7`, `DIGIT = 1 << ORBITS`) and derive the rest; derive the least-rotation counts in the header.

## Language gaps
None shown.

## Dream notes
No dream.
