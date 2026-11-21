# Graph search benchmarks

Matched-algorithm comparison of `std.graph` searches against hand-written
C++ on identical generated inputs (`python3 bench/graph/run.py`). Each pair
reads the same graph, runs the same algorithm and prints the same checksum;
the script checks the outputs agree and reports best-of-N wall time.

| bench | cpl (s) | hand C++ (s) | ratio |
|---|---:|---:|---:|
| bfs | 0.255 | 0.154 | 1.66x |
| bfs01 | 0.559 | 0.221 | 2.53x |
| dijkstra | 0.959 | 0.408 | 2.35x |
| fixed_cost | 0.443 | 0.239 | 1.85x |

The gap is not in the search loops. Two codegen limits cost most of it:

- A loop over a `yields` fn (`for a in g.out(u)`) streams without a list
  only when the body has no closure call. `dijkstra` and `bfs01` call
  `w(a)` in that body, so every popped vertex builds a `std::vector` of
  its arcs.
- Reading and building the graph (`[E](u, v, w)` input, CSR build) is
  about 1.6x the hand version on its own, which is the whole `bfs` gap.

Returning `Paths` (parent and arc witnesses, settle order) instead of a
bare distance array costs about 15% on `dijkstra` (0.97 s to 1.13 s on the
same input, same peak memory).
