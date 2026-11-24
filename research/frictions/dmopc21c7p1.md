# Friction report: dmopc21c7p1

No gap. Rows and columns are domains walked with `.rev`, neighbours are the grid `.down()`/`.right()` optionals, and the final check is one `all(...)` over the cells, in place of a `valid()` with early returns. Bounds are named constants (`ROOM` = 1e17, `TOP` = 1e18).

## Measurements

gen.py --max (1500 x 1500): port 0.13 s, accepted C++ 0.33 s.
