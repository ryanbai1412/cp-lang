# tsoc15c2p6

Score: 4/5
Dream: 4/5

Cold review of `port.cp` against the ideal under `## Dream` in
`research/frictions/tsoc15c2p6.md` (there is no `dream.cp`), without the C++. The header
derives why clamping the minimum at 0 afterwards is exact (values only decrease) and names the
lazy SegTree with range add and min, with its cost. Each attack is one `row.apply(l..=r, -kills)`
and one line printing both clamped folds: the editorial, line for line.

## Checklist failures
- idiomatic-code/R5: the bound on an attack's strength, `kills: 0..=1e9` (L10, L20), is taken
  from the original's `int` rather than the statement. Reported in the friction (low).

## Owed rewrites
- None beyond confirming the `kills` bound against the statement.

## Language gaps
- None. `max(row.fold(l..=r), 0)` is the canonical two-value `max`; the friction's
  `clamp_low` would only rename it.

## Dream notes
The dream is the port with a renamed clamp, so it inherits the port's editorial shape: a named
`SegTree` over `domain Column`, each fact once. 4/5, not 5: the `kills` bound is still unstated.
