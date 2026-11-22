# Corpus ports report: 76 problems re-solved in `proto/cpc`

76 accepted C++ submissions from `corpus/` are re-solved in CP Lang in
`proto/ports/`: 30 from the first two rounds, 8 that the scout sprint moved
up from its sketches, and 38 added in the third round. All 76 compile
with the prototype, and all 76 agree with their original on every
generated input that satisfies the port's declared preconditions. Another
15 ports live in `proto/ports/hillclimb_dp_graph/` with their own README;
`run_all.sh` and the benchmark cover them too, which makes 91 ports in
all. What the ports taught us about the language is in
[`friction.md`](friction.md).

The 76 ports total 1487 lines against 5798 lines of original C++, a raw
reduction of 74%. Against the originals with everything a perfect C++
contest library would have supplied removed (2807 lines), the reduction is
47%. Those figures count the whole port, including 91 lines of test
generators and precondition helpers that the originals do not have; without
them the reductions are 76% and 50%.

On the largest inputs the ports' own generators produce, the emitted C++
runs at a median of 0.85x the original's time (over the 29 ports whose
original needs at least 0.05 s) and a median of 1.04x its peak memory. The
numbers, and the ports that are off the median, are in
[Runtime, memory and emitted C++](#runtime-memory-and-emitted-c).

## Where things are

For a port named `<name>`:

- `proto/ports/<name>.cp` is the port.
- `proto/ports/<name>_orig.cpp` is the accepted submission, copied from
  `corpus/submissions/<id>.cpp`. It is the oracle for output.
- `proto/ports/<name>.cpp` is the C++17 the prototype emitted.
- `proto/ports/<name>.strip` lists the line ranges of the original that
  count as library, each with the facility it stands for, so the baseline
  can be argued with.
- `proto/ports/<name>_judge.py` is the judge of an interactive port
  (`aac5p2`, `yac6p3`, `yac7p5`).

The `proto/ports/scout_batch_*/` folders hold design sketches from scout
passes; they are not regression ports and are not counted here.

To reproduce everything:

```sh
cd proto
ITERS=200 bash run_all.sh               # every port against its original, plus the compiler's own tests
python3 loc.py                          # both line counts per port and overall
python3 ../tools/bench/bench.py         # lines, runtime, peak RSS and emitted C++ per port
```

## How the lines are counted

Blank lines and comment-only lines never count, on either side, so
commented-out code in an original does not inflate it. The library-stripped
original removes the template header (includes, debug macros, typedefs),
the `main` that only sets up fast I/O and loops over test cases, and
hand-written standard structures. For example `noi16p4.strip` reads:

```
1-30         template        includes, debug macros, typedefs
32-62        segtree         lazy segment tree: range add, global max
72-82        compress        coordinate compression
106-111      driver          fast I/O setup and the test-case loop
```

Two choices lean against the ports on purpose. A structure is stripped from
the original even where the port writes its own (the weighted union-find in
`noi02p1`, the binomials in `coci20c6p3`, the prefix function in
`noi14p4`). And the port's own test generators count as port lines.

## How the ports are tested

`cp.py stress` builds the port and the original, generates random inputs
from the port's declared input types (or from a generator the port names
with `read(gen: f)`), and compares the outputs token by token. An input
that fails a `where` precondition is rejected and not counted, which is
why some rows below show fewer than 200 inputs. The run below used 200
generated inputs per port at size 6. The three interactive ports run
against a judge instead (`interact.py`): the port and the original each
hold the conversations and both must be accepted.

Beyond that run, every port added in the third round passed at least 100
further inputs while it was written, most of them at a second size as well
(up to size 400 for `noip04p1`, 100000 for `cco17p1` and `noip00p1`, and
10^9 for `ccc23s5`). `tools/bench` then checks agreement again on the
largest inputs each generator produces.

Two limits of this evidence. `ccc10s4` accepts only about one generated
input in ten, because its generator cannot keep random polygons valid, so
its row rests on about 20 inputs per run. And the judges of `yac6p3` and
`yac7p5` were written from the accepted originals: they check the final
answer and a question budget the original stays within, not the contest's
exact scoring.

Not tested: inputs at the maximum declared size where the generator cannot
reach it, and the emitted C++ under ASan/UBSan (the red-team suite does
that for the compiler's own tests, not for the ports).

## Results per port

3 ports do not beat their library-stripped original, which the
validation plan counts as friction: `apio07p1`, `ccc21s2`, `noi02p1`.
`apio07p1` has 25 of its 47 lines in a tree generator and its
precondition. `noi02p1` writes a weighted union-find that the stripped
original gets for free. `ccc21s2` spends its lines on an enum and a `match`
where the original has one `if`/`else`.

Rows marked (new) are the third round; rows marked (scout sprint) were
moved up from `scout_batch_1`. `ccc10s4`'s original is marked "not
my code" in the corpus; it is a valid oracle, but its line counts say less
about the corpus author's style.

| port | corpus submission | kind | port lines | original | raw | original, stripped | stripped | compiles | against the original |
|---|---|---|---:|---:|---:|---:|---:|---|---|
| `aac4p2` | `4140241.cpp` | binary search, math | 14 | 66 | 79% | 31 | 55% | yes | 200 random inputs agree |
| `aac5p2` | `4239718.cpp` | interactive | 24 | 87 | 72% | 52 | 54% | yes | 200 conversations accepted for both |
| `apio07p1` | `5532655.cpp` | trees | 47 | 73 | 36% | 42 | -12% | yes | 200 random inputs agree |
| `btoi17p3` | `4456180.cpp` | trees (LCA) | 29 | 141 | 79% | 84 | 65% | yes | 200 random inputs agree |
| `btoi17p6` | `4456183.cpp` | trees, greedy | 16 | 71 | 77% | 40 | 60% | yes | 200 random inputs agree |
| `btoi18p6` | `4456176.cpp` | graphs, bitmask DP | 19 | 74 | 74% | 40 | 52% | yes | 200 random inputs agree |
| `ccc03s4` (new) | `4182950.cpp` | strings (hashing), multitest | 12 | 109 | 89% | 29 | 59% | yes | 200 random inputs agree |
| `ccc10s4` (new) | `7171512.cpp` | graphs (MST), planar faces | 24 | 91 | 74% | 68 | 65% | yes | 25 random inputs agree |
| `ccc17s4` (new) | `4899340.cpp` | graphs (MST), union-find | 24 | 104 | 77% | 50 | 52% | yes | 156 random inputs agree |
| `ccc18s4` (new) | `4329704.cpp` | math (floor-division blocks), memo | 13 | 85 | 85% | 42 | 69% | yes | 200 random inputs agree |
| `ccc18s5` (scout sprint) | `4329864.cpp` | graphs (MST), union-find | 25 | 104 | 76% | 48 | 48% | yes | 200 random inputs agree |
| `ccc19s5` (new) | `5119937.cpp` | range structure (triangle maxima) | 18 | 64 | 72% | 34 | 47% | yes | 117 random inputs agree |
| `ccc20s2` (new) | `4323590.cpp` | graphs (BFS) | 12 | 73 | 84% | 38 | 68% | yes | 200 random inputs agree |
| `ccc20s3` | `4323643.cpp` | strings (hashing) | 11 | 109 | 90% | 29 | 62% | yes | 200 random inputs agree |
| `ccc21s2` | `4323443.cpp` | implementation | 21 | 54 | 61% | 19 | -11% | yes | 200 random inputs agree |
| `ccc21s3` (new) | `4323533.cpp` | math (convex search) | 7 | 61 | 89% | 26 | 73% | yes | 200 random inputs agree |
| `ccc21s4` (scout sprint) | `4323468.cpp` | graphs (BFS), minimum under swaps | 22 | 85 | 74% | 51 | 57% | yes | 107 random inputs agree |
| `ccc21s5` | `4323513.cpp` | segment tree, math | 21 | 95 | 78% | 44 | 52% | yes | 128 random inputs agree |
| `ccc22s4` (scout sprint) | `4371347.cpp` | geometry (circle), prefix sums | 17 | 58 | 71% | 32 | 47% | yes | 200 random inputs agree |
| `ccc23s4` | `5350808.cpp` | graphs (Dijkstra) | 20 | 109 | 82% | 32 | 38% | yes | 200 random inputs agree |
| `ccc23s5` (new) | `5350810.cpp` | math (Cantor set), recursion | 21 | 70 | 70% | 35 | 40% | yes | 200 random inputs agree |
| `cco16p2` | `5566900.cpp` | geometry | 31 | 114 | 73% | 51 | 39% | yes | 139 random inputs agree |
| `cco17p1` (new) | `3606070.cpp` | constructive (graphs) | 19 | 68 | 72% | 31 | 39% | yes | 200 random inputs agree |
| `cco18p1` (scout sprint) | `4801863.cpp` | DP (pairing two sequences) | 18 | 55 | 67% | 26 | 31% | yes | 200 random inputs agree |
| `cco20p1` (scout sprint) | `4561351.cpp` | geometry, sweep | 14 | 72 | 81% | 43 | 67% | yes | 200 random inputs agree |
| `cco24p1` | `6435479.cpp` | graphs (shortest paths) | 19 | 74 | 74% | 42 | 55% | yes | 200 random inputs agree |
| `ceoi09p3` | `4827241.cpp` | DP | 29 | 80 | 64% | 49 | 41% | yes | 200 random inputs agree |
| `ceoi18p1` | `5323668.cpp` | DP (knapsack) | 18 | 68 | 74% | 37 | 51% | yes | 200 random inputs agree |
| `cheerio1s3` (new) | `5338043.cpp` | geometry (slopes) | 20 | 62 | 68% | 31 | 35% | yes | 200 random inputs agree |
| `coci13c1p3` | `5245417.cpp` | 2-D prefix sums | 14 | 87 | 84% | 35 | 60% | yes | 200 random inputs agree |
| `coci20c6p3` | `4188081.cpp` | DP, math/mod, strings | 29 | 89 | 67% | 41 | 29% | yes | 200 random inputs agree |
| `dmopc21c2p1` | `3946478.cpp` | greedy, sorting | 14 | 58 | 76% | 21 | 33% | yes | 200 random inputs agree |
| `dmopc21c9p4` | `4565958.cpp` | grids, game | 49 | 97 | 49% | 66 | 26% | yes | 200 random inputs agree |
| `dmpg16s5` (new) | `5323572.cpp` | range structure (segment tree), prefix sums | 14 | 58 | 76% | 27 | 48% | yes | 117 random inputs agree |
| `dpe` (new) | `4131644.cpp` | DP (knapsack by value) | 10 | 56 | 82% | 21 | 52% | yes | 200 random inputs agree |
| `dpg` (new) | `4131646.cpp` | graphs (DAG longest path), memo DP | 13 | 67 | 81% | 32 | 59% | yes | 129 random inputs agree |
| `dph` (new) | `4131700.cpp` | DP (grid paths) | 17 | 59 | 71% | 24 | 29% | yes | 200 random inputs agree |
| `dpk` (new) | `4131775.cpp` | game DP | 9 | 59 | 85% | 21 | 57% | yes | 200 random inputs agree |
| `dpl` (new) | `4131776.cpp` | interval DP, game | 8 | 51 | 84% | 16 | 50% | yes | 200 random inputs agree |
| `dpm` (new) | `4131778.cpp` | DP, prefix sums | 13 | 67 | 81% | 32 | 59% | yes | 200 random inputs agree |
| `dpn` (new) | `4131779.cpp` | interval DP | 9 | 57 | 84% | 22 | 59% | yes | 200 random inputs agree |
| `dpr` (new) | `4131862.cpp` | math (matrix power) | 17 | 85 | 80% | 22 | 23% | yes | 200 random inputs agree |
| `dpt` (new) | `4131863.cpp` | DP, prefix sums | 23 | 72 | 68% | 34 | 32% | yes | 200 random inputs agree |
| `dpx` (new) | `4133420.cpp` | DP, greedy order | 9 | 59 | 85% | 24 | 62% | yes | 200 random inputs agree |
| `dpy` (new) | `4133511.cpp` | math (binomials, inclusion-exclusion) | 26 | 95 | 73% | 30 | 13% | yes | 97 random inputs agree |
| `dpz` (new) | `4133674.cpp` | DP (convex hull trick) | 24 | 93 | 74% | 27 | 11% | yes | 200 random inputs agree |
| `ioi00p1` (new) | `4819647.cpp` | strings, DP | 12 | 53 | 77% | 22 | 45% | yes | 200 random inputs agree |
| `ioi04p4` (scout sprint) | `4501742.cpp` | DP (guillotine cuts) | 12 | 35 | 66% | 29 | 59% | yes | 200 random inputs agree |
| `ioi05p6` | `5032744.cpp` | tree DP | 41 | 78 | 47% | 47 | 13% | yes | 200 random inputs agree |
| `ioi10p3` (scout sprint) | `4692375.cpp` | binary search, 2-D prefix sums | 11 | 76 | 86% | 42 | 74% | yes | 101 random inputs agree |
| `ioi11p2` (scout sprint) | `4848359.cpp` | trees (small-to-large merging) | 22 | 84 | 74% | 50 | 56% | yes | 200 random inputs agree |
| `ioi94p1` (new) | `4810274.cpp` | DP | 11 | 48 | 77% | 17 | 35% | yes | 200 random inputs agree |
| `mccc4j5` | `4264954.cpp` | graphs (heap) | 20 | 73 | 73% | 36 | 44% | yes | 200 random inputs agree |
| `nccc10s2` | `4214203.cpp` | graphs, memo DP | 12 | 78 | 85% | 43 | 72% | yes | 200 random inputs agree |
| `noi01p5` | `5079272.cpp` | meet in the middle, hashing | 15 | 103 | 85% | 29 | 48% | yes | 200 random inputs agree |
| `noi02p1` | `4793930.cpp` | union-find | 37 | 69 | 46% | 28 | -32% | yes | 200 random inputs agree |
| `noi14p4` | `3821774.cpp` | strings (KMP), multitest | 24 | 88 | 73% | 69 | 65% | yes | 200 random inputs agree |
| `noi16p4` | `4791678.cpp` | segment tree, two pointers | 29 | 98 | 70% | 30 | 3% | yes | 200 random inputs agree |
| `noip00p1` (new) | `5067849.cpp` | math (negative base) | 9 | 70 | 87% | 28 | 68% | yes | 200 random inputs agree |
| `noip01p2` (new) | `5086680.cpp` | DP (partitions) | 8 | 47 | 83% | 16 | 50% | yes | 200 random inputs agree |
| `noip04p1` (new) | `5117097.cpp` | simulation | 12 | 46 | 74% | 15 | 20% | yes | 200 random inputs agree |
| `noip04p2` (new) | `5117099.cpp` | greedy (heap) | 10 | 55 | 82% | 22 | 55% | yes | 200 random inputs agree |
| `noip04p3` (new) | `5117143.cpp` | DP (LIS) | 10 | 67 | 85% | 16 | 38% | yes | 200 random inputs agree |
| `noip06p2` (new) | `5165705.cpp` | DP (grouped knapsack) | 21 | 75 | 72% | 41 | 49% | yes | 143 random inputs agree |
| `noip07p1` (new) | `5184805.cpp` | sorting | 4 | 46 | 91% | 15 | 73% | yes | 200 random inputs agree |
| `noip07p2` (new) | `5184882.cpp` | strings | 20 | 80 | 75% | 49 | 59% | yes | 200 random inputs agree |
| `noip09p2` | `5184950.cpp` | number theory, multitest | 52 | 129 | 60% | 60 | 13% | yes | 200 random inputs agree |
| `noip09p3` | `5185139.cpp` | graphs | 33 | 99 | 67% | 66 | 50% | yes | 200 random inputs agree |
| `noip10p3` (new) | `5445663.cpp` | graphs, union-find | 14 | 89 | 84% | 58 | 76% | yes | 86 random inputs agree |
| `noip14p2` (new) | `5445666.cpp` | trees, math/mod | 25 | 66 | 62% | 35 | 29% | yes | 200 random inputs agree |
| `noip14p3` | `5445668.cpp` | DP | 45 | 92 | 51% | 61 | 26% | yes | 200 random inputs agree |
| `noip14p5` (new) | `5445670.cpp` | graphs (BFS) | 16 | 90 | 82% | 59 | 73% | yes | 200 random inputs agree |
| `noip18p2` | `5032734.cpp` | DP, multitest | 15 | 71 | 79% | 32 | 53% | yes | 200 random inputs agree |
| `yac1p1` | `4376122.cpp` | math/mod | 16 | 64 | 75% | 33 | 52% | yes | 200 random inputs agree |
| `yac6p3` (new) | `5149597.cpp` | interactive, geometry | 19 | 61 | 69% | 31 | 39% | yes | 165 conversations accepted for both |
| `yac7p5` (new) | `5705290.cpp` | interactive, divide and conquer | 29 | 83 | 65% | 57 | 49% | yes | 200 conversations accepted for both |
| **all 76** | | | 1487 | 5798 | 74% | 2807 | 47% | 76 of 76 | |

## Runtime, memory and emitted C++

`tools/bench/bench.py` builds every port and its original with `g++
-std=c++17 -O2` and runs both on the same generated inputs: the largest
size cap at which the original still answers within two seconds, three
accepted seeds per port (see [`tools/bench/README.md`](../tools/bench/README.md)).
The full table is [`tools/bench/results.md`](../tools/bench/results.md), the
raw numbers `results.json` beside it. Figures below are the worst of the
measured inputs for each side, on one shared machine, so differences of a
few milliseconds mean nothing.

**Agreement.** All 91 ports (76 in `proto/ports/` and the 15 in
`hillclimb_dp_graph/`) agree with their original on every measured input
(88 by output, 3 interactive ports by judge).

**Runtime.** Many of these problems are small: for 59 of the 88 timed
ports the original finishes in under 0.05 s on the largest input the
declared limits allow, and both sides measure process start-up. Over the
29 ports where the original needs at least 0.05 s, the port takes a median
of 0.85x the original's time, from 0.12x to 26.11x.
Slower by more than 30%: `ioi04p4` (1.51 s against 0.06 s), `dpn` (0.03 s against 0.01 s), `ccc03s4` (1.61 s against 0.38 s), `hillclimb_dp_graph/dpp` (0.05 s against 0.01 s), `cco20p1` (0.04 s against 0.02 s).
Faster by more than 20%: `ccc20s3` (0.01 s against 0.05 s), `dpz` (0.01 s against 0.07 s), `ccc21s4` (0.04 s against 0.19 s), `noi14p4` (0.03 s against 0.07 s), `noip07p1` (0.03 s against 0.06 s).
Part of every speed-up is input and output: the emitted C++ reads and
writes through the runtime's own buffers, the originals through iostream.
The slower ports have not all been explained; `friction.md` has what is
known.

**Peak RSS.** Median 1.04x the original, from 0.11x to 3.89x.
More than 25% above: `hillclimb_dp_graph/dmopc21c5p1` (29 MB against 7 MB), `ioi04p4` (12 MB against 4 MB), `hillclimb_dp_graph/dpp` (20 MB against 8 MB), `hillclimb_dp_graph/dpv` (20 MB against 9 MB), `cco20p1` (12 MB against 5 MB).
More than 20% below: `ccc10s4` (4 MB against 34 MB), `hillclimb_dp_graph/dps` (4 MB against 20 MB), `noi14p4` (18 MB against 75 MB), `ioi05p6` (4 MB against 12 MB), `ccc21s3` (5 MB against 15 MB).

**Emitted C++.** The generated `.cpp` is between 25.0 KB and 36.7 KB
(median 27.4 KB), most of it the runtime prelude; the 64 KB budget of
the validation plan holds for every port. The stripped `-O2` binary is a
median of 1.28x the original's, from 0.54x to 2.42x.

What explains the outliers is in [`friction.md`](friction.md) under "What
the benchmark found".

## Coverage

DP: `ceoi09p3`, `ceoi18p1`, `noip14p3`, `noip18p2`, `coci20c6p3`,
`btoi18p6` (bitmask), `ioi05p6` (tree), `nccc10s2` (memo); new: `dpe`,
`dph`, `dpk`, `dpl`, `dpm`, `dpn`, `dpt`, `dpx`, `dpz` (convex hull trick),
`ioi94p1`, `noip01p2`, `noip04p3`, `noip06p2`, `ioi00p1`; scout sprint:
`cco18p1`, `ioi04p4`.
Graphs: `ccc23s4`, `cco24p1`, `noip09p3`, `mccc4j5`, `btoi18p6`; new: `dpg`
(DAG), `ccc17s4` and `ccc10s4` (spanning trees), `ccc20s2` and `noip14p5`
(BFS), `noip10p3`; scout sprint: `ccc18s5`, `ccc21s4`.
Trees: `btoi17p3`, `btoi17p6`, `apio07p1`, `ioi05p6`; new: `noip14p2`;
scout sprint: `ioi11p2`.
Range structures: `ccc21s5` (segment tree), `noi16p4` (lazy segment tree),
`coci13c1p3` (2-D prefix sums); new: `dmpg16s5` (segment tree over prefix
sums), `ccc19s5` (triangle maxima by growing windows), `dpm` (prefix
sums); scout sprint: `ioi10p3`. There is still no Fenwick-tree port.
Strings: `ccc20s3`, `noi14p4`, `coci20c6p3`; new: `ccc03s4` (hashing),
`noip07p2`, `ioi00p1`.
Geometry: `cco16p2`; new, on integers: `cheerio1s3` (slopes), `yac6p3`;
scout sprint: `ccc22s4` (points on a circle), `cco20p1` (cones as
intervals).
Math and modular arithmetic: `yac1p1`, `noip09p2`, `aac4p2`,
`coci20c6p3`, `noi01p5` (meet in the middle); new: `dpr` (matrix power),
`dpy` (binomials), `ccc18s4`, `ccc21s3`, `ccc23s5`, `noip00p1`.
Constructive: new: `cco17p1`.
Greedy, sorting and simulation: `dmopc21c2p1`, `btoi17p6`, `noi16p4`; new:
`noip04p2`, `noip07p1`, `noip04p1`, `dpx`.
Interactive: `aac5p2`; new: `yac6p3`, `yac7p5`.
Multitest: `noip18p2`, `noi14p4`, `noip09p2`; new: `ccc03s4`.
Union-find: `noi02p1`; new: `ccc17s4`, `ccc10s4`, `noip10p3`; scout
sprint: `ccc18s5`.

Not covered, and why: floating point (the prototype has none, which
excluded `dpi`, `dpj`, `ccc05s5` and `cco18p4`), input that ends at a
sentinel or at end of file (`ccc04s5`, `noip99p1`), and grader-style
originals.

## What changed in the prototype

Language and library, each driven by a port: `x => e` lambdas as the plan
specifies; `sort`/`sorted` with `by:` or `less:`; `Subset<D>`; `min`/`max`
over optionals; `List.pop()` returning `?T`; `gcd`, `lcm`;
`D.first_where` and `D.partition_point`; `SegTree<D, T>`; `AddMax<D>`;
`let s: [domain S]Char = read()`; `StrHash`; `Rooted(adj, root)` with
`lca`; `runs`, `distinct`; reverse iteration over lists; `?T == T`;
counting into a map with `+=`; `ask(...)` for interactive problems.

Third round: `read(alphabet: "...")` and `where` on a word read; a fix
that made `x /= b` and `x %= b` Euclidean like `/` and `%`; and a fix for a
compiler crash on two effects in one expression. `crates/cpl-cli/tests/cases/ergo/`
(`compound_div`, `effects_first`) pins them. `Dsu`, `bfs` and `subsets`,
which this round also needed, came from the scout sprint's library.

Tooling: `proto/interact.py` (judge harness, which now skips inputs a
`where` rejects), `proto/loc.py` (line counts), `tools/bench/` (this
report's measurements), and `run_all.sh` covers interactive ports and
raises the stack limit, because some originals keep megabytes of tables on
the stack.
