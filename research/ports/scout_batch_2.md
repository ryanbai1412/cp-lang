# Scout batch 2

Ten hard graph, tree and range-query submissions from the corpus, re-solved
on top of `stdlib/`. This is a Stage-1 critique pass: nothing in `crates/`
or `proto/cpc/` was changed, and what the ports taught us is in
[`research/archive/frictions/friction-backlog-batch-2.md`](../../../research/archive/frictions/friction-backlog-batch-2.md).

Unlike batch 1, these files run. They are written for the reference
interpreter (`proto/cpi/`), which is the only prototype that accepts
`use std.x`, generic structures and operation values, and each one is
checked against its original:

```text
$ python3 proto/ports/scout_batch_2/check.py            # every port, 60 random inputs each
$ python3 proto/ports/scout_batch_2/check.py dpv 200    # one port, 200 inputs
$ python3 proto/cp.py lib-run proto/ports/scout_batch_2/dpv.cp < input
```

`check.py` compiles the original from `corpus/submissions/`, feeds both
sides the same small random inputs and compares the output. All ten agree
on 200 inputs each.

What that does and does not show. The inputs are small (up to about 40
vertices), because the interpreter is slow, so the ports are checked for
answers and not for speed or memory. The interpreter does no static
checking, so nothing here has been through the type checker or the borrow
rules; the backlog names the lines that the real checker would question.
`proto/run_all.sh` only looks at `proto/ports/*.cp`, so this folder does
not affect it.

Where the interpreter lacks a construct the language has (`enum` and
`match`, `min=` on an optional, `[n]T` sized by an `Int`, `any`), the
port uses the nearest spelling that runs. Those lines are listed under
"Gaps in the reference interpreter" in the backlog, so they are not
mistaken for the intended style.

## The ports

The originals are in `corpus/submissions/`.

- `apio09p3.cp` from `5536741.cpp`: strongly connected components, then
  the best path in the condensation. `scc` returns the components as a
  fresh domain in topological order, so the DP is one loop over `C.rev`.
- `ioi11p4.cp` from `3734919.cpp`: Dijkstra from the exits in which a
  chamber is settled the second time it leaves the heap.
- `cco19p5.cp` from `3617425.cpp`: sets of higher neighbours handed to
  the lowest of them, smaller set into larger.
- `cco20p2.cp` from `4561352.cpp`: a greedy over the days from the last,
  with a heap, and the inversions counted in a Fenwick tree as the order
  is built. The original counts them afterwards in an order-statistics
  tree.
- `ccc17s4.cp` from `4899340.cpp`: Kruskal with old pipes ahead of new
  ones of equal cost, then a second union-find to see whether the enhancer
  saves a day.
- `noip18p3.cp` from `4002963.cpp`: binary search on the track length,
  and a greedy pairing of chains at every junction going up the tree.
- `dpv.cp` from `4133644.cpp`: rerooting. Two passes over the preorder
  replace the two recursive functions and the memo table.
- `ioi08p2.cp` from `5107842.cpp`: every component is a cycle with trees
  on it. The trees are peeled leaf first with a queue and each cycle is
  walked once; the original finds the cycle with a recursive DFS and runs
  a memoised tree DP per cycle vertex.
- `noi15p2.cp` from `3775038.cpp`: path-to-root and subtree assignments
  as ranges of the heavy-first preorder over one lazy tree.
- `ioi14p2.cp` from `4843911.cpp`: range clamps pushed down a lazy tree
  and read at the leaves. The original sweeps the columns with a tree over
  time (grader-style original; `check.py` adds a `main` that reads the
  judge's format).

## Line counts

Counted the way `proto/loc.py` counts: blank and comment-only lines never
count. "Stripped" removes from the original what a perfect C++ contest
library would have supplied: the template header, the driver, and here the
hand-written union-find, segment trees, graph and SCC structures, the
order-statistics alias and the rooting and heavy-path DFS.

- apio09p3: 23 lines against 172 (86% fewer), 75 stripped (69% fewer).
- ioi11p4: 23 against 95 (75% fewer), 41 stripped (43% fewer).
- cco19p5: 15 against 53 (71% fewer), 26 stripped (42% fewer).
- cco20p2: 21 against 69 (69% fewer), 34 stripped (38% fewer).
- ccc17s4: 24 against 104 (76% fewer), 50 stripped (52% fewer).
- noip18p3: 51 against 106 (51% fewer), 61 stripped (16% fewer).
- dpv: 22 against 90 (75% fewer), 55 stripped (60% fewer).
- ioi08p2: 39 against 136 (71% fewer), 105 stripped (62% fewer).
- noi15p2: 22 against 175 (87% fewer), 47 stripped (53% fewer).
- ioi14p2: 12 against 99 (87% fewer), 50 stripped (76% fewer).

Together: 252 lines against 1099 (77% fewer) and 544 stripped (53%
fewer). `noip18p3` is the outlier, and the reason is the first item of
the backlog: there is no ordered bag, so the port carries 24 lines that
stand in for `multiset::lower_bound` and `erase`.
