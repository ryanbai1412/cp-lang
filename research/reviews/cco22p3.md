# cco22p3

Score: 4/5
Dream: 4/5

The layer DP from the header over (seen, room, kept), each state keeping its earliest time, with `last_le` on sorted starts. 0.30 s against 0.34 s.

## Checklist failures

- idiomatic-code/R4/R5: the slide schema `(a: 0..=1e9, b: 0..=1e9)` is written twice (lines 15-16) and then erased to `(a: Int, b: Int)` in `slides` (line 17).

## Owed rewrites

- `type Slide = (a: 0..=1e9, b: 0..=1e9)` and `let slides: [Room]List<Slide> = [read<[n1]Slide>().sorted(), read<[n2]Slide>().sorted()]`: this runs under 0.2, so the friction's "cannot be read in one `read()`" needs only that line.

## Language gaps

- Binding a list out of a table copies it, with no hint: research/frictions/cco22p3.md.

## Dream notes

- The dream is the port with the one-line read and `r.other`.
