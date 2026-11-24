# noi20p2

Score: 3/5
Dream: none (no dream.cp and no `## Dream` in the friction report)

Cold review on main c8a46b201: port, dream and friction report, not the C++. There is no dream, so the score is capped at 3 (research/reviews/README.md). The max-convolution is one std `MergeTrees` merge whose hook the header derives.

## Checklist failures

- 1.08 s vs C++ 0.59 s at max (1.8x) is measured but filed as "No friction left" (idiomatic-code/R10); now a friction (section 1).

## Owed rewrites

- Write the dream.

## Language gaps

- `MergeTrees.merge` with a hook is 1.8x the C++: research/frictions/noi20p2.md section 1 (added by this review).
