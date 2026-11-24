# noi17p1

Score: 3/5
Dream: none (no dream.cp and no `## Dream` in the friction report)

Cold review on main c8a46b201: port, dream and friction report, not the C++. There is no dream, so the score is capped at 3 (research/reviews/README.md). The header (lines 3-9) derives the up/down split and the borrow test, and the or-SegTree searched with `left_while` replaces the -1 sentinel.

## Checklist failures

- Lines 23-24 choose `up` or `down` twice from the same `x > 0` test, once to read and once to write (idiomatic-code/R4).
- `d` in `add` is a raw `Int`, so lines 23-25 convert it with `Digit(d)` five times (idiomatic-code/R6).
- Line 35 turns the cut back into a digit as `Digit(c.ord - 1)`: cut-to-position arithmetic through `ord` (idiomatic-code/R6).

## Owed rewrites

- Write the dream.
- Loop over `d: Digit` and bind the side once (`let side = if x > 0: mut up else: mut down`, or pass the side array to `add`).

## Language gaps

- None.
