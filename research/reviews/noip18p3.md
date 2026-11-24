# noip18p3

Score: 3/5
Dream: 4/5

Cold review: binary search k, and per k a bottom-up pass where each junction pairs its children's chains in a `SortedBag` and hands up the longest spare (lines 26-45), but the header is scrambled.

## Checklist failures

- R1: the header's lines 5-9 are out of order ("each junction gets the" then "pairs them in a SortedBag", line 8 continuing line 5) and still mention a binary search and path compression the code no longer has.

## Owed rewrites

- Rewrite the header's solution in order, describing the `SortedBag` pairing.

## Language gaps

- The post-order fold through `chains` and `tree.at.rev` (lines 35-41): in research/frictions/noip18p3.md, now with the `cpl run` error for `Rooted.fold_up`.

## Dream notes

- None; it recurses to depth 5e4, which the friction says the port avoids, so it relies on the fold gap being fixed.
