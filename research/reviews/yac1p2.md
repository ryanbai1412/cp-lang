# yac1p2

Score: 3/5
Dream: 4/5

Cold review of `port.cp` against the ideal under `## Dream` in `research/frictions/yac1p2.md`
(there is no `dream.cp`), without the C++. The header derives the complement count (paths with
at most two nodes of each colour, subtracted from all n + n(n-1)/2 paths), the `down[u, b, w]`
table, the children-up merge and the cost. `Rooted`, `domain Few = 0..3` and `choose2` keep the
setup short, but the merge is translated.

## Checklist failures
- idiomatic-code/R2: the pairing sum is four nested `for` loops over `Few` (L26-38) with
  `Int(b) + Int(b2) < 3 and Int(w) + Int(w2) < 3`, formatted across nine lines; the dream's one
  `sum(... for (b, w) in Few * Few for (b2, w2) in Few * Few if b + b2 < 3 ...)` says it.
  Reported: `b + b2` on `Few` is still E0309 "`+` can't be applied to `Few` and `Few`" under a
  fresh `cpl run` (2026-11-24), and `Few * Few` as a product domain is in the friction.
- idiomatic-code/R2: the step of a pair of counts is two nested `if let Some(..) = Few.at(..)`
  (L41-42); the friction routes a tuple pattern over optionals.

## Owed rewrites
- None today; both depend on the reported language gaps.

## Language gaps
- `Int`-valued `+` between positions of an integer-labelled domain (E0309), and product
  domains `Few * Few` (in the friction).
- Matching a tuple of optionals when every part is present (in the friction).

## Dream notes
The dream names the DP, sums the pairing in one expression and steps both counts with one
pattern. Its header is a single line rather than the port's derivation, so 4/5: it should carry
the port's Problem/Solution text.
