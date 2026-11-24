# noi18p3

Score: 3/5
Dream: none (no dream.cp and no `## Dream` in the friction report)

Cold review on main c8a46b201: port, dream and friction report, not the C++. There is no dream, so the score is capped at 3 (research/reviews/README.md). The suffix automaton of s with mergeable end-position trees is std `SuffixAutomaton` and `MergeSegs`, and the header states the step condition.

## Checklist failures

- Lines 24-28 keep `ends` as a list grown by hand to match the automaton's state count (`while ends.len < of_s.len`), parallel to `of_s` (idiomatic-code/R2).
- Line 30 walks `(1..order.len).rev` and indexes `order[i]` where `order.drop(1).rev` says it.
- Lines 45-53 are a `while true` loop with two `break`s for "step if allowed, else shorten".

## Owed rewrites

- Write the dream.
- Iterate `order.drop(1).rev` directly.

## Language gaps

- None.
