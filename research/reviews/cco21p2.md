# cco21p2

Score: 3/5
Dream: 3/5

A layered search from x toward 0 with each layer below the first as a `Bitset` window, shifted per digit by residue, then a walk back. Correct and fast, but the search and its reconstruction are by hand, and the friction describes a different hand search (a `HashMap` of back digits).

## Checklist failures

- idiomatic-code/R3: the BFS (lines 44-49) and path reconstruction (lines 50-56) are hand-written; the gap (`bfs` over `Int` states) is reported with a repro.
- idiomatic-code/R1: `domain Window = 0..2 * m + 8` (line 22): the header says "2M + 8" but not where the 8 comes from.

## Owed rewrites

- Update the friction's "Where" to this port's bitset layers.
- Derive the window size in the header.

## Language gaps

- `bfs` over `Int` states does not check: research/frictions/cco21p2.md.

## Dream notes

- `dream.cp` is itself a hand BFS over a `FlatMap` with a hand walk back (idiomatic-code/R3, caps at 3); the friction's one-line `bfs(peel, from: [x], until: 0)` is the dream it should hold.
