# aac5p3 friction

The port's code is `dream.cp`, so there is no dream/port gap. The port is within 1.1x in time
but uses 2.3x the memory.

## Twice the accepted C++'s memory
- **Where:** `port.cp`: `Rooted(Graph.undirected(...))`, `var climb: [V]Int`,
  `var reach: [V]Range<Int>`, and per node `tree.children(v).collect().sorted(by: ...)`.
- **What felt wrong:** 34 MB against the C++'s 15 MB at n = k = 2e5. Likely most of it (not profiled by
  allocation) is the rooted tree's tables (order, parent, size, child lists) next to the input edges, plus one sorted
  child list per node. The C++ keeps one adjacency list.
- **Goal it hurts:** performance (memory).
- **Ideal code:** the port as written.
- **General fix:** `Rooted` stores only the tables a program uses, or builds them lazily.
  Every tree port would use less memory.
- **Severity:** low (well under the 256 MB limit).
- **Owner:** Content (stdlib `Rooted`). Repro: `/usr/bin/time -f %M` on the port at `--max`.

## Measurements
- Tokens: port 347, original 709.
- `--max` (best of 10): port 0.068 s / 34 MB, accepted C++ 0.063 s / 15 MB (1.08x). Same
  output.
- Emitted C++: 28,078 bytes.
