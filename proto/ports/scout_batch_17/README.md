# Scout batch 17: statement first

Ten medium-to-hard problems from `corpus/`, each solved in CP Lang from the
problem itself before the accepted C++ was opened. The C++ was read only
afterwards, to confirm the input format and to run the two side by side.
Nothing in `proto/cpc/` or `crates/` was changed.

The statements come from memory of the published tasks (IOI, APIO, Baltic OI,
NOI, the AtCoder DP contest). `dmoj.ca/api/v2/problem/<code>` returns only the
name, limits and tags, and the statement pages sit behind a bot challenge.
[`research/friction-backlog-batch-17-statement-first.md`](../../../research/friction-backlog-batch-17-statement-first.md)
has the findings.

Every file builds with today's prototype and agrees with its original:

```
$ proto/ports/scout_batch_17/stress.sh 100 6      # 100 small cases each
apio07p2: stress OK: 100 tests agree (0 rejected by where)
...
$ proto/ports/scout_batch_17/stress.sh 20 2000    # 20 medium cases each
```

Two tasks (`ioi12p4`, `ioi13p1`) are judged through a function, so their
originals have no `main`. `harness/` holds a small `main` for each that reads
the same data from standard input.

The ports:

- `dpz.cp`: Frog 3. Lowest of a set of lines, as a Li Chao tree over the stones.
- `dpw.cp`: Intervals. A sweep with range add and range maximum.
- `ioi11p3.cp`: Rice Hub. Two pointers over prefix sums.
- `ioi13p1.cp`: Dreaming. Radius and diameter of every tree in a forest.
- `apio07p2.cp`: Backup. Greedy with undo, on a heap and a linked row of gaps.
- `noi10p2.cp`: Super Piano. The k best subarray sums from a heap of ranges.
- `btoi19p3.cp`: Alpine Valley. Offline in preorder, with a range minimum over depth.
- `ioi12p4.cp`: Ideal City. Runs of blocks form a tree; count pairs across each edge.
- `ioi07p4.cp`: Miners. DP over the last two foods at each mine.
- `ioi08p4.cp`: Teleporters. The ride is a path, the rest are loops to splice in.
