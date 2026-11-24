# ioi15p5 review

Score: 3/5
Dream: 4/5

Scored cold against the checklist, the port and `corpus/problems/ioi15p5/dream.cp`.

The binary search over `rounds` with std `perm_cycles` and the replay of Ermek's rounds read as
the header says. The row-and-inverse pair is hand-synced (`Row`, `swap`), which is reported.
The plan is built by a second hand loop that swaps `end` into place (lines 37-43) where the
dream reads it off `perm_cycles` of the same row, already in use on line 34.

## Checklist failures

- `var end = row(after(rounds))` and the selection-sort loop (lines 37-43) sort a copy of the
  row to list its swaps (R3); the dream's
  `after(rounds).perm_cycles().flat_map(c => c.adjacent())` says the plan is the row's cycles.
- `Row` with `w.at[w.val[x]] = x` (lines 27-32) keeps a permutation and its inverse in sync by
  hand; reported.

## Owed rewrites

- Plan from `perm_cycles(after(rounds))`, as the dream does, and drop the `end` row.

## Language gaps

- A `Perm<Pos>` type with `swap` and `inverse`, and folding swaps over a value: in
  research/frictions/ioi15p5.md.

## Dream notes

- At bar: the plan from cycles, a single inverse-kept `swap`, and the replay loop.
