// Source: IOI 2020 Day 1 P2 (Connecting Supertrees), https://dmoj.ca/problem/ioi20p2
// Problem: n <= 1000 towers and a symmetric matrix p with p[i][i] = 1 and 0 <= p[i][j] <= 3.
//   Build bridges (a simple undirected graph) with exactly p[i][j] simple paths from i to j
//   for every pair, or report that it is impossible.
// Solution: in a valid graph each component is a cycle with trees hanging off it. Towers with
//   p == 1 form one tree each (a star on its first tower); tree roots with p == 2 share a ring,
//   which needs at least 3 trees (a ring of 2 is impossible), and p == 3 never happens. Build
//   that, then check every p[i][j] is 1 within a tree, 2 within a ring and 0 otherwise. O(n^2).
// original: corpus/submissions/4724578.cpp (grader function construct, which calls build)
// stdin here, as the IOI sample grader: n, then the n x n matrix p.
// stdout: 0, or 1 followed by the bridge matrix.
let n: 1..=1000 = read()
domain V = 0..n
let p: [V, V]0..=3 = read()

// a tree is the towers with p == 1, a component those with p > 0; each is named by its
// first tower, and a tree hangs off its name as a star
let tree: [V]V = [V] v => V.find(u => p[v, u] == 1) ?? v
let comp: [V]V = [V] v => V.find(u => p[v, u] > 0) ?? v
var bridges: List<(V, V)> = [(v, tree[v]) for v in V if tree[v] != v]

// the trees of a component sit on one ring, which needs at least three of them
var ring: [V]List<V> = []
for v in V:
    if tree[v] == v: ring[comp[v]].push(v)
for r in ring:
    if r.len > 2:
        bridges.extend(r.cyclic_pairs())

// the layout determines every path count, so p must agree with it exactly
let ok = all(ring[c].len != 2 for c in V) and all(
    p[i, j] == (if tree[i] == tree[j]: 1 elif comp[i] == comp[j]: 2 else: 0) for i in V for j in V
)
if not ok:
    print(0)
else:
    print(1)
    var bridge: [V, V]0..=1 = 0
    for (a, b) in bridges:
        for (x, y) in [(a, b), (b, a)]: bridge[x, y] = 1
    for i in V: print(bridge[i])
