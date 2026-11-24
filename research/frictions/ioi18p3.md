# Friction report: ioi18p3

## Dream

```cp
let (domain H, human) = kruskal_tree(roads, w: e => -min(roads[e].u, roads[e].v))
let (domain W, wolf) = kruskal_tree(roads, w: e => max(roads[e].u, roads[e].v))
print_lines(trips.map(t => human.subtree(human.top(t.s, while: x => x >= t.l))
    .meets(wolf.subtree(wolf.top(t.e, while: x => x <= t.r)))))
```

## Hand-rolled reachability trees become `kruskal_tree`

The v0.2 port built each reconstruction tree with its own `find` and parent array
(`reach_tree`). The rewrite builds both with std `kruskal_tree`, weighted by the smaller
endpoint (human, largest first) and the larger endpoint (wolf). The subtree-meets test is an
offline rectangle count with a std `Fenwick` over the wolf preorder.

## KruskalTree kept a value table it never used

`KruskalTree` climbed with a `Lift<N, Int>`: about 19 levels of jump pointers plus an equal
table of always-zero edge values, two trees over 4e5 nodes, 362 MB at the max case against the
memory limit. std now keeps a `JumpTree<N>` (three arrays, `O(log n)` climbs) instead, which
answers `reach`, `top`, `bottleneck` and `tree` the same way.

## Output size

The generated C++ is about 54 KB, over the 32 KB target of judge `generic` (warning
W0201). That comes from two kruskal trees plus Fenwick and rooted-tree code in std.

## Verification

30 random cases agree with the v0.2 port and 30 with `orig.cpp` (the grader-interface AC
wrapped in a stdin main). On the max case the port runs in 0.50 s and 103 MB against the
accepted C++'s 0.49 s and 112 MB (the C++ with an unlimited stack, as its DFS recurses).
