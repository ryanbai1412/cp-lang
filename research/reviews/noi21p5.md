# noi21p5

Score: 3/5
Dream: none (no dream.cp and no `## Dream` in the friction report)

Cold review on main c8a46b201: port, dream and friction report, not the C++. There is no dream, so the score is capped at 3 (research/reviews/README.md). W and E as 2x2 matrices in a std `SeqFold` with a flip action, as the header derives; the lazy reduction in `dot` (line 21) is commented.

## Checklist failures

- Line 53 matches `REVERSE` as `_`, so any unknown op reverses.
- 0.41 s vs C++ 0.35 s at max (1.17x) has no performance friction (idiomatic-code/R10).

## Owed rewrites

- Write the dream.
- Match `"REVERSE"` explicitly, or read the op as an `enum`.

## Language gaps

- None.
