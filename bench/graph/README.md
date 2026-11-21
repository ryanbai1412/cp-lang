# Graph search benchmarks

Matched-algorithm comparison of `std.graph` searches against hand-written
C++ on identical generated inputs (`python3 bench/graph/run.py`). Each pair
reads the same graph, runs the same algorithm and prints the same checksum;
the script checks the outputs agree and reports best-of-N wall time.

| bench | cpl (s) | hand C++ (s) | ratio |
|---|---:|---:|---:|
| bfs | 0.187 | 0.190 | 0.98x |
| bfs01 | 0.367 | 0.299 | 1.23x |
| dijkstra | 0.535 | 0.524 | 1.02x |
| fixed_cost | 0.217 | 0.189 | 1.15x |
| scc | 0.250 | 0.146 | 1.71x |
| topo | 0.151 | 0.158 | 0.96x |
| bridges | 0.259 | 0.176 | 1.48x |
| articulation | 0.209 | 0.152 | 1.38x |
| lca | 0.419 | 0.242 | 1.73x |
| euler | 0.199 | 0.149 | 1.34x |
| two_sat | 0.792 | 0.484 | 1.64x |
| mst | 0.466 | 0.376 | 1.24x |

p50 1.29x, max 1.73x (bar: 1.1x and 1.5x). Every hand version runs the
library's algorithm: `bridges_hand.cpp` finds two-edge-connected components
and reports the tree edges between them, as `bridges` does. An earlier hand
version with a different bridge algorithm made `bridges` look 3.50x; against
the matched one it was 2.2x before the two fixes below.

What closed the gap so far:

- Out-edge loops stream without a list even when the body calls a closure
  (`w(a)` in `dijkstra`, `bfs01`).
- Unread result fields are not stored (decision 135): `Paths.back` is gone
  when only distances are read. `bfs` 1.07x to 0.94x, `bfs01` 1.36x to
  1.20x, `dijkstra` 1.09x to 1.02x on one run.
- A `let` takes the array its comprehension built by move, not by copy:
  `bridges` 0.49 s to 0.39 s, 203 MB to 145 MB.
- The depth-first searches read `next` through one flat `Adj` with
  `deg`/`arc` instead of one list per vertex (decision 136): `bridges`
  2.2x to 1.45x, `scc` 2.92x to 1.69x, `articulation` 2.70x to 1.32x,
  `two_sat` 2.37x to 1.70x, `euler` 2.04x to 1.32x.

Still over the bar:

- `scc` and `two_sat` (which runs `scc`): `?Int` timestamps take 16 bytes
  where the hand version uses 4, the frame stack is a tuple of 8-byte
  fields, and a stored `Graph` is still copied into the flat `Adj` once.
  Ranged optionals (`?0..n`, 4 bytes) are coming from the flows lane; the
  timestamps move to them in `graph.cp` after that.
- `lca`: not profiled yet.
