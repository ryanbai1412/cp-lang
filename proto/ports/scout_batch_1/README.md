# Scout batch 1

Ten corpus submissions re-solved the way we would like to write them. This
is a Stage-1 critique pass: the files are design sketches, not regression
ports. Every line that the prototype rejects today is marked `// WISH Fn`,
and `Fn` is an item in [`research/friction-backlog.md`](../../../research/friction-backlog.md).

None of the ten builds with `proto/cpc` yet, and none has been run against
its original. `proto/run_all.sh` only looks at `proto/ports/*.cp`, so this
folder does not affect it. When a Stage-2 sprint lands the items a file
needs, move the file up to `proto/ports/` with its original and stress it.

The originals are in `corpus/submissions/`:

- `ccc18s5.cp` from `4329864.cpp`: Kruskal on a product graph, two union-finds.
- `ccc19s5.cp` from `5119937.cpp`: maximum over every size-k sub-triangle.
- `cco18p1.cp` from `4801863.cpp`: pairing DP over two sequences.
- `ioi04p4.cp` from `4501742.cpp`: guillotine-cut DP over slab sizes.
- `ioi10p3.cp` from `4692375.cpp`: binary search on the median with 2-D prefix counts (grader-style original).
- `ccc21s4.cp` from `4323468.cpp`: reverse BFS plus a minimum kept under swaps.
- `ccc22s4.cp` from `4371347.cpp`: counting on a circle with wrapping range sums.
- `ccc05s5.cp` from `4173126.cpp`: online rank queries, float output.
- `ioi11p2.cp` from `4848359.cpp`: small-to-large map merging on a tree (grader-style original).
- `cco20p1.cp` from `4561351.cpp`: sweep over interval end points.

Two of them changed shape and need the most care when they are first
stressed: `ccc22s4.cp` counts the bad triples directly instead of the
original's doubled prefix arrays, and `ioi11p2.cp` walks the preorder
backwards instead of recursing.
