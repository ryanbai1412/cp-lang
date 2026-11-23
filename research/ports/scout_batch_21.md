# Scout batch 21

Ten geometry, sweepline, probability and expected-value submissions from
the corpus, re-solved in CP Lang. This is a Stage-1 critique pass. Nothing
in `proto/cpc/` or `crates/` was changed for it. What the ports taught us
is in
[`research/archive/frictions/friction-backlog-batch-21.md`](../../../research/archive/frictions/friction-backlog-batch-21.md).

Every port runs today on the reference interpreter (`proto/cp.py
lib-run`), which has `Float`, and `check.py` compares each one with its
original on small random inputs:

```
$ python3 proto/ports/scout_batch_21/check.py        # every port, 60 inputs each
$ python3 proto/ports/scout_batch_21/check.py ioi07p5 300
$ python3 proto/cp.py test proto/ports/scout_batch_21/float.cp
```

The four exact ports and `sac22cc4jp3` must print the same answer as the
original. The five that print reals are compared with a tolerance, because
the two programs do not print the same digits. The inputs are small (the
sizes are in `check.py`); no port was run at the problem's largest size or
timed, and none was built with `proto/cpc`, which has no `Float`.

`float.cp` is a small library (`lib.toml` names it `real`) with what the
ports needed and the interpreter does not have: printing with a fixed
number of decimals, comparison with a tolerance, and sine and cosine. It
is ordinary CP Lang, and it is the proposal of backlog items E1, E3 and
E5.

`proto/run_all.sh` only looks at `proto/ports/*.cp`, so this folder does
not affect it.

## The ports

Each port names its original on its `// original:` line.

Exact integer geometry and sweeps:

- `ioi07p5.cp`: Pairs. Manhattan balls become boxes after a 45 degree
  turn: a window over a sorted line, a window with a Fenwick tree on the
  grid, and 2D prefix sums per layer in the cube (the original keeps a 3D
  Fenwick tree).
- `ccc14s4.cp`: Tinted Glass Window. Compressed difference grid instead of
  the original's sweep over a sqrt-decomposed axis.
- `ioi08p6.cp`: Pyramid Base. Two sweeps over x: range add with a global
  minimum under a binary search, and a sliding window over a hand-written
  "how much is covered" tree.
- `nccc10s4.cp`: polygons with one side on each of n lines. Crossings are
  whole-number triples (homogeneous coordinates), so the original's
  fraction class is gone and nothing is divided.

`Float`:

- `sac22cc4jp3.cp`: closest direction to an aim, with `atan2`.
- `cmt21peter2.cp`: sums of distances, estimated from shadows on 30
  directions.
- `sdoi17r2p1.cp`: distribution of a sum of dice. A window over prefix
  sums replaces the original's polynomial powers and its `1e-5` cut-off.
- `noi07p2.cp`: Cash. A Li Chao tree over the days in order of price
  ratio replaces the original's dynamic hull.
- `apio23p1.cp`: Cyberland. Layers of the library's `dijkstra` over
  `?Float`; the original's `1e18` and `1e-9` are gone. The original is a
  grader function, so `check.py` gives it a `main`.
- `dpj.cp`: Sushi. Expected steps, filled bottom-up.

## What was left out

`ccc00s5` and `cheerio1s3` were ported first by batch 16 and by
`proto/ports/`, so they were dropped. The corpus has few geometry
problems, and none of these ten needs a convex hull, a polar sort of
points or a segment-crossing test, so those helpers are discussed in the
backlog from `stdlib/geometry.cp` and not from a port.
