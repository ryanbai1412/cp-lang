# Friction report: dmopc21c6p3

No gap. The expansion is k rounds of a frontier from the coloured cells in colour order over `p.adj4`; the first colour to reach a cell keeps it, so the paint grid also marks the cells seen and no distance grid is needed. That replaces the hand-written wave lists, the `DIRS` table, the bounds-checked `step` helper and the colour sort (the old `WISH L1`/`WISH L7` notes).

## Dream

The port as written. Painted cells spread for k rounds in colour order; the first colour to reach a cell keeps it, so the paint grid also marks the cells seen.

## Measurements

gen.py --max (1500 x 1500, K = 3000, 40 sources): port 0.43 s, accepted C++ 0.31 s.
