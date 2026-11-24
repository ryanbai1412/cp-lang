# cco22p3 — Double Attendance (CCO 2022 P3)

Port: `corpus/problems/cco22p3/port.cp`. The old port ran Dijkstra with a hand-kept priority
queue over (seen, room, kept) states; the rewrite notes every move sees exactly one more slide,
so states settle layer by layer and a plain DP over `Seen` keeps each state's earliest time.

## Frictions

- **Binding a list out of a table copies it.** `let (mine, theirs) = (slides[r], slides[o])`
  inside the per-state loop deep-copied both 3e5-element lists every state (quadratic in all: past 60 s on the max case),
  with no lint or `cpl explain` hint at the binding. The port indexes `slides[r][...]` directly.
- **Two rooms read as two lists.** `[Room]List<...>` cannot be read in one `read()` because the
  counts differ (`n1`, `n2`), so `first`/`second` are read and then put in the table.

## Dream

The port's body is the dream apart from the read; the ideal read is

```cp
let (n1, n2, k): (1..=3e5, 1..=3e5, 1..=1e9) = read()
domain Room = 0..2
let slides: [Room]List<(a: 0..=1e9, b: 0..=1e9)> = [read<[n1]_>(), read<[n2]_>()].map(sorted)
domain Seen = 0..=n1 + n2
fn started(r: Room, t: Int) -> ?Int = slides[r].last_le(t, by: s => s.a)
// ... DP body as in port.cp, indexing slides[r] and slides[r.other]
```

## Measurements

Max case (n1 = n2 = 3e5): port 0.30 s, accepted C++ 0.34 s, same output.
