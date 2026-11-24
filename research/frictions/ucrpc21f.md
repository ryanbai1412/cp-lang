# Friction report: ucrpc21f

`dream.cp` is now a copy of the port (added with this review): it checks, passes every package case and agrees with the accepted C++, as recorded by `corpus.py verify`. The ideal beyond today's language is below. The port is the ideal: std `dijkstra` over the `(Row, Col)` cells, with each cell's moves given by `p.adj4` and the cost of the cell entered. No graph is built.

## Dream

The port as written.

There are no dream/port gaps.

## Measurements

- **Tokens:** port 129, accepted C++ 550 (comments excluded).
- **Runtime and memory on gen.py --max:** port 0.18 s / 20 MB, accepted C++ 0.47 s / 12 MB, with the same output.
- **Emitted C++:** 24516 bytes.
- **Agreement:** `cpl stress` against the accepted C++ agrees on 30 gen.py cases at size 30.
