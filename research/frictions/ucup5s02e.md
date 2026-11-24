# Friction report: ucup5s02e

`dream.cp` is verified. The port differs from it in one place, and is the better spelling:
- the dream loops `for z in 0..3: for y in z + 1..3:` and indexes `rank[y]` and `rank[z]`;
- the port writes `for (lo, hi) in rank.pairs():` over the three rankings.

## Dream

`dream.cp`, with `rank.pairs()` in place of the index pair, which the port shows compiles. There is no dream/port gap; the dream should take the port's loop when it is next edited. A dream edit needs `corpus.py verify`, so it is left for that change.

## Measurements

- **Tokens:** port 124, accepted C++ 313 (comments excluded).
- **Runtime and memory on gen.py --max:** port 0.05 s / 15 MB, accepted C++ 0.06 s / 7 MB, with the same output.
- **Emitted C++:** 16906 bytes.
- **Agreement:** `cpl stress` against the accepted C++ agrees on 30 gen.py cases at size 30.
