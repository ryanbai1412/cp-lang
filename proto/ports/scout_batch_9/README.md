# Scout batch 9

Ten tree-decomposition and graph-structure submissions from the corpus,
re-solved in CP Lang: heavy-light paths, a centroid, rerooting, Euler
labels, LCA, the virtual-tree problem, 2-SAT and bridges. This is a
Stage-1 critique pass. Nothing in `proto/cpc/` or `crates/` was changed
for it.

Like batch 4, every file here builds with today's prototype and agrees
with its original. Where the prototype forced an awkward line, the port
keeps that line, and
[`research/friction-backlog-batch-9.md`](../../../research/friction-backlog-batch-9.md)
quotes it next to the line we would rather write.

Each port names its original on the `// original:` line of its header.
To stress all ten against those originals:

```
$ proto/ports/scout_batch_9/stress.sh        # 200 random cases each
btoi19p3: stress OK: 200 tests agree (0 rejected by where)
...
```

Two things in `harness/` are new to this batch:

- Three originals are grader-style: they define `LocateCentre`,
  `label` and `find_next_station`, or `find_split`, and have no `main`.
  `harness/<problem>.cpp` includes the original and adds a `main` that
  reads the port's input format.
- Two problems accept more than one answer (cmt21r2ztr and ioi19p2), and
  `cp.py stress` compares outputs word for word. Those ports carry a
  `// checker:` line, and `stress.sh` runs them through
  `harness/judge.py`, which asks the checker instead.

`proto/run_all.sh` only looks at `proto/ports/*.cp`, so this folder does
not affect it.

The ports, with code lines (no blanks or comments) against the original's
lines from its first declaration on, which leaves out the include and debug
template. Harness files are not counted on either side.

- `noi21p1.cp` (33 against 217): Heavy and Light Edges. Paint paths and count repainted edges, with two assign-and-sum structures over the heavy-light layout instead of a hand-written lazy segment tree.
- `utso15p5.cp` (34 against 146): the strictly second-best spanning tree. Kruskal, then the two largest different costs on a tree path.
- `btoi19p3.cp` (30 against 86): Valley. Nearest shop inside a cut-off subtree, as a minimum over the climb from the query vertex.
- `ioi19p2.cp` (45 against 264): Split the Attractions. A spanning tree's centroid, branches merged along the other edges, then two connected patches.
- `coci25c3p4.cp` (36 against 67): rerooting where the value at a vertex is a polynomial; the parent's side is divided back out.
- `ioi10p6.cp` (16 against 42): Traffic Congestion. The original reroots; subtree sums and one total are enough.
- `ioi20p6.cp` (36 against 50): Stations. Labels from an enter-or-leave numbering of the tree, and routing by labels alone.
- `sdoi17r2p4.cp` (54 against 232): Genius Hacker. The original builds a virtual tree per town; sorting each town's roads by dictionary order and chaining neighbours needs only LCA depths.
- `cmt21r2ztr.cp` (42 against 214): median-of-three facts as 2-SAT over "below x" and "above x" claims.
- `noip22p3.cp` (33 against 230): Barracks. Bridges, the tree of blocks between them, and a counting DP up that tree.
