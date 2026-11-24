# noi23p2

Score: 3/5
Dream: none (no dream.cp and no `## Dream` in the friction report)

Cold review on main 6a989896d: port, dream and friction report, not the C++. There is no dream (neither dream.cp nor a `## Dream` in the friction report), so the score is capped at 3 (research/reviews/README.md). The `Subset<Slot>` DP over pending deadlines reads like the header.

## Checklist failures

- Lines 39-42 accumulate raw residues (`ways[...].v`) and reduce once with `Z.new(w)`, a manual lazy-reduction the header justifies by speed; the max case is still 0.29 s against the C++'s 0.18 s (1.6x), which the report measured but listed under "No friction left" (idiomatic-code/R10). Now filed as friction 1.

## Owed rewrites

- Write the dream (with plain `Z` sums).

## Language gaps

- `Mod<P>` sums in the subset-DP loop are 1.6x the C++: friction 1 (added).
