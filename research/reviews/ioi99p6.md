# ioi99p6

Score: 3/5
Dream: 4/5

Cold review: per left column the row spans widen right, and two std `Window`s give the spread over a run of rows; the two-pointer walk and the best-so-far are hand loops, both reported with reasons that hold. 3.2x faster than the C++.

## Checklist failures

- R5: `best` is an optional compared with `best == None or area > best!.area` (line 40) and printed with `best!` (line 47); the reported `max_by` gap.
- R4: the `while let` test (lines 32-35) writes `hi[r]` and `lo[r]` twice each, once as the `??` fallback and once as the new value.

## Owed rewrites

- `max(peak.all ?? hi[r], hi[r])` is `peak.with(hi[r])`-shaped; until std has it, binding `let (top, low) = (max(..), min(..))` before the test keeps the condition on one line.

## Language gaps

- `Window.pop` needs the popped value, the two-pointer stays a loop, and the first-maximum `best`: all in research/frictions/ioi99p6.md.

## Dream notes

- The comment "kept incrementally" contradicts `h[r, left..=right].minmax()`, which reads as a recompute; the dream should say which it means.
