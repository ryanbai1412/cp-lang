# Friction report: dmopc21c6p3

No gap. The expansion is std `bfs` from every coloured cell over `p.adj4`, with `enter: d <= k`. In BFS order, each claimed cell takes the lowest colour among its neighbours one step closer. That replaces the hand-written wave lists, the `DIRS` table, the bounds-checked `step` helper and the colour sort (the old `WISH L1`/`WISH L7` notes).

## Measurements

gen.py --max (1500 x 1500, K = 3000, 40 sources): port 0.43 s, accepted C++ 0.31 s.
