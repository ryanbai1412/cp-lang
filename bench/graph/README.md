# Graph search benchmarks

Matched-algorithm comparison of `std.graph` searches against hand-written
C++ on identical generated inputs (`python3 bench/graph/run.py`). Each pair
reads the same graph, runs the same algorithm and prints the same checksum;
the script checks the outputs agree and reports best-of-N wall time.

| bench | cpl (s) | hand C++ (s) | ratio |
|---|---:|---:|---:|
| bfs | 0.156 | 0.157 | 0.99x |
| bfs01 | 0.320 | 0.202 | 1.58x |
| dijkstra | 0.412 | 0.350 | 1.18x |
| fixed_cost | 0.178 | 0.165 | 1.08x |

The gap is not in the search loops. Two codegen limits cost most of it:

- A loop over a `yields` fn (`for a in g.out(u)`) streams without a list
  only when the body has no closure call. `dijkstra` and `bfs01` call
  `w(a)` in that body, so every popped vertex builds a `std::vector` of
  its arcs.
- Reading and building the graph (`[E](u, v, w)` input, CSR build) is
  about 1.6x the hand version on its own, which is the whole `bfs` gap.

Returning `Paths` (one `back` witness array, settle order) instead of a
bare distance array is one extra write per improving relaxation. Storing
`parent` and `via` as two arrays cost `dijkstra` 0.478 s (1.30x); one
`back` array is 0.412 s (decision 78).
