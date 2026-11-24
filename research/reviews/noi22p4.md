# noi22p4

Score: 3/5
Dream: none (no dream.cp and no `## Dream` in the friction report)

Cold review on main 6a989896d: port, dream and friction report, not the C++. There is no dream (neither dream.cp nor a `## Dream` in the friction report), so the score is capped at 3 (research/reviews/README.md). Shapes from `Rooted.shapes`, a `Counter<Shape>` to cancel equal children and `max_matching` for the leftovers read like the header.

## Checklist failures

- Lines 38-50 build `left` and `right` by hand from one `Counter`, adding and removing as they go, to take two multiset differences of the children's shapes.
- Line 24 puts G and H in one `domain Node` by offsetting H's parents by `ng` (`Node.nth(base + p - 1)`); the header explains the shared forest (shapes must be comparable across both trees), but every H index is `ord` arithmetic (idiomatic-code/R6).
- Lines 53-60 declare `var ok` and reassign it inside `if ok:` instead of one expression.

## Owed rewrites

- Write the dream.
- Remeasure the max case: the friction's 0.28 s against 0.25 s is 1.12x, at the R10 line; if it repeats, report it as a perf friction (idiomatic-code/R10).

## Language gaps

- None.
