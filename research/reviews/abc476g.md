# abc476g

Score: 3/5
Dream: 4/5

Cold review of `port.cp` against the ideal under `## Dream` in `research/frictions/abc476g.md`
(there is no `dream.cp`), without the C++. The header derives the Dilworth reduction (the
answer is the longest subsequence with non-increasing popcount), the `f[k][a][b]` table over
aligned blocks and the per-block DP, with costs. `@alt(program) fn slow()` is the direct scan,
for stress.

## Checklist failures
- idiomatic-code/R12: reads outside the table's triangle are kept out by hand guards and clamps:
  `(if c > k - 1: 0 else: f[k - 1, min(a, k - 1), c])` and
  `(if c == 0 or b - 1 > k - 1: 0 else: f[k - 1, min(c - 1, k - 1), max(0, b - 1)])`. They are
  correct but hard to check; the friction routes a checked multi-index `get`.
- idiomatic-code/R2: the block decomposition is a `while` loop with a nested `while` growing
  `k` (`while k < MAXB and not lo.bit(k) and lo + (2 << k) <= hi`), which reads like the C++.
  A std "aligned dyadic blocks of [l, r)" would name the object the editorial uses.

## Owed rewrites
- None today: both items above need std support. The friction routes the first; the second
  goes to the friction as a follow-up if another port needs it.

## Language gaps
- Multi-index `get` (E0303, in the friction).

## Dream notes
The dream changes only the guarded reads, so it inherits the hand-rolled block loop; 4/5.
