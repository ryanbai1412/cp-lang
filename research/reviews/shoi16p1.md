# shoi16p1

Score: 3/5
Dream: 4/5

Cold review: the signed-reach tree DP with a suffix min (lines 26-38) is the header's, but the children's sum is spelled five times where dream.cp names it once, and `marked` is a loop where `V.mask` exists.

## Checklist failures

- R4: `sum(best[c, ...] for c in t.children(v))` at lines 29, 32, 34 and 36; dream.cp's `fn at(r)` (line 16) says it once.
- R3: `marked` is filled by a loop (lines 23-24); `V.mask(must)` (dream.cp line 10) runs today.

## Owed rewrites

- Dream.cp's `kids`, `fn at` and `V.mask`.

## Language gaps

- Signed domain arithmetic and a row-valued `Rooted.up`: in research/frictions/shoi16p1.md.

## Dream notes

- None.
