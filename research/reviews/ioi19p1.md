# ioi19p1 review

Score: 3/5
Dream: 4/5

Scored cold against the checklist, the port and `corpus/problems/ioi19p1/dream.cp`.

The greedy is the editorial's: fix the leftmost shoe, pull its nearest waiting partner, count
the standing shoes between with a `Fenwick`. But "j is paired" is kept twice, and the header
names the wrong structure.

## Checklist failures

- `var paired: [P]Bool` and `standing.add(j, -1)` (lines 20, 25-26) record the same fact in
  two places (R4); the dream keeps it once in `matched`. Not reported.
- The header says "A SegTree of standing shoes" (line 9); the port uses a `Fenwick` (R1).
- `domain Shoe = -n..=n` with `Shoe(-Int(x))` stands in for a signed anonymous range;
  reported with its E0501 repro.

## Owed rewrites

- Drop `paired`; test `standing.get(i) == 1` (or keep the dream's `matched`).

## Language gaps

- A table over a signed anonymous range: in research/frictions/ioi19p1.md.

## Dream notes

- At bar: one `Fenwick`, `waiting` queues, one `swaps +=` line.
