# cco18p5

Score: 4/5
Dream: 4/5

The header's argument is the program: an arg-max `SegTree` of `?Peak` finds each lecture's best partner within K - 1 on either side, a max `SegTree` holds the pair at the larger end, and an update refreshes the position and its two old partners. 0.40 s against 1.90 s for the C++.

## Checklist failures

None beyond the reported gaps.

## Owed rewrites

- `var paired = 0` then a nested `if let` (lines 37-40) can be one expression, as the dream's `if let ... and ...: a[i] + v else: 0`.

## Language gaps

- The `better` Option monoid: research/frictions/cco18p5.md.
- "Within k - 1 on one side" as `I.nth(max(0, i.ord - k + 1))` index arithmetic: same report.

## Dream notes

- `max` lifted over `?(Int, I)` and `i.before(within: k - 1)` read like the editorial. The window fold is written four times (two in `refresh`, two in the update); a `fn partners(i)` would say it once. No header.
