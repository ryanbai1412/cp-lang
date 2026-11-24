# noi21p2

Score: 3/5
Dream: none (no dream.cp and no `## Dream` in the friction report)

Cold review on main c8a46b201: port, dream and friction report, not the C++. There is no dream, so the score is capped at 3 (research/reviews/README.md). LGV reduces the answer to one determinant, std `det`. The friction report says the counts are built with `Mat` `identity`, `zeros` and `at`; the port (lines 24, 29) builds `List<List<Z>>` by hand and converts with `Mat.from_rows`.

## Checklist failures

- Line 24 hand-writes the identity (`Z(if r == c: 1 else: 0)`) where `Mat<Z>.identity` exists (idiomatic-code/R3).
- Lines 26 and 32 index layers and rows through `g.ord`, `u.ord`, `v.ord` (idiomatic-code/R6).
- Line 17 spells the case marker `@multitest` (idiomatic-code/R8).
- The friction report describes a different build of the matrix.

## Owed rewrites

- Correct the friction report or adopt what it says.
- Write the dream.

## Language gaps

- None.
