# Corpus ports report: 30 problems re-solved in `proto/cpc`

Thirty accepted C++ submissions from `corpus/` are re-solved in CP Lang: 11
from the earlier round and 19 added in this one. All 30 compile with the
prototype, and all 30 agree with their original on every generated input
that satisfies the port's declared preconditions. What the ports taught us
about the language is in [`friction.md`](friction.md).

The 30 ports total 764 lines against 2588 lines of original C++, a raw
reduction of 70%. Against the originals with everything a perfect C++
contest library would have supplied removed (1289 lines), the reduction is
41%. Those figures count the whole port, including 70 lines of test
generators and precondition helpers that the originals do not have; without
them the reductions are 73% and 46%. (Before the rewrites of
[`ergonomics-audit.md`](ergonomics-audit.md) the ports were 785 lines.)

## Where things are

For a port named `<name>`:

- `proto/ports/<name>.cp` is the port.
- `proto/ports/<name>_orig.cpp` is the accepted submission, copied from
  `corpus/submissions/<id>.cpp`. It is the oracle for output.
- `proto/ports/<name>.cpp` is the C++17 the prototype emitted.
- `proto/ports/<name>.strip` lists the line ranges of the original that
  count as library, each with the facility it stands for, so the baseline
  can be argued with.
- `proto/ports/aac5p2_judge.py` is the judge for the one interactive port.

To reproduce everything:

```sh
cd proto
ITERS=200 bash run_all.sh     # every port against its original, plus the must-not-compile tests
python3 loc.py                # both line counts per port and overall
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
that fails a `where` precondition is rejected and not counted. The run
below used 200 inputs per port at size 6. The interactive port runs against
a judge instead (`interact.py`): the port and the original each hold 200
conversations and both must be accepted.

Beyond that run, each port added this round passed at least 100 further
inputs while it was written, and several were stressed at other sizes: up
to size 30 for `noip14p3`, size 14 for `noi14p4`, size 12 for `noi16p4`,
and 1500 inputs at each of four sizes for `cco16p2`.

Not tested: inputs at the maximum declared size, running time and memory
against the original, and the emitted C++ under ASan/UBSan. Those belong to
the red-team suite, which is a separate piece of work.

## Results per port

Three ports do not beat their library-stripped original, which the
validation plan counts as friction. `apio07p1` has 25 of its 47 lines in a
tree generator and its precondition. `noi02p1` writes a weighted union-find
that the stripped original gets for free. `ccc21s2` spends its lines on an
enum and a `match` where the original has one `if`/`else`. The smallest wins are
`noi16p4` (3%: the original is already mostly library) and `ioi05p6` and
`noip09p2` (13% each, both carrying generators).

| port | corpus submission | kind | port lines | original | raw | original, stripped | stripped | compiles | against the original |
|---|---|---|---:|---:|---:|---:|---:|---|---|
| `aac4p2` | `4140241.cpp` | binary search, math | 14 | 66 | 79% | 31 | 55% | yes | 200 random inputs agree |
| `aac5p2` | `4239718.cpp` | interactive | 24 | 87 | 72% | 52 | 54% | yes | 200 conversations accepted for both the port and the original |
| `apio07p1` (earlier round) | `5532655.cpp` | trees | 47 | 73 | 36% | 42 | -12% | yes | 200 random inputs agree |
| `btoi17p3` | `4456180.cpp` | trees (LCA) | 29 | 141 | 79% | 84 | 65% | yes | 200 random inputs agree |
| `btoi17p6` | `4456183.cpp` | trees, greedy | 16 | 71 | 77% | 40 | 60% | yes | 200 random inputs agree |
| `btoi18p6` | `4456176.cpp` | graphs, bitmask DP | 19 | 74 | 74% | 40 | 52% | yes | 200 random inputs agree |
| `ccc20s3` | `4323643.cpp` | strings (hashing) | 11 | 109 | 90% | 29 | 62% | yes | 200 random inputs agree |
| `ccc21s2` (earlier round) | `4323443.cpp` | implementation | 21 | 54 | 61% | 19 | -11% | yes | 200 random inputs agree |
| `ccc21s5` | `4323513.cpp` | segment tree, math | 21 | 95 | 78% | 44 | 52% | yes | 128 random inputs agree (72 more rejected by the `where` precondition) |
| `ccc23s4` (earlier round) | `5350808.cpp` | graphs (Dijkstra) | 20 | 109 | 82% | 32 | 38% | yes | 200 random inputs agree |
| `cco16p2` | `5566900.cpp` | geometry | 31 | 114 | 73% | 51 | 39% | yes | 139 random inputs agree (61 more rejected by the `where` precondition) |
| `cco24p1` | `6435479.cpp` | graphs (shortest paths) | 19 | 74 | 74% | 42 | 55% | yes | 200 random inputs agree |
| `ceoi09p3` (earlier round) | `4827241.cpp` | DP | 29 | 80 | 64% | 49 | 41% | yes | 200 random inputs agree |
| `ceoi18p1` | `5323668.cpp` | DP (knapsack) | 18 | 68 | 74% | 37 | 51% | yes | 200 random inputs agree |
| `coci13c1p3` | `5245417.cpp` | 2-D prefix sums | 14 | 87 | 84% | 35 | 60% | yes | 200 random inputs agree |
| `coci20c6p3` | `4188081.cpp` | DP, math/mod, strings | 29 | 89 | 67% | 41 | 29% | yes | 200 random inputs agree |
| `dmopc21c2p1` | `3946478.cpp` | greedy, sorting | 14 | 58 | 76% | 21 | 33% | yes | 200 random inputs agree |
| `dmopc21c9p4` (earlier round) | `4565958.cpp` | grids, game | 49 | 97 | 49% | 66 | 26% | yes | 200 random inputs agree |
| `ioi05p6` (earlier round) | `5032744.cpp` | tree DP | 41 | 78 | 47% | 47 | 13% | yes | 200 random inputs agree |
| `mccc4j5` (earlier round) | `4264954.cpp` | graphs (heap) | 20 | 73 | 73% | 36 | 44% | yes | 200 random inputs agree |
| `nccc10s2` (earlier round) | `4214203.cpp` | graphs, memo DP | 12 | 78 | 85% | 43 | 72% | yes | 200 random inputs agree |
| `noi01p5` (earlier round) | `5079272.cpp` | meet in the middle, hashing | 15 | 103 | 85% | 29 | 48% | yes | 200 random inputs agree |
| `noi02p1` (earlier round) | `4793930.cpp` | union-find | 37 | 69 | 46% | 28 | -32% | yes | 200 random inputs agree |
| `noi14p4` | `3821774.cpp` | strings (KMP), multitest | 24 | 88 | 73% | 69 | 65% | yes | 200 random inputs agree |
| `noi16p4` | `4791678.cpp` | segment tree, two pointers | 29 | 98 | 70% | 30 | 3% | yes | 200 random inputs agree |
| `noip09p2` (earlier round) | `5184950.cpp` | number theory, multitest | 52 | 129 | 60% | 60 | 13% | yes | 200 random inputs agree |
| `noip09p3` | `5185139.cpp` | graphs | 33 | 99 | 67% | 66 | 50% | yes | 200 random inputs agree |
| `noip14p3` | `5445668.cpp` | DP | 45 | 92 | 51% | 61 | 26% | yes | 200 random inputs agree |
| `noip18p2` | `5032734.cpp` | DP, multitest | 15 | 71 | 79% | 32 | 53% | yes | 200 random inputs agree |
| `yac1p1` | `4376122.cpp` | math/mod | 16 | 64 | 75% | 33 | 52% | yes | 200 random inputs agree |
| **all 30** | | | 785 | 2588 | 70% | 1289 | 39% | 30 of 30 | |

## Coverage

DP: `ceoi09p3`, `ceoi18p1`, `noip14p3`, `noip18p2`, `coci20c6p3`,
`btoi18p6` (bitmask), `ioi05p6` (tree), `nccc10s2` (memo).
Graphs: `ccc23s4`, `cco24p1`, `noip09p3`, `mccc4j5`, `btoi18p6`.
Trees: `btoi17p3`, `btoi17p6`, `apio07p1`, `ioi05p6`.
Range structures: `ccc21s5` (segment tree), `noi16p4` (lazy segment tree),
`coci13c1p3` (2-D prefix sums). There is no Fenwick-tree port; the two
segment-tree ports cover that slot.
Strings: `ccc20s3`, `noi14p4`, `coci20c6p3`.
Geometry: `cco16p2`.
Math and modular arithmetic: `yac1p1`, `noip09p2`, `aac4p2`,
`coci20c6p3`; meet in the middle: `noi01p5`.
Greedy and two pointers: `dmopc21c2p1`, `btoi17p6`, `noi16p4`.
Interactive: `aac5p2`.
Multitest: `noip18p2`, `noi14p4`, `noip09p2`.
Union-find: `noi02p1`.

## What changed in the prototype

Language and library, each driven by a port: `x => e` lambdas as the plan
specifies; `sort`/`sorted` with `by:` or `less:`; `Subset<D>`; `min`/`max`
over optionals; `List.pop()` returning `?T`; `gcd`, `lcm`;
`D.first_where` and `D.partition_point`; `SegTree<D, T>`; `AddMax<D>`;
`let s: [domain S]Char = read()`; `StrHash`; `Rooted(adj, root)` with
`lca`; `runs`, `distinct`; reverse iteration over lists; `?T == T`;
counting into a map with `+=`; `ask(...)` for interactive problems.

Tooling: `proto/interact.py` (judge harness), `proto/loc.py` (line
counts), and `run_all.sh` now covers interactive ports and raises the stack
limit, because some originals keep megabytes of tables on the stack.
