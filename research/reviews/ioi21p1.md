# ioi21p1

Score: 4/5
Dream: 4/5

Cold review: the editorial in code. `Walk` with `then` is the segment tree's monoid, the sweep sets each day's step as boxes enter its range, and the answer is one `left_while` plus one `fold` (lines 45-47).

## Checklist failures

- None beyond the reported gaps.

## Owed rewrites

- Line 47 is one long `if let ... and ...: ... else: ...`; binding `let before = Day.at(Int(from) - 1)` first would split the case from the formula.

## Language gaps

- The grader entry, hand-bucketed `opens`/`closes`, `Day.at(Int(from) - 1)` for the day before a cut, `Monoid(Walk(0, 0, 0), then)` beside the type, and the closed `fold(from..Day.cuts.last!)`: all in research/frictions/ioi21p1.md.

## Dream notes

- `w.span`, `tail.below_top` and `tail.above_low` are named but not defined on `Walk`; the dream should give them as `Walk` methods so the closing formula is derived.
