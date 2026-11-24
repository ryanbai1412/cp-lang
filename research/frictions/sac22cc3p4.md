# Friction report: sac22cc3p4

There is no dream.cp. The port is the ideal: the queries run backwards, so cuts become `Dsu` unions, and a connected pair's answer is `tree.dist(u, v)` in the original tree, rooted at 1.

## Dream

The port as written.

There are no dream/port gaps. `answers: List<?Int>` collected in reverse and printed `a ?? -1` keeps the "not connected" case typed until output.

## Measurements

- **Tokens:** port 165, accepted C++ 1022 (comments excluded).
- **Runtime and memory on gen.py --max:** port 0.05 s / 15 MB, accepted C++ 0.17 s / 45 MB, with the same output.
- **Emitted C++:** 24327 bytes.
- **Agreement:** `cpl stress` against the accepted C++ agrees on 30 gen.py cases at size 30.
