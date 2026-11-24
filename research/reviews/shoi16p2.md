# shoi16p2

Score: pending
Dream: pending

Cold review: Mod<100_000_007> replaces the hand %-MOD arithmetic, X and Y domains make point() return None off the grid, and a square is named by its lowest and highest corners (always opposite) instead of a hand-packed code list. A square met at a pair of its c deleted corners adds back 2/c mod the prime, so no list of squares is kept (upstream showed the sorted list blew the 125 MB limit on clustered points) and no corner order is needed.
