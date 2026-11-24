# noi21p1

Score: 3/5
Dream: none (no dream.cp and no `## Dream` in the friction report)

Cold review on main c8a46b201: port, dream and friction report, not the C++. There is no dream, so the score is capped at 3 (research/reviews/README.md). The colour-run monoid with a `None` empty range and a lazy paint over heavy-light chains is the editorial.

## Checklist failures

- Lines 35-38 keep a `random_tree` generator in the solution for `read(gen:)`: test scaffolding the reader skips.
- Line 40 seeds distinct start colours as `-p.ord - 1` so they never equal an update number; an enum or `?Int` colour would say "own colour" without the negative trick (idiomatic-code/R5).
- Line 44 reads the op kind as `1..=2` and branches on `kind == 1`; an `enum` read with `@tag` names it.

## Owed rewrites

- Write the dream.
- Read the operation as an enum.

## Language gaps

- None.
