# Scout batch 3

Ten string, number-theory, geometry and interactive submissions from the
corpus, re-solved the way we would like to write them. This is a Stage-1
critique pass, like [`scout_batch_1/`](../scout_batch_1/): the files in
this folder are design sketches, and every line the prototype rejects
today is marked `// WISH Gn` (an item in
[`research/archive/frictions/friction-backlog-batch-3.md`](../../../research/archive/frictions/friction-backlog-batch-3.md))
or `// WISH Fn` (an item of batch 1 that came up again, in
[`research/archive/frictions/friction-backlog.md`](../../../research/archive/frictions/friction-backlog.md)).

None of the ten sketches builds with `proto/cpc` yet. What is different
from batch 1 is the [`today/`](today/) folder: for nine of the ten it holds
the same solution in the form the prototype accepts now, and those files
were built and run. They are where the "today" snippets in the backlog
come from, and they give a Stage-2 sprint something to compare a sketch
with once its items land.

`proto/run_all.sh` only looks at `proto/ports/*.cp`, so nothing here
affects it. `proto/cpc/` and `crates/` were not touched.

## The ten

Each line gives the original in `corpus/submissions/`, the idea, and how
the `today/` form was checked. "Stress OK" means `proto/cp.py stress`
against the original at commit `c217120`: 100 generated inputs of size 6,
and a second run of 40 to 100 inputs at a larger size (40 for the strings
and the points, 30 for the graph, 1e5 for modinv, up to 9e8 for ccc18s4).

- `ccc03s4.cp` from `4182950.cpp`: distinct substrings from the sorted
  suffixes and their common prefixes. Today: one hash set per length;
  stress OK.
- `noip99p2.cp` from `5044551.cpp`: add a number to its mirror image in
  base b until it is a palindrome. Today: stress OK.
- `coci06c1p6.cp` from `4183051.cpp`: largest square of a bit grid that is
  unchanged by half a turn, by a 2-D hash. Today: built, and agrees with
  the original on 500 random grids up to 8 x 8 fed by a script, because the
  typed generator cannot produce this input (G8).
- `modinv.cp` from `5034440.cpp`: modular inverse of numbers below 2^64.
  Today: `Int` only, so inputs up to 1e18; stress OK on coprime inputs.
- `aac4p3.cp` from `4140246.cpp`: pairs of points on a common line with one
  of the given slopes. Today: stress OK.
- `ccc18s4.cp` from `4329704.cpp`: a memoised count over floor quotients.
  Today: stress OK.
- `dpr.cp` from `4131862.cpp`: walks of length k by a matrix power. Today:
  stress OK.
- `noip01p4.cp` from `5086731.cpp`: fourth corner of a rectangle, then
  all-pairs shortest paths over airports with real-valued prices. No
  `today/` form: every price is a float (F15).
- `cco04p4.cp` from `5429989.cpp`: closest approach of two craft that
  repeat a list of moves. Today: built; it prints the least squared
  distance, since there are no floats, and on 800 inputs made by a script
  the root of that, to two decimals, is what the original prints.
- `cco09p4.cp` from `5034519.cpp`: interactive matching of caps to bottles
  by a two-sided quicksort. Today: `proto/interact.py` with
  `today/cco09p4_judge.py`, 60 conversations of size 8 accepted for both
  the port and the original.

To repeat a check, build from a copy, since the build stamps its source:

```
cp proto/ports/scout_batch_3/today/ccc18s4.cp /tmp/ccc18s4.cp
python3 proto/cp.py stress /tmp/ccc18s4.cp corpus/submissions/4329704.cpp 100 6
```

## Shape changes to look at first

Three sketches do not follow their original, and are the ones to read
with care when they are first stressed. `ccc03s4.cp` replaces the
original's double rolling hash and open-addressing table with a suffix
array. `noip99p2.cp` never reverses a number: the column sums of a number
and its mirror image read the same from both ends, so the order of the
digits does not matter (the `today/` form does the same and agrees with
the original). `cco04p4.cp` takes one square root at the end instead of
one per tick.

## Size

Lines of code, with blank and comment lines left out, as sketch / today /
original. The original's count is the whole accepted file, including the
author's template and library code, so it is the raw measure only; the
library-stripped measure needs a `.strip` file per port and was not done
in this pass.

ccc03s4 7 / 11 / 109, noip99p2 20 / 24 / 75, coci06c1p6 13 / 23 / 128,
modinv 5 / 7 / 50, aac4p3 6 / 17 / 61, ccc18s4 7 / 13 / 85, dpr 11 / 16 /
85, noip01p4 23 / none / 87, cco04p4 21 / 30 / 92, cco09p4 13 / 24 / 107.
