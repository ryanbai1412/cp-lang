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
- **Memory gap:** the port's RSS at `--max` is over 2x the accepted C++'s (above). The repro (this package's `--max` case) is sent to perf via Factory 7bd146f2. Owner: Perf B (33c55557). **Status:** fixed: now 1.02x (below).

## Tuned twin retired (2026-11-24)

The plain port is within 1.1x of the accepted C++ on `gen.py --max` on both counts (best of 9, CPU time and peak RSS, `scripts/corpus.py`'s judge builds): 0.169 s / 11.5 MB against 0.169 s / 11.2 MB (1.0x time, 1.02x memory), with the same output, so `perf.cp` is deleted (performance/R6 keeps a twin only over 1.1x).
