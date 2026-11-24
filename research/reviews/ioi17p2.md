# ioi17p2 review

Score: 3/5
Dream: 4/5

Scored cold against the checklist, the port and the dream (the `## Dream` section of research/frictions/ioi17p2.md).

The DP is the editorial's: `cost` over `P.cuts`, each point either wired to its nearest other
colour or closing the block since the latest cut with the same balance, `latest` keyed by a
`Balance` domain. The merge-by-sort and the two nearest scans are reported. But `near` uses a
`FAR` sentinel the friction report does not describe: its Where still cites
`min(gaps[p].0 ?? gaps[p].1!, ...)`, an earlier form.

## Checklist failures

- `const FAR = 1_000_000_000` and `var near: [P]0..=FAR = FAR` (lines 25-26) stand for "not
  seen yet" (R5); both colours occur, so every `near[p]` is overwritten, but the type does not
  say so. Not reported in this form.
- The forward and backward scans (lines 28-34) are the same two lines mirrored; reported.

## Owed rewrites

- Refresh the Where line of the nearest-match friction to the `FAR` form, or compute
  `near[p]` as the `min` of the two optional neighbours once `min_some` exists.

## Language gaps

- `merge_by` and `prev_match`/`next_match`: in research/frictions/ioi17p2.md.

## Dream notes

- Close to the bar: `merge_by`, `points.nearest(where:)`, the same DP. `bal` and `colour` are
  used undefined.
