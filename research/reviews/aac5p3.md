# aac5p3

Score: 3/5
Dream: 3/5

Cold review of `port.cp` against `dream.cp` (the same code bar the header lines), without the C++.
Std `Rooted` walks parents first. Each child gets its slice of the parent's skill interval, split
at the midpoints between neighbouring hills, and `count_in` counts the sorted skills inside.
That is the editorial, with the tie rule derived.

## Checklist failures
- idiomatic-code/R5: `var climb: [V]Int = 0` (line 18) gives the root a fake hill of difficulty
  0. It is never read for the root, so it is harmless, but `?Int` or a `[tree.non_root]` table
  would say so.

- idiomatic-code/R4: the midpoint rule `(climb[x] + climb[y]) / 2 + 1` is written twice, once
  for each neighbour (lines 29-30). Each interior boundary is computed twice.

## Owed rewrites
- Compute the boundaries once, `[reach[v].l] + [(climb[x] + climb[y]) / 2 + 1 for (x, y) in
  down.adjacent] + [reach[v].r]`, and give child i the clipped range between boundaries i and
  i + 1.

## Language gaps
- 2.3x the C++'s memory (34 MB against 15 MB), put down to `Rooted`'s tables: in the friction
  (stdlib).

## Dream notes
The dream is the port, so it repeats the midpoint formula (caps at 3). The `(a + b) / 2 + 1`
itself is derived by the "tie to the easier hill" line.
