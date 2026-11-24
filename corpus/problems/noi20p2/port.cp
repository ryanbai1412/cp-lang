// Source: NOI 2020 P2 (Destiny), https://dmoj.ca/problem/noi20p2
// limits: 2000 ms, 1024 MB
// Problem: a tree on n <= 5e5 vertices rooted at 1 and m <= 5e5 pairs (u, v), u an ancestor
//   of v. Count the 0/1 labellings of the edges where every pair's path has an edge labelled 1,
//   mod 998244353.
// Solution: dp[v][d] counts labellings of v's subtree whose deepest unsatisfied pair has its
//   top at depth d (0: none). Merging child c: edge (v, c) = 1 moves all of c's states with
//   d <= depth(v) to 0; then new[d] = a[d] * B(<= d) + b[d] * A(< d), a max-convolution done
//   by a std `MergeTrees` merge, whose hook sees both values and both prefix sums. O((n + m) log n).
// original: corpus/submissions/5655455.cpp
// Bounds: n: 1..=5e5; edges.u, edges.v: 1..=n; m: 1..=5e5; paths.u, paths.v: 1..=n

use std.segtree.{MergeTree, MergeTrees}

type Z = Mod<998_244_353>

let n: 1..=5e5 = read()
domain V = 1..=n
domain E = 1..n
let edges: [E](u: V, v: V) = read()
let m: 1..=5e5 = read()
domain P = 0..m
let paths: [P](u: V, v: V) = read()

let tree = Rooted(Graph<V, E>.undirected(edges), V.first!)
domain Dep = 0..=n  // one more than the depth of a pair's top; 0 when nothing is unsatisfied

var need: [V]Dep = Dep(0)
for p in P: need[paths[p].v] max= Dep(tree.depth(paths[p].u) + 1)

var pool = MergeTrees<Dep, Z>()
var dp: [V]MergeTree<Dep> = MergeTree<Dep>()
for u in tree.at.rev:
    let d = tree.depth(u) + 1
    var rt = MergeTree<Dep>()
    pool.set(mut rt, need[u], Z(1))
    for c in tree.children(u):
        // labelling (u, c) with 1 satisfies every pair of c's subtree whose top is above c
        let freed = pool.sum(dp[c], Dep.cuts[0]..Dep.cuts[d + 1])
        rt = pool.merge(rt, dp[c], (va, vb, pa, pb) => va * (pb + vb) + vb * pa, pre_b: freed)
    dp[u] = rt
print(pool.get(dp[V.first!], Dep(0)))
