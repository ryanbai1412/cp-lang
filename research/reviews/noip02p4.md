# noip02p4

Score: 3/5
Dream: 3/5

Cold review on main 6a989896d: port, dream and friction report, not the C++. Recursive separating cuts plus the k = 4 pinwheel priced from quadrant tables follow the header. The dream leaves `pinwheel` undefined, so the score is provisional until it is rewritten.

## Checklist failures

- Lines 48-51 call `quadrant(ps, xs, ys, false, true)` with two positional `Bool`s; named arguments (`right: false, up: true`) say which quadrant.
- Lines 52-62 index `below` and the quadrant tables by raw `i1, i2, j1, j2` over the deduped coordinates (idiomatic-code/R6).
- Line 79 `for turn in 0..2` with `if turn == 0` picks the axis by an integer.
- Lines 18-30 and 66-72 hand-roll the bounding box: reported (friction 1).

## Owed rewrites

- Name the quadrant arguments.
- Rewrite the dream with `pinwheel` written out.

## Language gaps

- A `Rect` bounding-box monoid: friction 1.
